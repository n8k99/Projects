// MP3-to-Wav Converter — audio buffer + resample + normalize + RIFF encoding.
package graphics

import (
	"encoding/binary"
	"fmt"
)

type AudioBuffer struct {
	SampleRate uint32
	Channels   uint16
	Samples    []int16
}

func (b *AudioBuffer) FrameCount() int {
	if b.Channels == 0 {
		return 0
	}
	return len(b.Samples) / int(b.Channels)
}

func (b *AudioBuffer) DurationMs() uint64 {
	if b.SampleRate == 0 {
		return 0
	}
	return uint64(b.FrameCount()) * 1000 / uint64(b.SampleRate)
}

type Mp3Frame struct {
	SampleRate       uint32
	Channels         uint16
	BitrateKbps      uint32
	SamplesPerFrame  uint32
	Pcm              []int16
}

type CodecError struct{ Reason string }

func (e *CodecError) Error() string { return e.Reason }

func DecodeMp3ToBuffer(frames []Mp3Frame) (*AudioBuffer, error) {
	if len(frames) == 0 {
		return nil, &CodecError{"no_frames"}
	}
	sr := frames[0].SampleRate
	ch := frames[0].Channels
	if sr == 0 {
		return nil, &CodecError{"sample_rate_zero"}
	}
	if ch != 1 && ch != 2 {
		return nil, &CodecError{fmt.Sprintf("unsupported_channels: %d", ch)}
	}
	for _, f := range frames {
		if f.SampleRate != sr || f.Channels != ch {
			return nil, &CodecError{"mixed_formats"}
		}
	}
	var samples []int16
	for _, f := range frames {
		samples = append(samples, f.Pcm...)
	}
	return &AudioBuffer{SampleRate: sr, Channels: ch, Samples: samples}, nil
}

func clampI16(v int64) int16 {
	if v > 32767 {
		return 32767
	}
	if v < -32768 {
		return -32768
	}
	return int16(v)
}

func Resample(buf *AudioBuffer, dstRate uint32) (*AudioBuffer, error) {
	if buf.SampleRate == 0 || dstRate == 0 {
		return nil, &CodecError{"sample_rate_zero"}
	}
	if dstRate == buf.SampleRate {
		out := make([]int16, len(buf.Samples))
		copy(out, buf.Samples)
		return &AudioBuffer{SampleRate: buf.SampleRate, Channels: buf.Channels, Samples: out}, nil
	}
	inFrames := buf.FrameCount()
	if inFrames == 0 {
		return &AudioBuffer{SampleRate: dstRate, Channels: buf.Channels}, nil
	}
	outFrames := (uint64(inFrames)*uint64(dstRate) + uint64(buf.SampleRate)/2) / uint64(buf.SampleRate)
	channels := int(buf.Channels)
	out := make([]int16, 0, int(outFrames)*channels)
	for i := uint64(0); i < outFrames; i++ {
		srcPos := i * uint64(buf.SampleRate) * 1000 / uint64(dstRate)
		srcFrame := int(srcPos / 1000)
		frac := int64(srcPos % 1000)
		nextFrame := srcFrame + 1
		if nextFrame >= inFrames {
			nextFrame = inFrames - 1
		}
		for c := 0; c < channels; c++ {
			a := int64(buf.Samples[srcFrame*channels+c])
			b := int64(buf.Samples[nextFrame*channels+c])
			v := a + (b-a)*frac/1000
			out = append(out, clampI16(v))
		}
	}
	return &AudioBuffer{SampleRate: dstRate, Channels: buf.Channels, Samples: out}, nil
}

func NormalizeToPeak(buf *AudioBuffer, targetAmplitude int32) *AudioBuffer {
	if len(buf.Samples) == 0 {
		return &AudioBuffer{SampleRate: buf.SampleRate, Channels: buf.Channels}
	}
	var maxAbs int32
	for _, s := range buf.Samples {
		a := int32(s)
		if a < 0 {
			a = -a
		}
		if a > maxAbs {
			maxAbs = a
		}
	}
	if maxAbs == 0 {
		out := make([]int16, len(buf.Samples))
		copy(out, buf.Samples)
		return &AudioBuffer{SampleRate: buf.SampleRate, Channels: buf.Channels, Samples: out}
	}
	target := targetAmplitude
	if target > 32767 {
		target = 32767
	}
	if target < 0 {
		target = 0
	}
	out := make([]int16, len(buf.Samples))
	for i, s := range buf.Samples {
		v := int64(s) * int64(target) / int64(maxAbs)
		out[i] = clampI16(v)
	}
	return &AudioBuffer{SampleRate: buf.SampleRate, Channels: buf.Channels, Samples: out}
}

func ToMono(buf *AudioBuffer) *AudioBuffer {
	if buf.Channels == 1 {
		out := make([]int16, len(buf.Samples))
		copy(out, buf.Samples)
		return &AudioBuffer{SampleRate: buf.SampleRate, Channels: 1, Samples: out}
	}
	n := buf.FrameCount()
	out := make([]int16, n)
	for i := 0; i < n; i++ {
		l := int32(buf.Samples[i*2])
		r := int32(buf.Samples[i*2+1])
		out[i] = int16((l + r) / 2)
	}
	return &AudioBuffer{SampleRate: buf.SampleRate, Channels: 1, Samples: out}
}

func ToStereo(buf *AudioBuffer) *AudioBuffer {
	if buf.Channels == 2 {
		out := make([]int16, len(buf.Samples))
		copy(out, buf.Samples)
		return &AudioBuffer{SampleRate: buf.SampleRate, Channels: 2, Samples: out}
	}
	out := make([]int16, 0, len(buf.Samples)*2)
	for _, s := range buf.Samples {
		out = append(out, s, s)
	}
	return &AudioBuffer{SampleRate: buf.SampleRate, Channels: 2, Samples: out}
}

func EncodeWav(buf *AudioBuffer) []byte {
	bitsPerSample := uint16(16)
	byteRate := buf.SampleRate * uint32(buf.Channels) * uint32(bitsPerSample/8)
	blockAlign := buf.Channels * (bitsPerSample / 8)
	dataSize := uint32(len(buf.Samples) * 2)
	riffSize := 36 + dataSize

	out := make([]byte, 0, 44+int(dataSize))
	out = append(out, "RIFF"...)
	out = binary.LittleEndian.AppendUint32(out, riffSize)
	out = append(out, "WAVE"...)
	out = append(out, "fmt "...)
	out = binary.LittleEndian.AppendUint32(out, 16)
	out = binary.LittleEndian.AppendUint16(out, 1)
	out = binary.LittleEndian.AppendUint16(out, buf.Channels)
	out = binary.LittleEndian.AppendUint32(out, buf.SampleRate)
	out = binary.LittleEndian.AppendUint32(out, byteRate)
	out = binary.LittleEndian.AppendUint16(out, blockAlign)
	out = binary.LittleEndian.AppendUint16(out, bitsPerSample)
	out = append(out, "data"...)
	out = binary.LittleEndian.AppendUint32(out, dataSize)
	for _, s := range buf.Samples {
		out = binary.LittleEndian.AppendUint16(out, uint16(s))
	}
	return out
}

func DecodeWav(data []byte) (*AudioBuffer, error) {
	if len(data) < 44 {
		return nil, &CodecError{"empty_buffer"}
	}
	if string(data[0:4]) != "RIFF" || string(data[8:12]) != "WAVE" {
		return nil, &CodecError{"empty_buffer"}
	}
	channels := binary.LittleEndian.Uint16(data[22:24])
	sampleRate := binary.LittleEndian.Uint32(data[24:28])
	dataSize := int(binary.LittleEndian.Uint32(data[40:44]))
	samples := make([]int16, dataSize/2)
	for i := 0; i < dataSize/2; i++ {
		samples[i] = int16(binary.LittleEndian.Uint16(data[44+i*2 : 44+i*2+2]))
	}
	return &AudioBuffer{SampleRate: sampleRate, Channels: channels, Samples: samples}, nil
}

func Mp3WavConverterDemo() {
	frames := []Mp3Frame{
		{SampleRate: 22050, Channels: 2, BitrateKbps: 128, SamplesPerFrame: 4,
			Pcm: []int16{100, -100, 200, -200, 400, -400, 800, -800}},
		{SampleRate: 22050, Channels: 2, BitrateKbps: 128, SamplesPerFrame: 2,
			Pcm: []int16{-500, 500, -1000, 1000}},
	}
	buf, _ := DecodeMp3ToBuffer(frames)
	fmt.Printf("decoded: rate=%d ch=%d samples=%d\n", buf.SampleRate, buf.Channels, len(buf.Samples))

	resampled, _ := Resample(buf, 44100)
	fmt.Printf("resampled: rate=%d ch=%d samples=%d\n", resampled.SampleRate, resampled.Channels, len(resampled.Samples))

	normalized := NormalizeToPeak(resampled, 32000)
	var maxAbs int32
	for _, s := range normalized.Samples {
		a := int32(s)
		if a < 0 { a = -a }
		if a > maxAbs { maxAbs = a }
	}
	fmt.Printf("normalized max_abs: %d\n", maxAbs)

	mono := ToMono(normalized)
	fmt.Printf("mono: ch=%d samples=%d\n", mono.Channels, len(mono.Samples))

	wav := EncodeWav(mono)
	fmt.Printf("wav bytes: %d\n", len(wav))
	fmt.Printf("header: %s %s\n", wav[0:4], wav[8:12])

	decoded, _ := DecodeWav(wav)
	match := len(decoded.Samples) == len(mono.Samples)
	if match {
		for i := range decoded.Samples {
			if decoded.Samples[i] != mono.Samples[i] {
				match = false
				break
			}
		}
	}
	fmt.Printf("decoded wav: rate=%d ch=%d samples=%d\n", decoded.SampleRate, decoded.Channels, len(decoded.Samples))
	fmt.Printf("samples match: %v\n", match)
}
