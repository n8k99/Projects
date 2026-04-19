//! Shape Area and Perimeter — one trait, many implementations.
//!
//! Rust dispatches on traits. `impl Shape for Circle` makes circle values
//! usable wherever `dyn Shape` is expected. Heterogeneous collections are
//! `Vec<Box<dyn Shape>>` — the values live on the heap, the trait pointer
//! carries the vtable. This is **open polymorphism**: a new shape is added
//! by writing `impl Shape for NewShape`, nowhere else.

use std::f64::consts::PI;

pub trait Shape {
    fn area(&self) -> f64;
    fn perimeter(&self) -> f64;
    fn name(&self) -> String;
}

#[derive(Debug, Clone)]
pub struct Circle {
    pub radius: f64,
}

impl Circle {
    pub fn new(radius: f64) -> Result<Self, String> {
        if radius < 0.0 { return Err(format!("negative radius: {radius}")); }
        Ok(Self { radius })
    }
}

impl Shape for Circle {
    fn area(&self) -> f64 { PI * self.radius * self.radius }
    fn perimeter(&self) -> f64 { 2.0 * PI * self.radius }
    fn name(&self) -> String { "Circle".to_string() }
}

#[derive(Debug, Clone)]
pub struct Rectangle {
    pub width: f64,
    pub height: f64,
}

impl Rectangle {
    pub fn new(width: f64, height: f64) -> Result<Self, String> {
        if width < 0.0 || height < 0.0 {
            return Err(format!("negative dimension: {width}x{height}"));
        }
        Ok(Self { width, height })
    }
}

impl Shape for Rectangle {
    fn area(&self) -> f64 { self.width * self.height }
    fn perimeter(&self) -> f64 { 2.0 * (self.width + self.height) }
    fn name(&self) -> String {
        if self.width == self.height { "Square".into() } else { "Rectangle".into() }
    }
}

#[derive(Debug, Clone)]
pub struct Triangle {
    pub a: f64,
    pub b: f64,
    pub c: f64,
}

impl Triangle {
    pub fn new(a: f64, b: f64, c: f64) -> Result<Self, String> {
        if a <= 0.0 || b <= 0.0 || c <= 0.0 {
            return Err(format!("non-positive side: {a} {b} {c}"));
        }
        if !(a + b > c && a + c > b && b + c > a) {
            return Err(format!("triangle inequality violated: {a} {b} {c}"));
        }
        Ok(Self { a, b, c })
    }
}

impl Shape for Triangle {
    fn area(&self) -> f64 {
        let s = (self.a + self.b + self.c) / 2.0;
        (s * (s - self.a) * (s - self.b) * (s - self.c)).sqrt()
    }
    fn perimeter(&self) -> f64 { self.a + self.b + self.c }
    fn name(&self) -> String { "Triangle".into() }
}

#[derive(Debug, Clone)]
pub struct RegularPolygon {
    pub sides: u32,
    pub side_length: f64,
}

impl RegularPolygon {
    pub fn new(sides: u32, side_length: f64) -> Result<Self, String> {
        if sides < 3 { return Err(format!("polygon needs >= 3 sides, got {sides}")); }
        if side_length < 0.0 { return Err(format!("negative side: {side_length}")); }
        Ok(Self { sides, side_length })
    }
}

impl Shape for RegularPolygon {
    fn area(&self) -> f64 {
        let n = self.sides as f64;
        let s = self.side_length;
        n * s * s / (4.0 * (PI / n).tan())
    }
    fn perimeter(&self) -> f64 { self.sides as f64 * self.side_length }
    fn name(&self) -> String { format!("Regular-{}-gon", self.sides) }
}

// --- heterogeneous collection operations ---

pub fn total_area(shapes: &[Box<dyn Shape>]) -> f64 {
    shapes.iter().map(|s| s.area()).sum()
}

pub fn total_perimeter(shapes: &[Box<dyn Shape>]) -> f64 {
    shapes.iter().map(|s| s.perimeter()).sum()
}

pub fn largest_by_area(shapes: &[Box<dyn Shape>]) -> Option<&Box<dyn Shape>> {
    shapes.iter().max_by(|a, b| {
        a.area().partial_cmp(&b.area()).unwrap_or(std::cmp::Ordering::Equal)
    })
}

pub fn sort_by_perimeter(shapes: Vec<Box<dyn Shape>>) -> Vec<Box<dyn Shape>> {
    let mut s = shapes;
    s.sort_by(|a, b| {
        a.perimeter().partial_cmp(&b.perimeter()).unwrap_or(std::cmp::Ordering::Equal)
    });
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn circle_area_matches_pi_r_squared() {
        let c = Circle::new(3.0).unwrap();
        assert!((c.area() - PI * 9.0).abs() < 1e-9);
        assert!((c.perimeter() - 2.0 * PI * 3.0).abs() < 1e-9);
    }

    #[test]
    fn rectangle_names_itself_square_when_sides_equal() {
        let r = Rectangle::new(4.0, 4.0).unwrap();
        assert_eq!(r.name(), "Square");
        let r2 = Rectangle::new(4.0, 5.0).unwrap();
        assert_eq!(r2.name(), "Rectangle");
    }

    #[test]
    fn right_triangle_area_by_heron() {
        let t = Triangle::new(3.0, 4.0, 5.0).unwrap();
        assert!((t.area() - 6.0).abs() < 1e-9);
        assert!((t.perimeter() - 12.0).abs() < 1e-9);
    }

    #[test]
    fn triangle_inequality_rejected() {
        assert!(Triangle::new(1.0, 2.0, 10.0).is_err());
    }

    #[test]
    fn regular_polygon_rejects_fewer_than_three_sides() {
        assert!(RegularPolygon::new(2, 1.0).is_err());
        assert!(RegularPolygon::new(3, 1.0).is_ok());
    }

    #[test]
    fn heterogeneous_collection_works_through_trait() {
        let shapes: Vec<Box<dyn Shape>> = vec![
            Box::new(Circle::new(1.0).unwrap()),
            Box::new(Rectangle::new(2.0, 3.0).unwrap()),
            Box::new(Triangle::new(3.0, 4.0, 5.0).unwrap()),
        ];
        // Circle: π, Rectangle: 6, Triangle: 6 — the rectangle or triangle is largest.
        let biggest = largest_by_area(&shapes).unwrap();
        assert!(matches!(biggest.name().as_str(), "Rectangle" | "Triangle"));
        assert_eq!(shapes.len(), 3);
    }
}
