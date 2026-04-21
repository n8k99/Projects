// Signature Maker — stroke model + Catmull-Rom + variable-width + normalize.
package graphics

import "fmt"

type SigPoint struct {
	X, Y     int32
	Pressure uint16 // 0..1024
	TMs      uint64
}

type Stroke struct {
	Points []SigPoint
}

type Signature struct {
	Strokes []Stroke
}

func NewSignature() *Signature { return &Signature{} }

func (s *Signature) AddStroke(st Stroke) {
	s.Strokes = append(s.Strokes, st)
}

func (s *Signature) PointCount() int {
	n := 0
	for _, st := range s.Strokes {
		n += len(st.Points)
	}
	return n
}

func (s *Signature) BoundingBox() (minX, minY, maxX, maxY int32, ok bool) {
	minX = 1 << 30
	minY = 1 << 30
	maxX = -(1 << 30)
	maxY = -(1 << 30)
	for _, st := range s.Strokes {
		for _, p := range st.Points {
			if p.X < minX { minX = p.X }
			if p.Y < minY { minY = p.Y }
			if p.X > maxX { maxX = p.X }
			if p.Y > maxY { maxY = p.Y }
			ok = true
		}
	}
	return
}

func crCompute(a, b, c, d, tt, t2, t3 int64) int64 {
	termA := 2 * b
	termB := (-a + c) * tt
	termC := (2*a - 5*b + 4*c - d) * t2
	termD := (-a + 3*b - 3*c + d) * t3
	sum := termA*1_000_000 + termB*1_000 + termC + termD/1_000
	return sum / 2_000_000
}

func CatmullRom(p0, p1, p2, p3 [2]int64, tt int64) (int64, int64) {
	t2 := tt * tt
	t3 := t2 * tt / 1000
	x := crCompute(p0[0], p1[0], p2[0], p3[0], tt, t2, t3)
	y := crCompute(p0[1], p1[1], p2[1], p3[1], tt, t2, t3)
	return x, y
}

func SmoothStroke(st Stroke, steps uint32) Stroke {
	pts := st.Points
	if len(pts) < 4 || steps == 0 {
		out := make([]SigPoint, len(pts))
		copy(out, pts)
		return Stroke{Points: out}
	}
	out := []SigPoint{pts[0]}
	for i := 0; i < len(pts)-3; i++ {
		p0 := [2]int64{int64(pts[i].X), int64(pts[i].Y)}
		p1 := [2]int64{int64(pts[i+1].X), int64(pts[i+1].Y)}
		p2 := [2]int64{int64(pts[i+2].X), int64(pts[i+2].Y)}
		p3 := [2]int64{int64(pts[i+3].X), int64(pts[i+3].Y)}
		for s := uint32(1); s <= steps; s++ {
			tt := int64(s) * 1000 / int64(steps)
			x, y := CatmullRom(p0, p1, p2, p3, tt)
			pressure := int64(pts[i+1].Pressure) +
				(int64(pts[i+2].Pressure)-int64(pts[i+1].Pressure))*tt/1000
			if pressure < 0 { pressure = 0 }
			if pressure > 1024 { pressure = 1024 }
			tms := int64(pts[i+1].TMs) +
				(int64(pts[i+2].TMs)-int64(pts[i+1].TMs))*tt/1000
			if tms < 0 { tms = 0 }
			out = append(out, SigPoint{
				X: int32(x), Y: int32(y),
				Pressure: uint16(pressure), TMs: uint64(tms),
			})
		}
	}
	out = append(out, pts[len(pts)-1])
	return Stroke{Points: out}
}

func NormalizeSignature(sig *Signature, targetW, targetH int32) *Signature {
	minX, minY, maxX, maxY, ok := sig.BoundingBox()
	if !ok {
		out := NewSignature()
		for _, s := range sig.Strokes {
			pts := make([]SigPoint, len(s.Points))
			copy(pts, s.Points)
			out.AddStroke(Stroke{Points: pts})
		}
		return out
	}
	w := maxX - minX
	if w < 1 { w = 1 }
	h := maxY - minY
	if h < 1 { h = 1 }
	scaleX := int64(targetW) * 1000 / int64(w)
	scaleY := int64(targetH) * 1000 / int64(h)
	out := NewSignature()
	for _, s := range sig.Strokes {
		pts := make([]SigPoint, 0, len(s.Points))
		for _, p := range s.Points {
			nx := int64(p.X-minX) * scaleX / 1000
			ny := int64(p.Y-minY) * scaleY / 1000
			pts = append(pts, SigPoint{
				X: int32(nx), Y: int32(ny),
				Pressure: p.Pressure, TMs: p.TMs,
			})
		}
		out.AddStroke(Stroke{Points: pts})
	}
	return out
}

func SimilarityDistance(a, b *Signature) uint64 {
	var total uint64
	for _, sa := range a.Strokes {
		for _, p := range sa.Points {
			var minD2 uint64 = ^uint64(0)
			for _, sb := range b.Strokes {
				for _, q := range sb.Points {
					dx := int64(p.X - q.X)
					dy := int64(p.Y - q.Y)
					d2 := uint64(dx*dx + dy*dy)
					if d2 < minD2 { minD2 = d2 }
				}
			}
			total += minD2
		}
	}
	return total
}

func SignatureMakerDemo() {
	sig := NewSignature()
	sig.AddStroke(Stroke{Points: []SigPoint{
		{X: 10, Y: 20, Pressure: 512, TMs: 0},
		{X: 30, Y: 40, Pressure: 768, TMs: 100},
		{X: 50, Y: 30, Pressure: 1024, TMs: 200},
		{X: 70, Y: 50, Pressure: 512, TMs: 300},
	}})
	sig.AddStroke(Stroke{Points: []SigPoint{
		{X: 80, Y: 30, Pressure: 600, TMs: 400},
		{X: 100, Y: 50, Pressure: 700, TMs: 500},
	}})

	fmt.Printf("point count: %d\n", sig.PointCount())
	minX, minY, maxX, maxY, _ := sig.BoundingBox()
	fmt.Printf("bbox: (%d %d %d %d)\n", minX, minY, maxX, maxY)

	smoothed := SmoothStroke(sig.Strokes[0], 5)
	fmt.Printf("smoothed points: %d\n", len(smoothed.Points))

	norm := NormalizeSignature(sig, 1000, 1000)
	nminX, nminY, nmaxX, nmaxY, _ := norm.BoundingBox()
	fmt.Printf("normalized bbox: (%d %d %d %d)\n", nminX, nminY, nmaxX, nmaxY)

	x0, y0 := CatmullRom([2]int64{0, 0}, [2]int64{10, 10}, [2]int64{20, 10}, [2]int64{30, 0}, 0)
	x1, y1 := CatmullRom([2]int64{0, 0}, [2]int64{10, 10}, [2]int64{20, 10}, [2]int64{30, 0}, 1000)
	fmt.Printf("cr t=0: (%d, %d)\n", x0, y0)
	fmt.Printf("cr t=1000: (%d, %d)\n", x1, y1)

	fmt.Printf("self-similarity: %d\n", SimilarityDistance(sig, sig))

	small := NewSignature()
	small.AddStroke(Stroke{Points: []SigPoint{
		{0, 0, 500, 0}, {10, 0, 500, 100}, {10, 10, 500, 200}, {0, 10, 500, 300},
	}})
	big := NewSignature()
	big.AddStroke(Stroke{Points: []SigPoint{
		{0, 0, 500, 0}, {100, 0, 500, 100}, {100, 100, 500, 200}, {0, 100, 500, 300},
	}})
	nS := NormalizeSignature(small, 1000, 1000)
	nB := NormalizeSignature(big, 1000, 1000)
	fmt.Printf("normalized same-shape similarity: %d\n", SimilarityDistance(nS, nB))
}
