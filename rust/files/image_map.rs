//! Image Map Generator — clickable regions on an image with hit testing.
//!
//! Thirteenth Files project. An image map is a list of shaped regions
//! (rect, circle, polygon) on a 2D canvas; given a click at (x, y), find
//! the region containing it. The defining novel moves:
//!   * **Shape polymorphism** — every shape answers `contains(x, y) -> bool`.
//!   * **Z-order for overlaps** — topmost-first iteration means the first
//!      matching region wins.
//!   * **Point-in-polygon via ray casting** — the standard algorithm;
//!      every GIS library, every CAD tool, every SVG renderer implements it.
//!
//! Coordinates are integers. Polygons are closed implicitly by the
//! ray-casting algorithm; no explicit closing edge needed.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Shape {
    Rect { x: i32, y: i32, w: i32, h: i32 },
    Circle { cx: i32, cy: i32, r: i32 },
    Polygon { points: Vec<(i32, i32)> },
}

impl Shape {
    /// Point-in-shape test.
    pub fn contains(&self, x: i32, y: i32) -> bool {
        match self {
            Shape::Rect { x: rx, y: ry, w, h } => {
                x >= *rx && x < *rx + *w && y >= *ry && y < *ry + *h
            }
            Shape::Circle { cx, cy, r } => {
                let dx = x - cx;
                let dy = y - cy;
                dx * dx + dy * dy <= r * r
            }
            Shape::Polygon { points } => point_in_polygon(x, y, points),
        }
    }

    /// Axis-aligned bounding box (x, y, w, h).
    pub fn bbox(&self) -> (i32, i32, i32, i32) {
        match self {
            Shape::Rect { x, y, w, h } => (*x, *y, *w, *h),
            Shape::Circle { cx, cy, r } => (cx - r, cy - r, 2 * r, 2 * r),
            Shape::Polygon { points } => {
                let mut min_x = i32::MAX;
                let mut min_y = i32::MAX;
                let mut max_x = i32::MIN;
                let mut max_y = i32::MIN;
                for (px, py) in points {
                    if *px < min_x { min_x = *px; }
                    if *py < min_y { min_y = *py; }
                    if *px > max_x { max_x = *px; }
                    if *py > max_y { max_y = *py; }
                }
                (min_x, min_y, max_x - min_x, max_y - min_y)
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Region {
    pub id: u32,
    pub shape: Shape,
    pub action: String,
    pub alt: String,
    pub z: i32,  // higher = on top
}

#[derive(Debug, Clone)]
pub struct ImageMap {
    pub regions: Vec<Region>,
}

impl ImageMap {
    pub fn new() -> Self { Self { regions: Vec::new() } }

    pub fn add(&mut self, region: Region) {
        self.regions.push(region);
    }

    /// Find the topmost region containing (x, y), if any.
    pub fn hit_test(&self, x: i32, y: i32) -> Option<&Region> {
        let mut sorted: Vec<&Region> = self.regions.iter().collect();
        sorted.sort_by(|a, b| b.z.cmp(&a.z));
        sorted.into_iter().find(|r| r.shape.contains(x, y))
    }

    /// All regions containing (x, y), topmost first.
    pub fn hit_test_all(&self, x: i32, y: i32) -> Vec<&Region> {
        let mut sorted: Vec<&Region> = self.regions.iter().collect();
        sorted.sort_by(|a, b| b.z.cmp(&a.z));
        sorted.into_iter().filter(|r| r.shape.contains(x, y)).collect()
    }

    /// Render to HTML <map> element text.
    pub fn to_html(&self, map_name: &str) -> String {
        let mut out = format!("<map name=\"{map_name}\">\n");
        for r in &self.regions {
            let (shape_name, coords) = match &r.shape {
                Shape::Rect { x, y, w, h } => ("rect",
                    format!("{x},{y},{},{}", x + w, y + h)),
                Shape::Circle { cx, cy, r } => ("circle",
                    format!("{cx},{cy},{r}")),
                Shape::Polygon { points } => ("poly",
                    points.iter().map(|(x, y)| format!("{x},{y}"))
                        .collect::<Vec<_>>().join(",")),
            };
            out.push_str(&format!(
                "  <area shape=\"{shape_name}\" coords=\"{coords}\" \
                 href=\"{}\" alt=\"{}\">\n", r.action, r.alt));
        }
        out.push_str("</map>\n");
        out
    }
}

impl Default for ImageMap { fn default() -> Self { Self::new() } }

/// Standard ray-casting point-in-polygon, works for convex and concave polygons.
fn point_in_polygon(x: i32, y: i32, points: &[(i32, i32)]) -> bool {
    if points.len() < 3 { return false; }
    let mut inside = false;
    let n = points.len();
    let mut j = n - 1;
    for i in 0..n {
        let (xi, yi) = points[i];
        let (xj, yj) = points[j];
        // Ray from (x, y) horizontally to the right — does edge (i,j) cross it?
        if (yi > y) != (yj > y) {
            let x_intersect = (xj - xi) as i64 * (y - yi) as i64 / (yj - yi) as i64 + xi as i64;
            if (x as i64) < x_intersect {
                inside = !inside;
            }
        }
        j = i;
    }
    inside
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rect_contains() {
        let s = Shape::Rect { x: 10, y: 20, w: 30, h: 40 };
        assert!(s.contains(15, 25));
        assert!(s.contains(10, 20));     // inclusive top-left
        assert!(!s.contains(40, 60));    // exclusive bottom-right
        assert!(!s.contains(9, 25));
    }

    #[test]
    fn circle_contains() {
        let s = Shape::Circle { cx: 50, cy: 50, r: 10 };
        assert!(s.contains(50, 50));
        assert!(s.contains(55, 50));
        assert!(s.contains(50, 60));     // boundary
        assert!(!s.contains(61, 50));
    }

    #[test]
    fn polygon_triangle_contains() {
        let s = Shape::Polygon { points: vec![(0, 0), (10, 0), (5, 10)] };
        assert!(s.contains(5, 2));
        assert!(s.contains(5, 5));
        assert!(!s.contains(0, 10));
        assert!(!s.contains(15, 5));
    }

    #[test]
    fn polygon_concave_works() {
        // Arrow-like L shape: concave polygon.
        let s = Shape::Polygon { points: vec![
            (0, 0), (10, 0), (10, 5), (5, 5), (5, 10), (0, 10),
        ] };
        assert!(s.contains(2, 2));    // inside L
        assert!(s.contains(8, 2));    // inside L
        assert!(!s.contains(8, 8));   // in the concave notch, outside
    }

    #[test]
    fn bbox_rect() {
        let s = Shape::Rect { x: 10, y: 20, w: 30, h: 40 };
        assert_eq!(s.bbox(), (10, 20, 30, 40));
    }

    #[test]
    fn bbox_circle() {
        let s = Shape::Circle { cx: 50, cy: 50, r: 10 };
        assert_eq!(s.bbox(), (40, 40, 20, 20));
    }

    #[test]
    fn bbox_polygon() {
        let s = Shape::Polygon { points: vec![(10, 20), (50, 30), (30, 80)] };
        assert_eq!(s.bbox(), (10, 20, 40, 60));
    }

    #[test]
    fn hit_test_returns_topmost() {
        let mut m = ImageMap::new();
        m.add(Region { id: 1, shape: Shape::Rect { x: 0, y: 0, w: 100, h: 100 },
                       action: "bottom".into(), alt: "b".into(), z: 0 });
        m.add(Region { id: 2, shape: Shape::Rect { x: 0, y: 0, w: 50, h: 50 },
                       action: "top".into(), alt: "t".into(), z: 1 });
        let hit = m.hit_test(10, 10).unwrap();
        assert_eq!(hit.id, 2);
    }

    #[test]
    fn hit_test_miss_returns_none() {
        let mut m = ImageMap::new();
        m.add(Region { id: 1, shape: Shape::Circle { cx: 50, cy: 50, r: 5 },
                       action: "".into(), alt: "".into(), z: 0 });
        assert!(m.hit_test(100, 100).is_none());
    }

    #[test]
    fn hit_test_all_returns_overlapping() {
        let mut m = ImageMap::new();
        m.add(Region { id: 1, shape: Shape::Rect { x: 0, y: 0, w: 100, h: 100 },
                       action: "a".into(), alt: "a".into(), z: 0 });
        m.add(Region { id: 2, shape: Shape::Rect { x: 0, y: 0, w: 50, h: 50 },
                       action: "b".into(), alt: "b".into(), z: 1 });
        let all = m.hit_test_all(10, 10);
        assert_eq!(all.len(), 2);
        assert_eq!(all[0].id, 2);  // topmost first
        assert_eq!(all[1].id, 1);
    }

    #[test]
    fn to_html_emits_rect_area() {
        let mut m = ImageMap::new();
        m.add(Region { id: 1, shape: Shape::Rect { x: 10, y: 20, w: 30, h: 40 },
                       action: "/home".into(), alt: "Home".into(), z: 0 });
        let html = m.to_html("nav");
        assert!(html.contains("<map name=\"nav\">"));
        assert!(html.contains("shape=\"rect\""));
        assert!(html.contains("coords=\"10,20,40,60\""));
        assert!(html.contains("href=\"/home\""));
    }

    #[test]
    fn to_html_emits_circle_and_poly() {
        let mut m = ImageMap::new();
        m.add(Region { id: 1, shape: Shape::Circle { cx: 50, cy: 50, r: 10 },
                       action: "/c".into(), alt: "circle".into(), z: 0 });
        m.add(Region { id: 2, shape: Shape::Polygon { points: vec![(0,0), (10,0), (5,10)] },
                       action: "/p".into(), alt: "poly".into(), z: 0 });
        let html = m.to_html("x");
        assert!(html.contains("shape=\"circle\""));
        assert!(html.contains("coords=\"50,50,10\""));
        assert!(html.contains("shape=\"poly\""));
        assert!(html.contains("coords=\"0,0,10,0,5,10\""));
    }

    #[test]
    fn empty_polygon_does_not_contain() {
        let s = Shape::Polygon { points: vec![] };
        assert!(!s.contains(0, 0));
    }
}
