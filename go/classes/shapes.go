// Shape Area and Perimeter — one interface, many implementations.
//
// Go dispatches structurally. A type that defines Area(), Perimeter(), and
// Name() methods automatically satisfies the Shape interface — no "implements"
// declaration required. This is **open, structural polymorphism**: shapes
// can be added in any package without touching the interface definition.
package classes

import (
	"errors"
	"fmt"
	"math"
	"sort"
)

// Shape is satisfied by any type with these three methods.
type Shape interface {
	Area() float64
	Perimeter() float64
	Name() string
}

// Shape errors.
var (
	ErrNegativeDim       = errors.New("negative dimension")
	ErrTriangleInequality = errors.New("triangle inequality violated")
	ErrPolygonTooFewSides = errors.New("polygon needs at least 3 sides")
)

// Circle is a disk with a given radius.
type Circle struct{ Radius float64 }

// NewCircle builds a Circle after validation.
func NewCircle(radius float64) (Circle, error) {
	if radius < 0 {
		return Circle{}, fmt.Errorf("%w: radius %v", ErrNegativeDim, radius)
	}
	return Circle{Radius: radius}, nil
}

// Area implements Shape.
func (c Circle) Area() float64 { return math.Pi * c.Radius * c.Radius }

// Perimeter implements Shape.
func (c Circle) Perimeter() float64 { return 2 * math.Pi * c.Radius }

// Name implements Shape.
func (c Circle) Name() string { return "Circle" }

// Rectangle has an axis-aligned width and height.
type Rectangle struct{ Width, Height float64 }

// NewRectangle builds a Rectangle after validation.
func NewRectangle(width, height float64) (Rectangle, error) {
	if width < 0 || height < 0 {
		return Rectangle{}, fmt.Errorf("%w: %vx%v", ErrNegativeDim, width, height)
	}
	return Rectangle{Width: width, Height: height}, nil
}

// Area implements Shape.
func (r Rectangle) Area() float64 { return r.Width * r.Height }

// Perimeter implements Shape.
func (r Rectangle) Perimeter() float64 { return 2 * (r.Width + r.Height) }

// Name implements Shape, returning "Square" when sides are equal.
func (r Rectangle) Name() string {
	if r.Width == r.Height {
		return "Square"
	}
	return "Rectangle"
}

// Triangle is defined by three side lengths.
type Triangle struct{ A, B, C float64 }

// NewTriangle validates positivity and the triangle inequality.
func NewTriangle(a, b, c float64) (Triangle, error) {
	if a <= 0 || b <= 0 || c <= 0 {
		return Triangle{}, fmt.Errorf("%w: %v %v %v", ErrNegativeDim, a, b, c)
	}
	if !(a+b > c && a+c > b && b+c > a) {
		return Triangle{}, fmt.Errorf("%w: %v %v %v", ErrTriangleInequality, a, b, c)
	}
	return Triangle{A: a, B: b, C: c}, nil
}

// Area by Heron's formula.
func (t Triangle) Area() float64 {
	s := (t.A + t.B + t.C) / 2
	return math.Sqrt(s * (s - t.A) * (s - t.B) * (s - t.C))
}

// Perimeter implements Shape.
func (t Triangle) Perimeter() float64 { return t.A + t.B + t.C }

// Name implements Shape.
func (t Triangle) Name() string { return "Triangle" }

// RegularPolygon has N equal sides.
type RegularPolygon struct {
	Sides      int
	SideLength float64
}

// NewRegularPolygon validates >= 3 sides and non-negative length.
func NewRegularPolygon(sides int, sideLength float64) (RegularPolygon, error) {
	if sides < 3 {
		return RegularPolygon{}, fmt.Errorf("%w: got %d", ErrPolygonTooFewSides, sides)
	}
	if sideLength < 0 {
		return RegularPolygon{}, fmt.Errorf("%w: side %v", ErrNegativeDim, sideLength)
	}
	return RegularPolygon{Sides: sides, SideLength: sideLength}, nil
}

// Area by the standard n-side regular-polygon formula.
func (p RegularPolygon) Area() float64 {
	n := float64(p.Sides)
	s := p.SideLength
	return n * s * s / (4 * math.Tan(math.Pi/n))
}

// Perimeter implements Shape.
func (p RegularPolygon) Perimeter() float64 { return float64(p.Sides) * p.SideLength }

// Name implements Shape, including the side count.
func (p RegularPolygon) Name() string { return fmt.Sprintf("Regular-%d-gon", p.Sides) }

// --- heterogeneous collection operations ---

// TotalArea sums Area() across shapes.
func TotalArea(shapes []Shape) float64 {
	var total float64
	for _, s := range shapes {
		total += s.Area()
	}
	return total
}

// TotalPerimeter sums Perimeter() across shapes.
func TotalPerimeter(shapes []Shape) float64 {
	var total float64
	for _, s := range shapes {
		total += s.Perimeter()
	}
	return total
}

// LargestByArea returns the shape with the greatest Area().
func LargestByArea(shapes []Shape) (Shape, bool) {
	if len(shapes) == 0 {
		return nil, false
	}
	best := shapes[0]
	for _, s := range shapes[1:] {
		if s.Area() > best.Area() {
			best = s
		}
	}
	return best, true
}

// SortByPerimeter returns a copy sorted ascending by Perimeter().
func SortByPerimeter(shapes []Shape) []Shape {
	out := append([]Shape(nil), shapes...)
	sort.Slice(out, func(i, j int) bool { return out[i].Perimeter() < out[j].Perimeter() })
	return out
}
