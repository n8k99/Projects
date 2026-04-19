//! Online White Board — shared canvas of strokes with spatial queries.
//!
//! First Rosetta Stone project with **spatial/geometric data**: coordinates,
//! bounding boxes, distance-based queries. Strokes are first-class entities
//! with identity (you can delete or highlight a specific stroke), ordering
//! (insertion order = z-order, later strokes draw on top), and authorship
//! (every stroke knows who made it).
//!
//! Like G069 WYSIWYG, the data structure IS the display — render is a walk
//! over strokes in z-order producing SVG or any other target format. Unlike
//! WYSIWYG, the coordinates are 2D spatial, not 1D textual, which opens up
//! a new class of queries (near-point, within-bbox, intersect-rectangle).

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    pub fn new(x: f64, y: f64) -> Self { Self { x, y } }
    pub fn distance_to(&self, other: &Point) -> f64 {
        ((self.x - other.x).powi(2) + (self.y - other.y).powi(2)).sqrt()
    }
}

pub type StrokeId = u64;

#[derive(Debug, Clone)]
pub struct Stroke {
    pub id: StrokeId,
    pub author: String,
    pub color: String,
    pub thickness: f64,
    pub points: Vec<Point>,
}

impl Stroke {
    /// Axis-aligned bounding box of the stroke's points, or None if empty.
    pub fn bbox(&self) -> Option<(Point, Point)> {
        let first = self.points.first()?;
        let (mut minx, mut miny) = (first.x, first.y);
        let (mut maxx, mut maxy) = (first.x, first.y);
        for p in &self.points[1..] {
            if p.x < minx { minx = p.x; }
            if p.x > maxx { maxx = p.x; }
            if p.y < miny { miny = p.y; }
            if p.y > maxy { maxy = p.y; }
        }
        Some((Point::new(minx, miny), Point::new(maxx, maxy)))
    }

    /// Distance from the stroke to `point`, defined as the minimum distance
    /// from `point` to any of the stroke's polyline points. (A proper
    /// implementation would also consider segments between points; G074 uses
    /// vertex distance for clarity and sufficiency at small scale.)
    pub fn distance_to(&self, point: &Point) -> f64 {
        self.points.iter().map(|p| p.distance_to(point))
            .fold(f64::INFINITY, f64::min)
    }
}

pub struct Whiteboard {
    strokes: Vec<Stroke>,   // insertion order = z-order, later draws on top
    next_id: StrokeId,
}

impl Whiteboard {
    pub fn new() -> Self { Self { strokes: Vec::new(), next_id: 1 } }

    pub fn add_stroke(&mut self, author: &str, color: &str,
                      thickness: f64, points: Vec<Point>) -> StrokeId {
        let id = self.next_id;
        self.next_id += 1;
        self.strokes.push(Stroke {
            id, author: author.into(), color: color.into(), thickness, points,
        });
        id
    }

    pub fn remove_stroke(&mut self, id: StrokeId) -> bool {
        match self.strokes.iter().position(|s| s.id == id) {
            Some(pos) => { self.strokes.remove(pos); true }
            None => false,
        }
    }

    pub fn clear(&mut self) { self.strokes.clear(); }

    pub fn stroke_count(&self) -> usize { self.strokes.len() }

    pub fn strokes(&self) -> &[Stroke] { &self.strokes }

    pub fn strokes_by_author(&self, author: &str) -> Vec<&Stroke> {
        self.strokes.iter().filter(|s| s.author == author).collect()
    }

    /// Strokes whose distance to `point` is ≤ `radius`. Ordered by Z-order
    /// (insertion order), not by distance — the caller sorts if they want.
    pub fn strokes_near(&self, point: &Point, radius: f64) -> Vec<&Stroke> {
        self.strokes.iter()
            .filter(|s| s.distance_to(point) <= radius)
            .collect()
    }

    /// Bounding box of every stroke on the board, or None if empty.
    pub fn bounding_box(&self) -> Option<(Point, Point)> {
        let mut iter = self.strokes.iter().filter_map(|s| s.bbox());
        let first = iter.next()?;
        let (mut minx, mut miny) = (first.0.x, first.0.y);
        let (mut maxx, mut maxy) = (first.1.x, first.1.y);
        for (lo, hi) in iter {
            if lo.x < minx { minx = lo.x; }
            if lo.y < miny { miny = lo.y; }
            if hi.x > maxx { maxx = hi.x; }
            if hi.y > maxy { maxy = hi.y; }
        }
        Some((Point::new(minx, miny), Point::new(maxx, maxy)))
    }

    /// Render the board as SVG. Walks strokes in z-order.
    pub fn to_svg(&self, width: f64, height: f64) -> String {
        let mut out = format!(
            "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{width}\" height=\"{height}\">\n"
        );
        for s in &self.strokes {
            if s.points.is_empty() { continue; }
            let pts: Vec<String> = s.points.iter()
                .map(|p| format!("{},{}", p.x, p.y)).collect();
            out.push_str(&format!(
                "  <polyline fill=\"none\" stroke=\"{}\" stroke-width=\"{}\" points=\"{}\"/>\n",
                s.color, s.thickness, pts.join(" ")
            ));
        }
        out.push_str("</svg>");
        out
    }
}

impl Default for Whiteboard {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pts(ps: &[(f64, f64)]) -> Vec<Point> {
        ps.iter().map(|(x, y)| Point::new(*x, *y)).collect()
    }

    #[test]
    fn add_stroke_assigns_incrementing_ids() {
        let mut b = Whiteboard::new();
        let a = b.add_stroke("alice", "#000", 2.0, pts(&[(0.0, 0.0), (10.0, 10.0)]));
        let c = b.add_stroke("alice", "#000", 2.0, pts(&[(20.0, 0.0), (30.0, 10.0)]));
        assert_ne!(a, c);
        assert_eq!(b.stroke_count(), 2);
    }

    #[test]
    fn z_order_is_insertion_order() {
        let mut b = Whiteboard::new();
        b.add_stroke("a", "red",   2.0, pts(&[(0.0, 0.0)]));
        b.add_stroke("a", "blue",  2.0, pts(&[(1.0, 1.0)]));
        b.add_stroke("a", "green", 2.0, pts(&[(2.0, 2.0)]));
        let colors: Vec<&str> = b.strokes().iter().map(|s| s.color.as_str()).collect();
        assert_eq!(colors, vec!["red", "blue", "green"]);
    }

    #[test]
    fn remove_stroke_removes_by_id() {
        let mut b = Whiteboard::new();
        let id = b.add_stroke("a", "#000", 1.0, pts(&[(0.0, 0.0)]));
        b.add_stroke("a", "#000", 1.0, pts(&[(5.0, 5.0)]));
        assert!(b.remove_stroke(id));
        assert_eq!(b.stroke_count(), 1);
        assert!(!b.remove_stroke(id));   // already gone
    }

    #[test]
    fn strokes_by_author_filters() {
        let mut b = Whiteboard::new();
        b.add_stroke("alice", "#000", 1.0, pts(&[(0.0, 0.0)]));
        b.add_stroke("bob",   "#000", 1.0, pts(&[(1.0, 1.0)]));
        b.add_stroke("alice", "#000", 1.0, pts(&[(2.0, 2.0)]));
        assert_eq!(b.strokes_by_author("alice").len(), 2);
        assert_eq!(b.strokes_by_author("bob").len(), 1);
    }

    #[test]
    fn strokes_near_returns_only_within_radius() {
        let mut b = Whiteboard::new();
        b.add_stroke("a", "#000", 1.0, pts(&[(0.0, 0.0), (1.0, 0.0)]));
        b.add_stroke("a", "#000", 1.0, pts(&[(100.0, 100.0)]));
        let near = b.strokes_near(&Point::new(0.5, 0.0), 1.0);
        assert_eq!(near.len(), 1);
        assert_eq!(near[0].points.len(), 2);
    }

    #[test]
    fn bounding_box_covers_all_strokes() {
        let mut b = Whiteboard::new();
        b.add_stroke("a", "#000", 1.0, pts(&[(10.0, 20.0), (30.0, 40.0)]));
        b.add_stroke("a", "#000", 1.0, pts(&[(-5.0, 15.0), (5.0, 50.0)]));
        let (lo, hi) = b.bounding_box().unwrap();
        assert_eq!(lo, Point::new(-5.0, 15.0));
        assert_eq!(hi, Point::new(30.0, 50.0));
    }

    #[test]
    fn empty_board_has_no_bounding_box() {
        let b = Whiteboard::new();
        assert!(b.bounding_box().is_none());
    }

    #[test]
    fn svg_renders_every_stroke_in_z_order() {
        let mut b = Whiteboard::new();
        b.add_stroke("a", "red",  2.0, pts(&[(0.0, 0.0), (10.0, 0.0)]));
        b.add_stroke("a", "blue", 3.0, pts(&[(0.0, 5.0), (10.0, 5.0)]));
        let svg = b.to_svg(100.0, 100.0);
        assert!(svg.contains("<svg"));
        // Red appears before blue — z-order preserved.
        let red_pos = svg.find("red").unwrap();
        let blue_pos = svg.find("blue").unwrap();
        assert!(red_pos < blue_pos);
        assert!(svg.contains("stroke-width=\"2\""));
        assert!(svg.contains("stroke-width=\"3\""));
    }

    #[test]
    fn clear_removes_everything() {
        let mut b = Whiteboard::new();
        b.add_stroke("a", "#000", 1.0, pts(&[(0.0, 0.0)]));
        b.add_stroke("a", "#000", 1.0, pts(&[(1.0, 1.0)]));
        b.clear();
        assert_eq!(b.stroke_count(), 0);
        assert!(b.bounding_box().is_none());
    }

    #[test]
    fn remove_preserves_z_order_of_remaining() {
        let mut b = Whiteboard::new();
        b.add_stroke("a", "red",   1.0, pts(&[(0.0, 0.0)]));
        let mid = b.add_stroke("a", "blue",  1.0, pts(&[(1.0, 1.0)]));
        b.add_stroke("a", "green", 1.0, pts(&[(2.0, 2.0)]));
        b.remove_stroke(mid);
        let colors: Vec<&str> = b.strokes().iter().map(|s| s.color.as_str()).collect();
        assert_eq!(colors, vec!["red", "green"]);
    }
}
