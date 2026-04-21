// Screen Capture Program — region + delay FSM + annotations + ring history.
package graphics

import "fmt"

type RegionKind int

const (
	RegionFullScreen RegionKind = iota
	RegionRect
	RegionWindow
)

type CaptureRegion struct {
	Kind     RegionKind
	X, Y     uint32
	Width    uint32
	Height   uint32
	WindowID uint32
}

func RegionFull() CaptureRegion { return CaptureRegion{Kind: RegionFullScreen} }
func RegionRectNew(x, y, w, h uint32) CaptureRegion {
	return CaptureRegion{Kind: RegionRect, X: x, Y: y, Width: w, Height: h}
}
func RegionWindowNew(id uint32) CaptureRegion {
	return CaptureRegion{Kind: RegionWindow, WindowID: id}
}

type AnnotationKind int

const (
	AnnRectangle AnnotationKind = iota
	AnnHighlight
	AnnText
	AnnArrow
)

type Annotation struct {
	Kind       AnnotationKind
	X, Y       uint32
	Width      uint32
	Height     uint32
	X1, Y1     uint32
	X2, Y2     uint32
	Color      GSRgb
	Stroke     uint32
	Opacity    uint8
	Text       string
}

func AnnRect(x, y, w, h uint32, color GSRgb, stroke uint32) Annotation {
	return Annotation{Kind: AnnRectangle, X: x, Y: y, Width: w, Height: h, Color: color, Stroke: stroke}
}
func AnnHighlightNew(x, y, w, h uint32, color GSRgb, opacity uint8) Annotation {
	return Annotation{Kind: AnnHighlight, X: x, Y: y, Width: w, Height: h, Color: color, Opacity: opacity}
}
func AnnArrowNew(x1, y1, x2, y2 uint32, color GSRgb) Annotation {
	return Annotation{Kind: AnnArrow, X1: x1, Y1: y1, X2: x2, Y2: y2, Color: color}
}

type CaptureSpec struct {
	Region        CaptureRegion
	DelayMs       uint64
	IncludeCursor bool
	Annotations   []Annotation
}

type RecorderState int

const (
	RecIdle RecorderState = iota
	RecArmed
	RecCounting
	RecCapturing
	RecCaptured
)

type Capture struct {
	ID            uint64
	Spec          CaptureSpec
	Image         *GSImage
	CapturedAtMs  uint64
}

type RecorderEventKind int

const (
	RecEventArmed RecorderEventKind = iota
	RecEventCountdownTick
	RecEventCaptured
	RecEventEvicted
	RecEventCaptureFailed
)

type RecorderEvent struct {
	Kind   RecorderEventKind
	ID     uint64
	Detail string
}

type Recorder struct {
	State                RecorderState
	PendingSpec          *CaptureSpec
	PendingID            uint64
	CountdownRemainingMs uint64
	History              []*Capture
	HistoryMax           int
	ElapsedMs            uint64
	Events               []RecorderEvent
	nextID               uint64
}

func NewRecorder(historyMax int) *Recorder {
	return &Recorder{
		State: RecIdle, HistoryMax: historyMax, nextID: 1,
	}
}

func (r *Recorder) Arm(spec CaptureSpec) uint64 {
	id := r.nextID
	r.nextID++
	r.PendingID = id
	r.CountdownRemainingMs = spec.DelayMs
	specCopy := spec
	r.PendingSpec = &specCopy
	r.Events = append(r.Events, RecorderEvent{
		Kind: RecEventArmed, ID: id,
		Detail: fmt.Sprintf("delay=%dms", spec.DelayMs),
	})
	if spec.DelayMs == 0 {
		r.State = RecCapturing
	} else {
		r.State = RecCounting
	}
	return id
}

func (r *Recorder) Tick(ms uint64) {
	r.ElapsedMs += ms
	if r.State == RecCounting {
		if r.CountdownRemainingMs > ms {
			r.CountdownRemainingMs -= ms
		} else {
			r.CountdownRemainingMs = 0
			r.State = RecCapturing
		}
		r.Events = append(r.Events, RecorderEvent{
			Kind: RecEventCountdownTick, ID: r.PendingID,
			Detail: fmt.Sprintf("remaining=%dms", r.CountdownRemainingMs),
		})
	}
}

func (r *Recorder) Capture(screen *GSImage, windowLookup func(uint32) *GSImage) *Capture {
	if r.State != RecCapturing || r.PendingSpec == nil {
		return nil
	}
	spec := *r.PendingSpec
	r.PendingSpec = nil
	id := r.PendingID
	region := spec.Region

	var base *GSImage
	switch region.Kind {
	case RegionFullScreen:
		pixels := make([]GSRgb, len(screen.Pixels))
		copy(pixels, screen.Pixels)
		base = &GSImage{Width: screen.Width, Height: screen.Height, Pixels: pixels}
	case RegionRect:
		if region.X+region.Width > screen.Width || region.Y+region.Height > screen.Height {
			r.Events = append(r.Events, RecorderEvent{
				Kind: RecEventCaptureFailed, ID: id, Detail: "rect out of bounds",
			})
			r.State = RecIdle
			return nil
		}
		base = cropScreen(screen, region.X, region.Y, region.Width, region.Height)
	case RegionWindow:
		base = windowLookup(region.WindowID)
		if base == nil {
			r.Events = append(r.Events, RecorderEvent{
				Kind: RecEventCaptureFailed, ID: id,
				Detail: fmt.Sprintf("window %d not found", region.WindowID),
			})
			r.State = RecIdle
			return nil
		}
	}

	annotated := ApplyAnnotations(base, spec.Annotations)
	cap := &Capture{ID: id, Spec: spec, Image: annotated, CapturedAtMs: r.ElapsedMs}
	r.History = append(r.History, cap)
	if len(r.History) > r.HistoryMax {
		evicted := r.History[0]
		r.History = r.History[1:]
		r.Events = append(r.Events, RecorderEvent{Kind: RecEventEvicted, ID: evicted.ID})
	}
	r.Events = append(r.Events, RecorderEvent{
		Kind: RecEventCaptured, ID: id,
		Detail: fmt.Sprintf("at=%dms", r.ElapsedMs),
	})
	r.State = RecCaptured
	return cap
}

func (r *Recorder) Reset() {
	r.State = RecIdle
	r.PendingSpec = nil
	r.PendingID = 0
	r.CountdownRemainingMs = 0
}

func cropScreen(img *GSImage, x0, y0, w, h uint32) *GSImage {
	pixels := make([]GSRgb, int(w)*int(h))
	for y := uint32(0); y < h; y++ {
		for x := uint32(0); x < w; x++ {
			pixels[y*w+x] = img.Pixels[(y0+y)*img.Width+(x0+x)]
		}
	}
	return &GSImage{Width: w, Height: h, Pixels: pixels}
}

func ApplyAnnotations(base *GSImage, anns []Annotation) *GSImage {
	pixels := make([]GSRgb, len(base.Pixels))
	copy(pixels, base.Pixels)
	img := &GSImage{Width: base.Width, Height: base.Height, Pixels: pixels}
	for _, a := range anns {
		applyOneAnnotation(img, a)
	}
	return img
}

func applyOneAnnotation(img *GSImage, a Annotation) {
	switch a.Kind {
	case AnnRectangle:
		drawRectOutline(img, a.X, a.Y, a.Width, a.Height, a.Color, maxU32(1, a.Stroke))
	case AnnHighlight:
		blendRectSc(img, a.X, a.Y, a.Width, a.Height, a.Color, a.Opacity)
	case AnnText:
		setClamped(img, a.X, a.Y, a.Color)
	case AnnArrow:
		drawLineBresenham(img, a.X1, a.Y1, a.X2, a.Y2, a.Color)
	}
}

func maxU32(a, b uint32) uint32 {
	if a > b {
		return a
	}
	return b
}

func setClamped(img *GSImage, x, y uint32, color GSRgb) {
	if x < img.Width && y < img.Height {
		img.Pixels[y*img.Width+x] = color
	}
}

func drawRectOutline(img *GSImage, x, y, w, h uint32, color GSRgb, stroke uint32) {
	for s := uint32(0); s < stroke; s++ {
		for dx := uint32(0); dx < w; dx++ {
			setClamped(img, x+dx, y+s, color)
			if h > s {
				setClamped(img, x+dx, y+h-1-s, color)
			}
		}
		for dy := uint32(0); dy < h; dy++ {
			setClamped(img, x+s, y+dy, color)
			if w > s {
				setClamped(img, x+w-1-s, y+dy, color)
			}
		}
	}
}

func blendChannel(a, b, alpha uint8) uint8 {
	ai := uint32(a)
	bi := uint32(b)
	al := uint32(alpha)
	return uint8((ai*(255-al) + bi*al) / 255)
}

func blendRectSc(img *GSImage, x, y, w, h uint32, color GSRgb, opacity uint8) {
	for dy := uint32(0); dy < h; dy++ {
		for dx := uint32(0); dx < w; dx++ {
			px := x + dx
			py := y + dy
			if px >= img.Width || py >= img.Height {
				continue
			}
			cur := img.Pixels[py*img.Width+px]
			img.Pixels[py*img.Width+px] = GSRgb{
				R: blendChannel(cur.R, color.R, opacity),
				G: blendChannel(cur.G, color.G, opacity),
				B: blendChannel(cur.B, color.B, opacity),
			}
		}
	}
}

func drawLineBresenham(img *GSImage, x1, y1, x2, y2 uint32, color GSRgb) {
	x := int32(x1)
	y := int32(y1)
	x2i := int32(x2)
	y2i := int32(y2)
	dx := abs32(x2i - x)
	dy := -abs32(y2i - y)
	sx := int32(1)
	if x >= x2i {
		sx = -1
	}
	sy := int32(1)
	if y >= y2i {
		sy = -1
	}
	err := dx + dy
	for {
		if x >= 0 && y >= 0 {
			setClamped(img, uint32(x), uint32(y), color)
		}
		if x == x2i && y == y2i {
			break
		}
		e2 := 2 * err
		if e2 >= dy {
			err += dy
			x += sx
		}
		if e2 <= dx {
			err += dx
			y += sy
		}
	}
}

func abs32(x int32) int32 {
	if x < 0 {
		return -x
	}
	return x
}

func ScreenCaptureDemo() {
	rec := NewRecorder(3)
	rec.Arm(CaptureSpec{Region: RegionFull(), DelayMs: 1000})
	fmt.Printf("state: %d remaining: %dms\n", rec.State, rec.CountdownRemainingMs)
	rec.Tick(400)
	fmt.Printf("after 400ms tick: state: %d remaining: %dms\n", rec.State, rec.CountdownRemainingMs)
	rec.Tick(700)
	fmt.Printf("after 700ms tick: state: %d remaining: %dms\n", rec.State, rec.CountdownRemainingMs)

	screen := GSImageFilled(10, 8, GSRgb{100, 100, 100})
	cap := rec.Capture(screen, func(uint32) *GSImage { return nil })
	fmt.Printf("captured: id=%d image=%dx%d\n", cap.ID, cap.Image.Width, cap.Image.Height)

	rec.Reset()
	rec.Arm(CaptureSpec{
		Region: RegionRectNew(2, 1, 6, 5), DelayMs: 0,
		Annotations: []Annotation{
			AnnHighlightNew(0, 0, 6, 5, GSRgb{255, 255, 0}, 128),
			AnnRect(1, 1, 4, 3, GSRgb{255, 0, 0}, 1),
			AnnArrowNew(0, 0, 5, 4, GSRgb{0, 255, 0}),
		},
	})
	cap2 := rec.Capture(screen, func(uint32) *GSImage { return nil })
	fmt.Printf("annotated: %dx%d\n", cap2.Image.Width, cap2.Image.Height)
	p := cap2.Image.Pixels[0]
	fmt.Printf("pixels[0]=(%d,%d,%d)\n", p.R, p.G, p.B)

	for i := 0; i < 5; i++ {
		rec.Reset()
		rec.Arm(CaptureSpec{Region: RegionFull(), DelayMs: 0})
		rec.Capture(screen, func(uint32) *GSImage { return nil })
	}
	fmt.Printf("history size: %d\n", len(rec.History))
	ids := []uint64{}
	for _, c := range rec.History {
		ids = append(ids, c.ID)
	}
	fmt.Printf("history ids: %v\n", ids)
}
