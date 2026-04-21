//! Signature Maker — stroke model + Catmull-Rom smoothing + variable-width render.
//!
//! Fourteenth Graphics project. Distinctive moves:
//!
//!   * **Stroke model with pressure + timestamp** — `Point { x, y,
//!     pressure: u16 (0..=1024), t_ms: u64 }`. A stroke is a list of
//!     points (pen-down → pen-up); a signature is a list of strokes.
//!   * **Catmull-Rom spline smoothing** — takes raw input points and
//!     interpolates a smoother curve with `steps` intermediate samples
//!     per segment. Pure integer-scaled math for cross-language parity.
//!   * **Variable-width raster rendering** — pen thickness is derived
//!     from pressure (scaled by a `max_width` config). Thick strokes at
//!     high pressure; thin at low. Distinct from G123's fixed-width
//!     Bresenham arrow.
//!   * **Bounding-box normalization** — translate + scale a signature
//!     to a unit coordinate system. Two signatures at different sizes
//!     become directly comparable.
//!   * **Similarity via point-pair distance** — normalized signatures
//!     compared by summing point-to-nearest-point distance. Same shape
//!     → low distance.

use super::grayscale::{Image, Rgb};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Point {
    pub x: i32,
    pub y: i32,
    pub pressure: u16, // 0..=1024
    pub t_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Stroke {
    pub points: Vec<Point>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Signature {
    pub strokes: Vec<Stroke>,
}

impl Signature {
    pub fn new() -> Self { Self { strokes: vec![] } }

    pub fn add_stroke(&mut self, stroke: Stroke) {
        self.strokes.push(stroke);
    }

    pub fn point_count(&self) -> usize {
        self.strokes.iter().map(|s| s.points.len()).sum()
    }

    /// Returns (min_x, min_y, max_x, max_y) across all points, or None if empty.
    pub fn bounding_box(&self) -> Option<(i32, i32, i32, i32)> {
        let mut min_x = i32::MAX;
        let mut min_y = i32::MAX;
        let mut max_x = i32::MIN;
        let mut max_y = i32::MIN;
        let mut any = false;
        for stroke in &self.strokes {
            for p in &stroke.points {
                if p.x < min_x { min_x = p.x; }
                if p.y < min_y { min_y = p.y; }
                if p.x > max_x { max_x = p.x; }
                if p.y > max_y { max_y = p.y; }
                any = true;
            }
        }
        if any { Some((min_x, min_y, max_x, max_y)) } else { None }
    }
}

/// Catmull-Rom spline interpolation between four control points.
/// `t` is in 0..=1000 (integer scale). Returns the interpolated (x, y).
pub fn catmull_rom(p0: (i32, i32), p1: (i32, i32), p2: (i32, i32), p3: (i32, i32),
                    t: i64) -> (i32, i32) {
    // classic CR: 0.5 * ((2*p1) + (-p0 + p2)*t + (2p0 - 5p1 + 4p2 - p3)*t^2 + (-p0 + 3p1 - 3p2 + p3)*t^3)
    // we scale t by 1000: t_scaled = t in 0..=1000
    let t2 = t * t;               // 0..=1_000_000
    let t3 = t2 * t / 1_000;      // keep scale at ~1000^2 for multiplication sanity

    let compute = |a: i64, b: i64, c: i64, d: i64| -> i64 {
        // a = 2*p1
        // b = (-p0 + p2) * t / 1000
        // c = (2p0 - 5p1 + 4p2 - p3) * t2 / 1_000_000
        // d = (-p0 + 3p1 - 3p2 + p3) * t3 / 1_000_000_000
        let term_a = 2 * b;                                // 2 * p1
        let term_b = (-a + c) * t;                          // (-p0 + p2) * t
        let term_c = (2 * a - 5 * b + 4 * c - d) * t2;      // * t^2
        let term_d = (-a + 3 * b - 3 * c + d) * t3;         // * t^3
        // result = 0.5 * (term_a + term_b/1000 + term_c/1_000_000 + term_d/1_000_000_000)
        // Scale: we keep the 1000-based scale throughout; final /= 2
        let sum = term_a * 1_000_000
                + term_b * 1_000
                + term_c
                + term_d / 1_000;
        sum / 2_000_000
    };
    let x = compute(p0.0 as i64, p1.0 as i64, p2.0 as i64, p3.0 as i64);
    let y = compute(p0.1 as i64, p1.1 as i64, p2.1 as i64, p3.1 as i64);
    (x as i32, y as i32)
}

/// Smooth a stroke via Catmull-Rom. Returns a new stroke with
/// `steps` interpolated points per segment. If fewer than 4 input
/// points, returns the input unchanged.
pub fn smooth_stroke(stroke: &Stroke, steps: u32) -> Stroke {
    let pts = &stroke.points;
    if pts.len() < 4 || steps == 0 {
        return stroke.clone();
    }
    let mut out = Vec::with_capacity(pts.len() * steps as usize);
    out.push(pts[0]);
    for i in 0..pts.len() - 3 {
        let p0 = (pts[i].x, pts[i].y);
        let p1 = (pts[i + 1].x, pts[i + 1].y);
        let p2 = (pts[i + 2].x, pts[i + 2].y);
        let p3 = (pts[i + 3].x, pts[i + 3].y);
        for s in 1..=steps {
            let t = (s as i64 * 1_000) / steps as i64;
            let (x, y) = catmull_rom(p0, p1, p2, p3, t);
            // Interpolate pressure and timestamp linearly between p1 and p2
            let pressure = pts[i + 1].pressure as i64
                + (pts[i + 2].pressure as i64 - pts[i + 1].pressure as i64) * t / 1_000;
            let t_ms = pts[i + 1].t_ms as i64
                + (pts[i + 2].t_ms as i64 - pts[i + 1].t_ms as i64) * t / 1_000;
            out.push(Point {
                x, y,
                pressure: pressure.clamp(0, 1024) as u16,
                t_ms: t_ms.max(0) as u64,
            });
        }
    }
    out.push(*pts.last().unwrap());
    Stroke { points: out }
}

/// Normalize a signature: translate to origin, scale to fit (target_w, target_h).
/// Returns a new signature.
pub fn normalize(sig: &Signature, target_w: i32, target_h: i32) -> Signature {
    let Some((min_x, min_y, max_x, max_y)) = sig.bounding_box() else {
        return sig.clone();
    };
    let w = (max_x - min_x).max(1);
    let h = (max_y - min_y).max(1);
    let scale_x = (target_w as i64 * 1000) / w as i64; // scaled by 1000
    let scale_y = (target_h as i64 * 1000) / h as i64;
    let strokes = sig.strokes.iter().map(|s| {
        let points = s.points.iter().map(|p| {
            let nx = ((p.x - min_x) as i64 * scale_x) / 1000;
            let ny = ((p.y - min_y) as i64 * scale_y) / 1000;
            Point {
                x: nx as i32,
                y: ny as i32,
                pressure: p.pressure,
                t_ms: p.t_ms,
            }
        }).collect();
        Stroke { points }
    }).collect();
    Signature { strokes }
}

/// Similarity distance: for each point in `a`, find the nearest point in `b`;
/// sum the squared distances. Lower = more similar. Same signatures → 0.
/// Both signatures should be normalized to the same target box first.
pub fn similarity_distance(a: &Signature, b: &Signature) -> u64 {
    let mut total: u64 = 0;
    for stroke_a in &a.strokes {
        for p in &stroke_a.points {
            let mut min_d2 = u64::MAX;
            for stroke_b in &b.strokes {
                for q in &stroke_b.points {
                    let dx = (p.x - q.x) as i64;
                    let dy = (p.y - q.y) as i64;
                    let d2 = (dx * dx + dy * dy) as u64;
                    if d2 < min_d2 { min_d2 = d2; }
                }
            }
            total = total.saturating_add(min_d2);
        }
    }
    total
}

/// Variable-width render: rasterize all strokes into an image.
/// Line width at each segment is derived from pressure (scaled by max_width).
/// Uses Bresenham-based stamping (disk stamps along the line).
pub fn render_to_image(sig: &Signature, width: u32, height: u32,
                       max_width: u32, bg: Rgb, ink: Rgb) -> Image {
    let mut img = Image::filled(width, height, bg);
    for stroke in &sig.strokes {
        if stroke.points.len() < 2 { continue; }
        for i in 0..stroke.points.len() - 1 {
            let a = stroke.points[i];
            let b = stroke.points[i + 1];
            let w_a = (a.pressure as u32 * max_width) / 1024;
            let w_b = (b.pressure as u32 * max_width) / 1024;
            let w_seg = ((w_a + w_b) / 2).max(1);
            stamp_line(&mut img, a.x, a.y, b.x, b.y, w_seg, ink);
        }
    }
    img
}

fn stamp_disk(img: &mut Image, cx: i32, cy: i32, radius: u32, color: Rgb) {
    let r = radius as i32;
    let r2 = (r * r) as i64;
    for dy in -r..=r {
        for dx in -r..=r {
            let d2 = (dx as i64 * dx as i64 + dy as i64 * dy as i64);
            if d2 > r2 { continue; }
            let x = cx + dx;
            let y = cy + dy;
            if x < 0 || y < 0 { continue; }
            if (x as u32) >= img.width || (y as u32) >= img.height { continue; }
            img.set(x as u32, y as u32, color);
        }
    }
}

fn stamp_line(img: &mut Image, x1: i32, y1: i32, x2: i32, y2: i32,
               width: u32, color: Rgb) {
    let radius = (width / 2).max(1);
    let dx = (x2 - x1).abs();
    let dy = -(y2 - y1).abs();
    let sx = if x1 < x2 { 1 } else { -1 };
    let sy = if y1 < y2 { 1 } else { -1 };
    let mut err = dx + dy;
    let mut x = x1;
    let mut y = y1;
    loop {
        stamp_disk(img, x, y, radius, color);
        if x == x2 && y == y2 { break; }
        let e2 = 2 * err;
        if e2 >= dy { err += dy; x += sx; }
        if e2 <= dx { err += dx; y += sy; }
    }
}

// ---------- tests ----------

#[cfg(test)]
mod tests {
    use super::*;

    fn p(x: i32, y: i32, pressure: u16, t: u64) -> Point {
        Point { x, y, pressure, t_ms: t }
    }

    #[test]
    fn empty_signature_has_no_bbox() {
        let sig = Signature::new();
        assert!(sig.bounding_box().is_none());
    }

    #[test]
    fn bbox_spans_all_points() {
        let mut sig = Signature::new();
        sig.add_stroke(Stroke { points: vec![p(10, 20, 500, 0), p(30, 50, 600, 100)] });
        sig.add_stroke(Stroke { points: vec![p(5, 15, 700, 200), p(40, 60, 800, 300)] });
        assert_eq!(sig.bounding_box(), Some((5, 15, 40, 60)));
    }

    #[test]
    fn point_count_sums_across_strokes() {
        let mut sig = Signature::new();
        sig.add_stroke(Stroke { points: vec![p(0, 0, 500, 0); 3] });
        sig.add_stroke(Stroke { points: vec![p(0, 0, 500, 0); 5] });
        assert_eq!(sig.point_count(), 8);
    }

    #[test]
    fn smooth_stroke_preserves_endpoints() {
        let points = vec![
            p(0, 0, 500, 0),
            p(10, 10, 500, 100),
            p(20, 10, 500, 200),
            p(30, 0, 500, 300),
        ];
        let stroke = Stroke { points: points.clone() };
        let smoothed = smooth_stroke(&stroke, 5);
        assert!(smoothed.points.len() > points.len());
        assert_eq!(smoothed.points.first().unwrap().x, 0);
        assert_eq!(smoothed.points.last().unwrap().x, 30);
    }

    #[test]
    fn smooth_stroke_short_input_unchanged() {
        let stroke = Stroke { points: vec![p(0, 0, 500, 0), p(10, 10, 500, 100)] };
        let smoothed = smooth_stroke(&stroke, 10);
        assert_eq!(smoothed.points.len(), 2);
    }

    #[test]
    fn normalize_fits_to_target_box() {
        let mut sig = Signature::new();
        sig.add_stroke(Stroke { points: vec![
            p(100, 200, 500, 0),
            p(300, 200, 500, 100),
            p(300, 400, 500, 200),
        ]});
        let norm = normalize(&sig, 1000, 1000);
        let (min_x, min_y, max_x, max_y) = norm.bounding_box().unwrap();
        assert_eq!(min_x, 0);
        assert_eq!(min_y, 0);
        assert_eq!(max_x, 1000);
        assert_eq!(max_y, 1000);
    }

    #[test]
    fn normalize_preserves_pressure() {
        let mut sig = Signature::new();
        sig.add_stroke(Stroke { points: vec![p(0, 0, 512, 0), p(10, 10, 768, 100)] });
        let norm = normalize(&sig, 100, 100);
        assert_eq!(norm.strokes[0].points[0].pressure, 512);
        assert_eq!(norm.strokes[0].points[1].pressure, 768);
    }

    #[test]
    fn similarity_self_is_zero() {
        let mut sig = Signature::new();
        sig.add_stroke(Stroke { points: vec![p(10, 20, 500, 0), p(30, 40, 500, 100)] });
        assert_eq!(similarity_distance(&sig, &sig), 0);
    }

    #[test]
    fn similarity_different_signatures_positive() {
        let mut sig_a = Signature::new();
        sig_a.add_stroke(Stroke { points: vec![p(0, 0, 500, 0), p(10, 10, 500, 100)] });
        let mut sig_b = Signature::new();
        sig_b.add_stroke(Stroke { points: vec![p(100, 100, 500, 0), p(200, 200, 500, 100)] });
        assert!(similarity_distance(&sig_a, &sig_b) > 0);
    }

    #[test]
    fn similarity_after_normalization_same_shape_small() {
        let mut sig_small = Signature::new();
        sig_small.add_stroke(Stroke { points: vec![
            p(0, 0, 500, 0), p(10, 0, 500, 100), p(10, 10, 500, 200), p(0, 10, 500, 300),
        ]});
        let mut sig_big = Signature::new();
        sig_big.add_stroke(Stroke { points: vec![
            p(0, 0, 500, 0), p(100, 0, 500, 100), p(100, 100, 500, 200), p(0, 100, 500, 300),
        ]});
        // Same shape, different size. After normalize to same target, similarity → 0.
        let n_small = normalize(&sig_small, 1000, 1000);
        let n_big = normalize(&sig_big, 1000, 1000);
        assert_eq!(similarity_distance(&n_small, &n_big), 0);
    }

    #[test]
    fn render_produces_ink_pixels() {
        let mut sig = Signature::new();
        sig.add_stroke(Stroke { points: vec![
            p(5, 5, 512, 0), p(25, 25, 1024, 100),
        ]});
        let img = render_to_image(&sig, 40, 40, 6, Rgb::new(255, 255, 255), Rgb::new(0, 0, 0));
        // Should have at least one black pixel
        let has_ink = img.pixels.iter().any(|p| p == &Rgb::new(0, 0, 0));
        assert!(has_ink);
    }

    #[test]
    fn render_preserves_width_with_bg() {
        let sig = Signature::new();
        let img = render_to_image(&sig, 10, 8, 4, Rgb::new(255, 255, 255), Rgb::new(0, 0, 0));
        // All pixels white
        let all_white = img.pixels.iter().all(|p| p == &Rgb::new(255, 255, 255));
        assert!(all_white);
    }

    #[test]
    fn catmull_rom_endpoints() {
        let p0 = (0, 0);
        let p1 = (10, 10);
        let p2 = (20, 10);
        let p3 = (30, 0);
        // At t=0, result should equal p1
        let (x0, y0) = catmull_rom(p0, p1, p2, p3, 0);
        assert_eq!((x0, y0), (10, 10));
        // At t=1000, result should equal p2
        let (x1, y1) = catmull_rom(p0, p1, p2, p3, 1000);
        assert_eq!((x1, y1), (20, 10));
    }
}
