// Online White Board — shared canvas of strokes with spatial queries.
package web

import (
	"fmt"
	"math"
	"strings"
)

// WBPoint is a 2D coordinate on the whiteboard.
type WBPoint struct{ X, Y float64 }

// Distance returns the Euclidean distance from p to q.
func (p WBPoint) Distance(q WBPoint) float64 {
	return math.Hypot(p.X-q.X, p.Y-q.Y)
}

// WBStroke is one freehand stroke by one author.
type WBStroke struct {
	ID        int64
	Author    string
	Color     string
	Thickness float64
	Points    []WBPoint
}

// BBox returns the stroke's bounding box, or (zero, zero, false) if empty.
func (s *WBStroke) BBox() (lo, hi WBPoint, ok bool) {
	if len(s.Points) == 0 {
		return
	}
	lo, hi = s.Points[0], s.Points[0]
	for _, p := range s.Points[1:] {
		if p.X < lo.X {
			lo.X = p.X
		}
		if p.Y < lo.Y {
			lo.Y = p.Y
		}
		if p.X > hi.X {
			hi.X = p.X
		}
		if p.Y > hi.Y {
			hi.Y = p.Y
		}
	}
	return lo, hi, true
}

// DistanceTo returns min distance from any point of the stroke to p.
func (s *WBStroke) DistanceTo(p WBPoint) float64 {
	if len(s.Points) == 0 {
		return math.Inf(1)
	}
	best := math.Inf(1)
	for _, q := range s.Points {
		if d := q.Distance(p); d < best {
			best = d
		}
	}
	return best
}

// Whiteboard is a shared canvas of strokes in insertion-order z-order.
type Whiteboard struct {
	strokes []*WBStroke
	nextID  int64
}

// NewWhiteboard returns an empty whiteboard.
func NewWhiteboard() *Whiteboard { return &Whiteboard{nextID: 1} }

// AddStroke appends a stroke and returns its id.
func (b *Whiteboard) AddStroke(author, color string, thickness float64,
	points []WBPoint) int64 {
	id := b.nextID
	b.nextID++
	b.strokes = append(b.strokes, &WBStroke{
		ID: id, Author: author, Color: color,
		Thickness: thickness, Points: append([]WBPoint{}, points...),
	})
	return id
}

// RemoveStroke removes the stroke with id; returns whether it was present.
func (b *Whiteboard) RemoveStroke(id int64) bool {
	for i, s := range b.strokes {
		if s.ID == id {
			b.strokes = append(b.strokes[:i], b.strokes[i+1:]...)
			return true
		}
	}
	return false
}

// Clear removes all strokes.
func (b *Whiteboard) Clear() { b.strokes = nil }

// StrokeCount returns the number of strokes on the board.
func (b *Whiteboard) StrokeCount() int { return len(b.strokes) }

// Strokes returns a slice view of strokes in z-order.
func (b *Whiteboard) Strokes() []*WBStroke { return b.strokes }

// StrokesByAuthor returns strokes authored by the given name.
func (b *Whiteboard) StrokesByAuthor(author string) []*WBStroke {
	var out []*WBStroke
	for _, s := range b.strokes {
		if s.Author == author {
			out = append(out, s)
		}
	}
	return out
}

// StrokesNear returns strokes within radius of point.
func (b *Whiteboard) StrokesNear(p WBPoint, radius float64) []*WBStroke {
	var out []*WBStroke
	for _, s := range b.strokes {
		if s.DistanceTo(p) <= radius {
			out = append(out, s)
		}
	}
	return out
}

// BoundingBox returns the enclosing box of every stroke, or !ok if empty.
func (b *Whiteboard) BoundingBox() (lo, hi WBPoint, ok bool) {
	for _, s := range b.strokes {
		slo, shi, sok := s.BBox()
		if !sok {
			continue
		}
		if !ok {
			lo, hi = slo, shi
			ok = true
			continue
		}
		if slo.X < lo.X {
			lo.X = slo.X
		}
		if slo.Y < lo.Y {
			lo.Y = slo.Y
		}
		if shi.X > hi.X {
			hi.X = shi.X
		}
		if shi.Y > hi.Y {
			hi.Y = shi.Y
		}
	}
	return
}

// ToSVG renders the board as SVG at the given canvas dimensions.
func (b *Whiteboard) ToSVG(width, height float64) string {
	var sb strings.Builder
	fmt.Fprintf(&sb, `<svg xmlns="http://www.w3.org/2000/svg" width="%v" height="%v">`+"\n",
		width, height)
	for _, s := range b.strokes {
		if len(s.Points) == 0 {
			continue
		}
		pts := make([]string, 0, len(s.Points))
		for _, p := range s.Points {
			pts = append(pts, fmt.Sprintf("%v,%v", p.X, p.Y))
		}
		fmt.Fprintf(&sb, `  <polyline fill="none" stroke="%s" stroke-width="%v" points="%s"/>`+"\n",
			s.Color, s.Thickness, strings.Join(pts, " "))
	}
	sb.WriteString("</svg>")
	return sb.String()
}
