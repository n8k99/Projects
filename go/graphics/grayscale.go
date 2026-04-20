// Grayscale Converter — pixel buffers + named conversion algorithms + histogram.
package graphics

import "math"

// GSRgb is a three-channel colour.
type GSRgb struct{ R, G, B uint8 }

// GSBlack and GSWhite constants.
var GSBlack = GSRgb{0, 0, 0}
var GSWhite = GSRgb{255, 255, 255}

// GSImage is a row-major pixel grid.
type GSImage struct {
	Width  uint32
	Height uint32
	Pixels []GSRgb
}

// NewGSImage returns a black image.
func NewGSImage(w, h uint32) *GSImage {
	return &GSImage{Width: w, Height: h, Pixels: make([]GSRgb, int(w)*int(h))}
}

// GSImageFilled returns an image with all pixels set to a colour.
func GSImageFilled(w, h uint32, color GSRgb) *GSImage {
	pixels := make([]GSRgb, int(w)*int(h))
	for i := range pixels { pixels[i] = color }
	return &GSImage{Width: w, Height: h, Pixels: pixels}
}

// GSImageFromPixels validates length and wraps.
func GSImageFromPixels(w, h uint32, pixels []GSRgb) *GSImage {
	if len(pixels) != int(w)*int(h) { return nil }
	return &GSImage{Width: w, Height: h, Pixels: pixels}
}

// Get returns pixel at (x, y), or nil if out of range.
func (i *GSImage) Get(x, y uint32) (GSRgb, bool) {
	if x >= i.Width || y >= i.Height { return GSRgb{}, false }
	return i.Pixels[int(y)*int(i.Width)+int(x)], true
}

// Set writes pixel at (x, y); returns true on success.
func (i *GSImage) Set(x, y uint32, p GSRgb) bool {
	if x >= i.Width || y >= i.Height { return false }
	i.Pixels[int(y)*int(i.Width)+int(x)] = p
	return true
}

// GSGrayImage is a single-channel brightness grid.
type GSGrayImage struct {
	Width  uint32
	Height uint32
	Pixels []uint8
}

// GSAlgorithm selects the conversion strategy.
type GSAlgorithm int

const (
	GSLuminosity GSAlgorithm = 0
	GSAverage    GSAlgorithm = 1
	GSLightness  GSAlgorithm = 2
	GSRed        GSAlgorithm = 3
	GSGreen      GSAlgorithm = 4
	GSBlue       GSAlgorithm = 5
)

// GSPixelToGray converts one pixel.
func GSPixelToGray(p GSRgb, algo GSAlgorithm) uint8 {
	switch algo {
	case GSLuminosity:
		v := 0.2126*float64(p.R) + 0.7152*float64(p.G) + 0.0722*float64(p.B)
		v = math.Round(v)
		if v < 0 { v = 0 }
		if v > 255 { v = 255 }
		return uint8(v)
	case GSAverage:
		return uint8((uint32(p.R) + uint32(p.G) + uint32(p.B)) / 3)
	case GSLightness:
		mx, mn := p.R, p.R
		if p.G > mx { mx = p.G }
		if p.B > mx { mx = p.B }
		if p.G < mn { mn = p.G }
		if p.B < mn { mn = p.B }
		return uint8((uint32(mx) + uint32(mn)) / 2)
	case GSRed: return p.R
	case GSGreen: return p.G
	case GSBlue: return p.B
	}
	return 0
}

// GSConvert converts an entire image.
func GSConvert(img *GSImage, algo GSAlgorithm) *GSGrayImage {
	pixels := make([]uint8, len(img.Pixels))
	for i, p := range img.Pixels {
		pixels[i] = GSPixelToGray(p, algo)
	}
	return &GSGrayImage{Width: img.Width, Height: img.Height, Pixels: pixels}
}

// GSHistogram returns a 256-bin frequency table.
func GSHistogram(g *GSGrayImage) [256]uint32 {
	var bins [256]uint32
	for _, v := range g.Pixels { bins[v]++ }
	return bins
}

// GSMeanBrightness returns mean pixel value in [0, 255].
func GSMeanBrightness(g *GSGrayImage) float64 {
	if len(g.Pixels) == 0 { return 0 }
	var sum uint64
	for _, v := range g.Pixels { sum += uint64(v) }
	return float64(sum) / float64(len(g.Pixels))
}

// GSContrastStddev returns the standard deviation of brightness.
func GSContrastStddev(g *GSGrayImage) float64 {
	if len(g.Pixels) == 0 { return 0 }
	m := GSMeanBrightness(g)
	var var_ float64
	for _, v := range g.Pixels {
		d := float64(v) - m
		var_ += d * d
	}
	return math.Sqrt(var_ / float64(len(g.Pixels)))
}

// GSDynamicRange returns (min, max) or (0, 0, false) if empty.
func GSDynamicRange(g *GSGrayImage) (uint8, uint8, bool) {
	if len(g.Pixels) == 0 { return 0, 0, false }
	mn, mx := g.Pixels[0], g.Pixels[0]
	for _, v := range g.Pixels[1:] {
		if v < mn { mn = v }
		if v > mx { mx = v }
	}
	return mn, mx, true
}
