# G126 — MP3 to Wav Converter

> The Rosetta Stone's thirteenth **Graphics project**. Models the **audio format converter** every media tool ships (ffmpeg / lame / sox): decode MP3 frames to PCM, optionally resample to a different rate, normalize amplitude, remix channels, then encode as RIFF/WAV bytes. The distinctive move: audio is a **time-indexed typed buffer** (`AudioBuffer { sample_rate, channels, samples: Vec<i16> }`) — parallel to G116's pixel buffer but oriented on the time axis instead of the spatial axis. Every operation (resample, normalize, to_mono, to_stereo) is a pure function from buffer to buffer.

```yaml
id: G126
title: MP3 to Wav Converter
category: graphics
requires: [G116-grayscale-converter, G117-stream-player, G118-mp3-player]
provides: [audio-buffer, linear-resample, peak-normalize, channel-mix, riff-serialization]
```

## Insight: Audio Buffer Mirrors Pixel Buffer

G116 had `Image { width, height, pixels: Vec<Rgb> }` — a 2D grid in spatial dimensions. G126 has `AudioBuffer { sample_rate, channels, samples: Vec<i16> }` — a 1D interleaved sequence in the time dimension. `sample_rate` plays the role of dimension-per-unit-time that `width/height` played in space.

Same compositional pattern: ops take a buffer and return a buffer. Same discipline: interleaved layout (L, R, L, R, ...) parallels row-major pixel layout (R0C0, R0C1, R0C2, R1C0, ...).

First Rosetta Stone project where **the time axis gets the same typed-buffer treatment as the spatial axis**. The data shape (Vec of samples) is the same; the semantics (sample rate determines duration) is new.

## Insight: Linear-Interpolation Resample Is the Canonical "Good Enough"

For every output frame `i` at `dst_rate`, compute the source position in fractional source-frame units: `src_pos = i * src_rate / dst_rate`. Integer part = `src_frame`; fractional part = interpolation weight. Output sample = `a + (b - a) * frac / 1000` where a = source[src_frame] and b = source[src_frame + 1].

Integer-only math (scaled by 1000) gives byte-identical output across languages. Higher-order interpolation (sinc, polyphase) is better for audio fidelity but requires floats — bad for cross-language determinism.

First Rosetta Stone project where **integer-scaled linear interpolation** is chosen over float math for cross-language byte-identity. G116's grayscale also scaled to integers; G126 scales *time* to integers.

## Insight: Two-Pass Normalize Beats One-Pass "Compressor"

Pass 1: scan the buffer for `max_abs` (the loudest single sample, absolute value). Pass 2: scale every sample by `target / max_abs`.

A one-pass "compress to target" (threshold + ratio + attack + release) is fundamentally different — it's a dynamic-range compressor, not a normalizer. Normalize is **linear and memoryless**: pure function from samples to samples. Compression is stateful. This project does normalize because determinism matters.

First Rosetta Stone project with a **two-pass algorithm where the first pass computes a scalar that parameterizes the second**. G120's FFD bin-packing had a pre-sort pass; G126 has a scan-for-peak pass.

## Insight: Channel Mix Is a Micro-Operation

Stereo → mono: `out[i] = (L + R) / 2`. Mono → stereo: `out[2i] = out[2i+1] = in[i]`. Two-line implementations; no configuration. The "hard problems" (matrixed surround, HRTF binaural mixing) aren't in scope — the basic stereo/mono pair is.

First Rosetta Stone project where **channel remix is explicitly trivial**. Real pro-audio handles 5.1 → stereo via downmix coefficients; G126 handles L/R → mono via arithmetic mean.

## Insight: RIFF Is a Chunk-Structured Byte Layout

WAV is a RIFF file: "RIFF" magic + overall length + "WAVE" subtype + "fmt " chunk (16 bytes: PCM/channels/rate/byte-rate/block-align/bits) + "data" chunk (length + samples). All integers are **little-endian**; all lengths exclude the chunk header's own 8 bytes.

Encoding is a sequence of byte-appends; decoding is a sequence of offset-reads. The 44-byte header layout is canonical — tests assert exact offsets.

First Rosetta Stone project to serialize a **structured binary format with chunk-length prefixes**. G112's DDL dialect produced text; G126 produces bytes with explicit little-endian integer layouts.

## Insight: Int-16 PCM Is the Convergence Point

MP3's output frame (simulated or real) is int16. WAV's 16-bit PCM is int16. Every op in the pipeline keeps samples as int16. No float intermediary — same reason as resample: cross-language determinism.

Clamping to `[-32768, 32767]` happens explicitly at every arithmetic boundary (resample interpolation, normalize scaling). Overflow is a bug; the clamp is the tripwire.

First Rosetta Stone project where **a specific numeric type (i16) is the sole intermediate format** across a whole pipeline. Prior projects let floats cross boundaries; G126 forbids it.

## Insight: Frame-by-Frame Decoding Pattern

`decode_mp3_to_buffer(frames)` iterates over `Vec<Mp3Frame>` and concatenates PCM. Frames validate per-frame metadata (same sample_rate, same channels) before concatenation — a mixed-format stream errors, not silently mis-decodes.

Parallel to G117's segment loading: G117 had `loadNextSegment(player)` that accumulates bytes per segment; G126 has `decode(frames)` that accumulates samples per frame. Different units (bytes vs samples), same pattern.

First Rosetta Stone project where **frame iteration is the decoding primitive**. G117 had stream segments (variable rate); G126 has MP3 frames (fixed rate per frame).

## Choreographic Case: Vault Audio Archive Export

```innate
(@vault-audio-export){
  @mp3-file <- @vault/load-mp3{path: @input-path}
  @frames <- @mp3/parse-frames{bytes: @mp3-file}

  @buf <- @mpw/decode-mp3-to-buffer{frames: @frames}
  @resampled <- @mpw/resample{buffer: @buf, dst-rate: 44100}
  @normalized <- @mpw/normalize-to-peak{buffer: @resampled, target: 32000}
  @mono <- @mpw/to-mono{buffer: @normalized}
  @wav-bytes <- @mpw/encode-wav{buffer: @mono}

  @vault/save-bytes{path: @output-path, data: @wav-bytes}
}
```

Pure-function pipeline from frames to WAV bytes. Every step is referentially transparent; the same input produces the same bytes.

## Structures

```innate
(defstruct audio-buffer
  sample-rate : Int
  channels    : Int            ;; 1 or 2
  samples     : [Int16])       ;; interleaved: L,R,L,R,... for stereo

(defstruct mp3-frame
  sample-rate        : Int
  channels           : Int
  bitrate-kbps       : Int
  samples-per-frame  : Int
  pcm                : [Int16])

(defenum codec-error
  NO_FRAMES | MIXED_FORMATS | EMPTY_BUFFER
  | UNSUPPORTED_CHANNELS | SAMPLE_RATE_ZERO)
```

## Resolver Natives

```innate
@mpw/decode-mp3-to-buffer{frames}              -> AudioBuffer | CodecError
@mpw/resample{buffer, dst-rate}                -> AudioBuffer | CodecError
@mpw/normalize-to-peak{buffer, target}         -> AudioBuffer
@mpw/to-mono{buffer}                            -> AudioBuffer
@mpw/to-stereo{buffer}                          -> AudioBuffer
@mpw/encode-wav{buffer}                         -> Bytes
@mpw/decode-wav{bytes}                          -> AudioBuffer | CodecError
```

## Demo

```innate
(@demo){
  @frames <- [
    {sample-rate: 22050, channels: 2, samples-per-frame: 4,
     pcm: [100, -100, 200, -200, 400, -400, 800, -800]},
    {sample-rate: 22050, channels: 2, samples-per-frame: 2,
     pcm: [-500, 500, -1000, 1000]}
  ]
  @buf <- @mpw/decode-mp3-to-buffer{frames: @frames}
  @buf.sample-rate     ;; -> 22050
  @buf.channels        ;; -> 2
  (length @buf.samples) ;; -> 12

  @resampled <- @mpw/resample{buffer: @buf, dst-rate: 44100}
  (length @resampled.samples)  ;; -> 24 (doubled)

  @normalized <- @mpw/normalize-to-peak{buffer: @resampled, target: 32000}
  (max @normalized.samples (fn (s) (abs s)))  ;; -> 32000

  @mono <- @mpw/to-mono{buffer: @normalized}
  @mono.channels   ;; -> 1

  @wav <- @mpw/encode-wav{buffer: @mono}
  (bytes-slice @wav 0 4)     ;; -> "RIFF"
  (bytes-slice @wav 8 12)    ;; -> "WAVE"

  @decoded <- @mpw/decode-wav{bytes: @wav}
  @decoded.samples == @mono.samples   ;; -> true (byte-identical roundtrip)
}
```

## Where

AudioBuffer layout MUST be interleaved — parallel-array (separate L and R vectors) makes resample logic bifurcate and adds cache misses. Sample type MUST be int16 throughout the pipeline — floats introduce platform-specific rounding; Rosetta Stone demands byte-identical output. Resample MUST use integer-scaled linear interpolation — higher-order filters (sinc) require floats and lose cross-language determinism. Normalize MUST be two-pass — in-place peak computation is impossible; the scalar gates the second pass. Channel mix MUST be separate from normalize — composability requires each op stand alone. WAV encoder MUST emit little-endian — RIFF spec requires it; big-endian WAVs fail every decoder. Chunk sizes MUST exclude the 8-byte chunk header — off-by-8 is the classic WAV bug. Frame metadata mismatch MUST error, not silently concatenate — mixed sample rates produce garbage audio; fail loudly. The 44-byte header layout MUST be tested at specific offsets — "it decodes somewhere" is not a round-trip guarantee.
