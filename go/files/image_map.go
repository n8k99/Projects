// Image Map Generator — clickable regions on an image with hit testing.
package files

import (
	"fmt"
	"sort"
	"strings"
)

// ShapeKind tags a hit-testable shape.
type ShapeKind int

const (
	ShRect    ShapeKind = 0
	ShCircle  ShapeKind = 1
	ShPolygon ShapeKind = 2
)

// ImShape is a region's geometry.
type ImShape struct {
	Kind   ShapeKind
	X, Y   int  // rect origin
	W, H   int  // rect size
	CX, CY int  // circle centre
	R      int  // circle radius
	Points [][2]int
}

// ImRect constructs a rectangle shape.
func ImRect(x, y, w, h int) ImShape { return ImShape{Kind: ShRect, X: x, Y: y, W: w, H: h} }

// ImCircle constructs a circle shape.
func ImCircle(cx, cy, r int) ImShape { return ImShape{Kind: ShCircle, CX: cx, CY: cy, R: r} }

// ImPolygon constructs a polygon shape.
func ImPolygon(points [][2]int) ImShape { return ImShape{Kind: ShPolygon, Points: points} }

// Contains returns true if (x, y) is inside the shape.
func (s ImShape) Contains(x, y int) bool {
	switch s.Kind {
	case ShRect:
		return x >= s.X && x < s.X+s.W && y >= s.Y && y < s.Y+s.H
	case ShCircle:
		dx, dy := x-s.CX, y-s.CY
		return dx*dx+dy*dy <= s.R*s.R
	case ShPolygon:
		return pointInPolygon(x, y, s.Points)
	}
	return false
}

// BBox returns (x, y, w, h).
func (s ImShape) BBox() (int, int, int, int) {
	switch s.Kind {
	case ShRect:
		return s.X, s.Y, s.W, s.H
	case ShCircle:
		return s.CX - s.R, s.CY - s.R, 2 * s.R, 2 * s.R
	case ShPolygon:
		if len(s.Points) == 0 {
			return 0, 0, 0, 0
		}
		minX, minY := s.Points[0][0], s.Points[0][1]
		maxX, maxY := minX, minY
		for _, p := range s.Points {
			if p[0] < minX { minX = p[0] }
			if p[0] > maxX { maxX = p[0] }
			if p[1] < minY { minY = p[1] }
			if p[1] > maxY { maxY = p[1] }
		}
		return minX, minY, maxX - minX, maxY - minY
	}
	return 0, 0, 0, 0
}

// ImRegion is a clickable shape with action metadata.
type ImRegion struct {
	ID     int
	Shape  ImShape
	Action string
	Alt    string
	Z      int
}

// ImageMap is a collection of regions.
type ImageMap struct {
	Regions []ImRegion
}

// NewImageMap creates an empty image map.
func NewImageMap() *ImageMap { return &ImageMap{} }

// Add appends a region.
func (m *ImageMap) Add(r ImRegion) {
	m.Regions = append(m.Regions, r)
}

// HitTest returns the topmost region containing (x, y), or nil.
func (m *ImageMap) HitTest(x, y int) *ImRegion {
	sorted := make([]*ImRegion, len(m.Regions))
	for i := range m.Regions {
		sorted[i] = &m.Regions[i]
	}
	sort.SliceStable(sorted, func(i, j int) bool {
		return sorted[i].Z > sorted[j].Z
	})
	for _, r := range sorted {
		if r.Shape.Contains(x, y) {
			return r
		}
	}
	return nil
}

// HitTestAll returns all regions containing (x, y), topmost first.
func (m *ImageMap) HitTestAll(x, y int) []*ImRegion {
	sorted := make([]*ImRegion, len(m.Regions))
	for i := range m.Regions {
		sorted[i] = &m.Regions[i]
	}
	sort.SliceStable(sorted, func(i, j int) bool {
		return sorted[i].Z > sorted[j].Z
	})
	var out []*ImRegion
	for _, r := range sorted {
		if r.Shape.Contains(x, y) {
			out = append(out, r)
		}
	}
	return out
}

// ToHTML emits an HTML <map> element.
func (m *ImageMap) ToHTML(name string) string {
	var b strings.Builder
	fmt.Fprintf(&b, "<map name=\"%s\">\n", name)
	for _, r := range m.Regions {
		var shapeName, coords string
		switch r.Shape.Kind {
		case ShRect:
			shapeName = "rect"
			coords = fmt.Sprintf("%d,%d,%d,%d",
				r.Shape.X, r.Shape.Y, r.Shape.X+r.Shape.W, r.Shape.Y+r.Shape.H)
		case ShCircle:
			shapeName = "circle"
			coords = fmt.Sprintf("%d,%d,%d", r.Shape.CX, r.Shape.CY, r.Shape.R)
		case ShPolygon:
			shapeName = "poly"
			parts := make([]string, 0, 2*len(r.Shape.Points))
			for _, p := range r.Shape.Points {
				parts = append(parts, fmt.Sprintf("%d,%d", p[0], p[1]))
			}
			coords = strings.Join(parts, ",")
		}
		fmt.Fprintf(&b, "  <area shape=\"%s\" coords=\"%s\" href=\"%s\" alt=\"%s\">\n",
			shapeName, coords, r.Action, r.Alt)
	}
	fmt.Fprintf(&b, "</map>\n")
	return b.String()
}

func pointInPolygon(x, y int, points [][2]int) bool {
	n := len(points)
	if n < 3 {
		return false
	}
	inside := false
	j := n - 1
	for i := 0; i < n; i++ {
		xi, yi := points[i][0], points[i][1]
		xj, yj := points[j][0], points[j][1]
		if (yi > y) != (yj > y) {
			xIntersect := (xj-xi)*(y-yi)/(yj-yi) + xi
			if x < xIntersect {
				inside = !inside
			}
		}
		j = i
	}
	return inside
}
