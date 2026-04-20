// Bulk Picture Manipulator — transformation pipeline applied over a batch.
package graphics

import "fmt"

type OpKind int

const (
	OpResize OpKind = iota
	OpGrayscale
	OpRotate90
	OpRotate180
	OpRotate270
	OpFlipH
	OpFlipV
	OpCrop
	OpBrightness
	OpContrast
)

type Op struct {
	Kind            OpKind
	Width, Height   uint32
	X, Y            uint32
	Algorithm       GSAlgorithm
	BrightnessDelta int
	ContrastFactor  float64
}

type OpError struct{ Reason string }

func (e *OpError) Error() string { return e.Reason }

type Pipeline []Op

func applyOp(img *GSImage, op Op) (*GSImage, error) {
	switch op.Kind {
	case OpResize:
		if op.Width == 0 || op.Height == 0 {
			return nil, &OpError{"zero_dimension"}
		}
		return resizeNearest(img, op.Width, op.Height), nil
	case OpGrayscale:
		return grayscaleImg(img, op.Algorithm), nil
	case OpRotate90:
		return rotate90(img), nil
	case OpRotate180:
		return rotate180(img), nil
	case OpRotate270:
		return rotate270(img), nil
	case OpFlipH:
		return flipH(img), nil
	case OpFlipV:
		return flipV(img), nil
	case OpCrop:
		if op.Width == 0 || op.Height == 0 {
			return nil, &OpError{"zero_dimension"}
		}
		if op.X+op.Width > img.Width || op.Y+op.Height > img.Height {
			return nil, &OpError{"crop_out_of_bounds"}
		}
		return cropImg(img, op.X, op.Y, op.Width, op.Height), nil
	case OpBrightness:
		return mapPixels(img, func(p GSRgb) GSRgb { return adjBrightness(p, op.BrightnessDelta) }), nil
	case OpContrast:
		return mapPixels(img, func(p GSRgb) GSRgb { return adjContrast(p, op.ContrastFactor) }), nil
	}
	return nil, &OpError{"unknown_op"}
}

func ApplyPipeline(img *GSImage, pipeline Pipeline) (*GSImage, error) {
	cur := img
	for _, op := range pipeline {
		next, err := applyOp(cur, op)
		if err != nil {
			return nil, err
		}
		cur = next
	}
	return cur, nil
}

type BatchResult struct {
	Index  int
	Output *GSImage
	Error  string
}

func RunBatch(images []*GSImage, pipeline Pipeline) []BatchResult {
	results := make([]BatchResult, 0, len(images))
	for i, img := range images {
		out, err := ApplyPipeline(img, pipeline)
		if err != nil {
			results = append(results, BatchResult{Index: i, Error: err.Error()})
		} else {
			results = append(results, BatchResult{Index: i, Output: out})
		}
	}
	return results
}

func DryRunDims(start [2]uint32, pipeline Pipeline) ([][2]uint32, error) {
	out := [][2]uint32{start}
	cur := start
	for _, op := range pipeline {
		switch op.Kind {
		case OpResize:
			if op.Width == 0 || op.Height == 0 {
				return nil, &OpError{"zero_dimension"}
			}
			cur = [2]uint32{op.Width, op.Height}
		case OpGrayscale, OpFlipH, OpFlipV, OpRotate180, OpBrightness, OpContrast:
			// unchanged
		case OpRotate90, OpRotate270:
			cur = [2]uint32{cur[1], cur[0]}
		case OpCrop:
			if op.Width == 0 || op.Height == 0 {
				return nil, &OpError{"zero_dimension"}
			}
			if op.X+op.Width > cur[0] || op.Y+op.Height > cur[1] {
				return nil, &OpError{"crop_out_of_bounds"}
			}
			cur = [2]uint32{op.Width, op.Height}
		}
		out = append(out, cur)
	}
	return out, nil
}

// ---- constructors ----

func OpResizeNew(w, h uint32) Op { return Op{Kind: OpResize, Width: w, Height: h} }
func OpGrayscaleNew(a GSAlgorithm) Op { return Op{Kind: OpGrayscale, Algorithm: a} }
func OpRotate90New() Op { return Op{Kind: OpRotate90} }
func OpRotate180New() Op { return Op{Kind: OpRotate180} }
func OpRotate270New() Op { return Op{Kind: OpRotate270} }
func OpFlipHNew() Op { return Op{Kind: OpFlipH} }
func OpFlipVNew() Op { return Op{Kind: OpFlipV} }
func OpCropNew(x, y, w, h uint32) Op {
	return Op{Kind: OpCrop, X: x, Y: y, Width: w, Height: h}
}
func OpBrightnessNew(d int) Op { return Op{Kind: OpBrightness, BrightnessDelta: d} }
func OpContrastNew(f float64) Op { return Op{Kind: OpContrast, ContrastFactor: f} }

// ---- primitives ----

func clampU8(v int) uint8 {
	if v < 0 { return 0 }
	if v > 255 { return 255 }
	return uint8(v)
}

func adjBrightness(p GSRgb, d int) GSRgb {
	return GSRgb{clampU8(int(p.R) + d), clampU8(int(p.G) + d), clampU8(int(p.B) + d)}
}

func adjContrast(p GSRgb, f float64) GSRgb {
	adj := func(v uint8) uint8 {
		x := (float64(v)-128.0)*f + 128.0
		return clampU8(int(round(x)))
	}
	return GSRgb{adj(p.R), adj(p.G), adj(p.B)}
}

func round(x float64) float64 {
	if x >= 0 { return float64(int64(x + 0.5)) }
	return float64(int64(x - 0.5))
}

func mapPixels(img *GSImage, f func(GSRgb) GSRgb) *GSImage {
	out := make([]GSRgb, len(img.Pixels))
	for i, p := range img.Pixels {
		out[i] = f(p)
	}
	return &GSImage{Width: img.Width, Height: img.Height, Pixels: out}
}

func grayscaleImg(img *GSImage, algo GSAlgorithm) *GSImage {
	return mapPixels(img, func(p GSRgb) GSRgb {
		v := GSPixelToGray(p, algo)
		return GSRgb{v, v, v}
	})
}

func resizeNearest(img *GSImage, nw, nh uint32) *GSImage {
	pixels := make([]GSRgb, int(nw)*int(nh))
	for y := uint32(0); y < nh; y++ {
		for x := uint32(0); x < nw; x++ {
			sx := uint32(uint64(x) * uint64(img.Width) / uint64(nw))
			sy := uint32(uint64(y) * uint64(img.Height) / uint64(nh))
			if sx >= img.Width { sx = img.Width - 1 }
			if sy >= img.Height { sy = img.Height - 1 }
			p, _ := img.Get(sx, sy)
			pixels[int(y)*int(nw)+int(x)] = p
		}
	}
	return &GSImage{Width: nw, Height: nh, Pixels: pixels}
}

func buildImage(w, h uint32, cell func(x, y uint32) GSRgb) *GSImage {
	pixels := make([]GSRgb, int(w)*int(h))
	for y := uint32(0); y < h; y++ {
		for x := uint32(0); x < w; x++ {
			pixels[int(y)*int(w)+int(x)] = cell(x, y)
		}
	}
	return &GSImage{Width: w, Height: h, Pixels: pixels}
}

func rotate90(img *GSImage) *GSImage {
	w, h := img.Width, img.Height
	return buildImage(h, w, func(nx, ny uint32) GSRgb {
		p, _ := img.Get(ny, h-1-nx)
		return p
	})
}

func rotate180(img *GSImage) *GSImage {
	w, h := img.Width, img.Height
	return buildImage(w, h, func(nx, ny uint32) GSRgb {
		p, _ := img.Get(w-1-nx, h-1-ny)
		return p
	})
}

func rotate270(img *GSImage) *GSImage {
	w, h := img.Width, img.Height
	return buildImage(h, w, func(nx, ny uint32) GSRgb {
		p, _ := img.Get(w-1-ny, nx)
		return p
	})
}

func flipH(img *GSImage) *GSImage {
	w, h := img.Width, img.Height
	return buildImage(w, h, func(nx, ny uint32) GSRgb {
		p, _ := img.Get(w-1-nx, ny)
		return p
	})
}

func flipV(img *GSImage) *GSImage {
	w, h := img.Width, img.Height
	return buildImage(w, h, func(nx, ny uint32) GSRgb {
		p, _ := img.Get(nx, h-1-ny)
		return p
	})
}

func cropImg(img *GSImage, x0, y0, w, h uint32) *GSImage {
	return buildImage(w, h, func(nx, ny uint32) GSRgb {
		p, _ := img.Get(x0+nx, y0+ny)
		return p
	})
}

func BulkPictureDemo() {
	pixels := make([]GSRgb, 16)
	for y := 0; y < 4; y++ {
		for x := 0; x < 4; x++ {
			if (x+y)%2 == 0 {
				pixels[y*4+x] = GSRgb{200, 100, 50}
			} else {
				pixels[y*4+x] = GSRgb{50, 100, 200}
			}
		}
	}
	img := &GSImage{Width: 4, Height: 4, Pixels: pixels}

	pipeline := Pipeline{
		OpRotate90New(),
		OpBrightnessNew(10),
		OpGrayscaleNew(GSLuminosity),
	}

	out, _ := ApplyPipeline(img, pipeline)
	dims, _ := DryRunDims([2]uint32{4, 4}, pipeline)
	fmt.Printf("pipeline steps: %d\n", len(pipeline))
	fmt.Printf("dry-run dims: %v\n", dims)
	p := out.Pixels[0]
	fmt.Printf("output: %dx%d pixels[0]=(%d,%d,%d)\n", out.Width, out.Height, p.R, p.G, p.B)

	batch := []*GSImage{
		GSImageFilled(6, 6, GSRgb{128, 128, 128}),
		GSImageFilled(2, 2, GSRgb{128, 128, 128}),
	}
	cropPipe := Pipeline{OpCropNew(0, 0, 3, 3)}
	for _, r := range RunBatch(batch, cropPipe) {
		if r.Output != nil {
			fmt.Printf("  [%d] ok: %dx%d\n", r.Index, r.Output.Width, r.Output.Height)
		} else {
			fmt.Printf("  [%d] err: %s\n", r.Index, r.Error)
		}
	}
}
