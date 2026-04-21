// Watermarking — LSB steganography + visible overlay + position ADT.
package graphics

import "fmt"

type WmPosition int

const (
	WmTopLeft WmPosition = iota
	WmTopRight
	WmBottomLeft
	WmBottomRight
	WmCenter
	WmTiled
)

type WatermarkError struct{ Reason string }

func (e *WatermarkError) Error() string { return e.Reason }

func LsbCapacity(image *GSImage, bitsPerChannel uint8) int {
	if bitsPerChannel == 0 || bitsPerChannel > 4 {
		return 0
	}
	totalBits := len(image.Pixels) * 3 * int(bitsPerChannel)
	totalBytes := totalBits / 8
	if totalBytes < 4 {
		return 0
	}
	return totalBytes - 4
}

func EmbedLsb(image *GSImage, payload []byte, bitsPerChannel uint8) (*GSImage, error) {
	if bitsPerChannel == 0 || bitsPerChannel > 4 {
		return nil, &WatermarkError{"invalid_bits_per_channel"}
	}
	cap := LsbCapacity(image, bitsPerChannel)
	if len(payload) > cap {
		return nil, &WatermarkError{fmt.Sprintf("payload %d > capacity %d", len(payload), cap)}
	}

	bits := make([]uint8, 0, (4+len(payload))*8)
	n := uint32(len(payload))
	lenBytes := []byte{byte(n >> 24), byte(n >> 16), byte(n >> 8), byte(n)}
	appendBits := func(b byte) {
		for i := 7; i >= 0; i-- {
			bits = append(bits, (b>>i)&1)
		}
	}
	for _, b := range lenBytes {
		appendBits(b)
	}
	for _, b := range payload {
		appendBits(b)
	}

	mask := ^((uint8(1) << bitsPerChannel) - 1)
	newPixels := make([]GSRgb, len(image.Pixels))
	copy(newPixels, image.Pixels)
	bitIdx := 0
	totalBits := len(bits)

	consumeChunk := func() uint8 {
		chunk := uint8(0)
		for i := uint8(0); i < bitsPerChannel; i++ {
			if bitIdx >= totalBits { break }
			chunk = (chunk << 1) | bits[bitIdx]
			bitIdx++
		}
		return chunk
	}

	for i := range newPixels {
		if bitIdx >= totalBits { break }
		p := &newPixels[i]
		p.R = (p.R & mask) | consumeChunk()
		if bitIdx < totalBits {
			p.G = (p.G & mask) | consumeChunk()
		}
		if bitIdx < totalBits {
			p.B = (p.B & mask) | consumeChunk()
		}
	}

	return &GSImage{Width: image.Width, Height: image.Height, Pixels: newPixels}, nil
}

func ExtractLsb(image *GSImage, bitsPerChannel uint8) ([]byte, error) {
	if bitsPerChannel == 0 || bitsPerChannel > 4 {
		return nil, &WatermarkError{"invalid_bits_per_channel"}
	}
	mask := (uint8(1) << bitsPerChannel) - 1
	bits := make([]uint8, 0, len(image.Pixels)*3*int(bitsPerChannel))
	for _, p := range image.Pixels {
		for _, ch := range [3]uint8{p.R, p.G, p.B} {
			chunk := ch & mask
			for i := uint8(0); i < bitsPerChannel; i++ {
				bits = append(bits, (chunk>>(bitsPerChannel-1-i))&1)
			}
		}
	}
	if len(bits) < 32 {
		return nil, &WatermarkError{"truncated_data"}
	}
	var payloadLen uint32
	for i := 0; i < 32; i++ {
		payloadLen = (payloadLen << 1) | uint32(bits[i])
	}
	totalBitsNeeded := 32 + int(payloadLen)*8
	if len(bits) < totalBitsNeeded {
		return nil, &WatermarkError{"truncated_data"}
	}
	payload := make([]byte, payloadLen)
	for byteI := uint32(0); byteI < payloadLen; byteI++ {
		var b byte
		for bitJ := 0; bitJ < 8; bitJ++ {
			b = (b << 1) | byte(bits[32+int(byteI)*8+bitJ])
		}
		payload[byteI] = b
	}
	return payload, nil
}

func ResolveWmPosition(base, wm *GSImage, pos WmPosition) (int32, int32) {
	bw := int32(base.Width)
	bh := int32(base.Height)
	ww := int32(wm.Width)
	wh := int32(wm.Height)
	switch pos {
	case WmTopLeft:
		return 0, 0
	case WmTopRight:
		x := bw - ww
		if x < 0 { x = 0 }
		return x, 0
	case WmBottomLeft:
		y := bh - wh
		if y < 0 { y = 0 }
		return 0, y
	case WmBottomRight:
		x := bw - ww
		y := bh - wh
		if x < 0 { x = 0 }
		if y < 0 { y = 0 }
		return x, y
	case WmCenter:
		return (bw - ww) / 2, (bh - wh) / 2
	}
	return 0, 0
}

func wmBlendChannel(a, b, alpha uint8) uint8 {
	return uint8((uint16(a)*uint16(255-alpha) + uint16(b)*uint16(alpha)) / 255)
}

func wmBlendRegion(out *GSImage, x0, y0 int32, wm *GSImage, opacity uint8) {
	for wy := uint32(0); wy < wm.Height; wy++ {
		for wx := uint32(0); wx < wm.Width; wx++ {
			bx := x0 + int32(wx)
			by := y0 + int32(wy)
			if bx < 0 || by < 0 { continue }
			if uint32(bx) >= out.Width || uint32(by) >= out.Height { continue }
			baseIdx := uint32(by)*out.Width + uint32(bx)
			wmIdx := wy*wm.Width + wx
			bp := out.Pixels[baseIdx]
			wp := wm.Pixels[wmIdx]
			out.Pixels[baseIdx] = GSRgb{
				R: wmBlendChannel(bp.R, wp.R, opacity),
				G: wmBlendChannel(bp.G, wp.G, opacity),
				B: wmBlendChannel(bp.B, wp.B, opacity),
			}
		}
	}
}

func ApplyVisibleWatermark(base, watermark *GSImage, position WmPosition, opacity uint8) *GSImage {
	pixels := make([]GSRgb, len(base.Pixels))
	copy(pixels, base.Pixels)
	out := &GSImage{Width: base.Width, Height: base.Height, Pixels: pixels}
	if position == WmTiled {
		for y := uint32(0); y < base.Height; y += watermark.Height {
			for x := uint32(0); x < base.Width; x += watermark.Width {
				wmBlendRegion(out, int32(x), int32(y), watermark, opacity)
			}
		}
	} else {
		x, y := ResolveWmPosition(base, watermark, position)
		wmBlendRegion(out, x, y, watermark, opacity)
	}
	return out
}

func WatermarkDemo() {
	base := GSImageFilled(32, 32, GSRgb{128, 128, 128})
	fmt.Printf("capacity @ 1 bit/chan: %d bytes\n", LsbCapacity(base, 1))
	fmt.Printf("capacity @ 2 bits/chan: %d bytes\n", LsbCapacity(base, 2))
	fmt.Printf("capacity @ 4 bits/chan: %d bytes\n", LsbCapacity(base, 4))

	payload := []byte("Hello, watermark!")
	embedded, _ := EmbedLsb(base, payload, 1)
	extracted, _ := ExtractLsb(embedded, 1)
	ok := len(extracted) == len(payload)
	if ok {
		for i := range extracted {
			if extracted[i] != payload[i] { ok = false; break }
		}
	}
	fmt.Printf("roundtrip ok: %v\n", ok)
	fmt.Printf("extracted: %s\n", extracted)

	wm := GSImageFilled(8, 8, GSRgb{255, 0, 0})
	tl := ApplyVisibleWatermark(base, wm, WmTopLeft, 255)
	br := ApplyVisibleWatermark(base, wm, WmBottomRight, 255)
	tr := ApplyVisibleWatermark(base, wm, WmTopRight, 128)
	tiled := ApplyVisibleWatermark(base, wm, WmTiled, 255)
	tlp := tl.Pixels[0]
	brp := br.Pixels[31*32+31]
	trp := tr.Pixels[31]
	fmt.Printf("top-left (0,0): Rgb(r=%d, g=%d, b=%d)\n", tlp.R, tlp.G, tlp.B)
	fmt.Printf("bottom-right (31,31): Rgb(r=%d, g=%d, b=%d)\n", brp.R, brp.G, brp.B)
	fmt.Printf("top-right (31,0) blended: Rgb(r=%d, g=%d, b=%d)\n", trp.R, trp.G, trp.B)
	allRed := true
	for _, p := range tiled.Pixels {
		if p.R != 255 || p.G != 0 || p.B != 0 { allRed = false; break }
	}
	fmt.Printf("tiled all red: %v\n", allRed)
}
