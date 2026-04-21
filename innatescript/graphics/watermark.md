# G129 — Watermarking Application

> The Rosetta Stone's sixteenth **Graphics project**. Models **two orthogonal watermark techniques** every image-processing tool ships: (1) **visible watermarks** composited onto the base image at a position-ADT location with alpha-blend opacity; and (2) **LSB steganography** — hiding arbitrary bytes in the least-significant bits of pixel channels, with a 32-bit length-prefix header. The distinctive move: LSB steganography is new territory — we're storing data *in* pixels rather than *on* pixels. Capacity is a pure function of image size and bits-per-channel; embed/extract is a bit-packing/unpacking roundtrip.

```yaml
id: G129
title: Watermarking Application
category: graphics
requires: [G100-versioning-tool, G116-grayscale-converter, G123-screen-capture]
provides: [lsb-steganography, length-prefix-encoding, position-adt, alpha-compositing]
```

## Insight: LSB Steganography Hides Bytes in Pixels

Each pixel has 3 channels (R, G, B), each an 8-bit value. The low `bits_per_channel ∈ 1..=4` bits are "noise" in the visual sense — flipping them changes the pixel by at most `2^bits_per_channel - 1`, which for 1 bit is a ±1 change (invisible).

Embed: serialize (length + payload) as a bit stream; replace the low N bits of each channel with chunks of the stream.
Extract: read back the low N bits of each channel; reassemble the bit stream; decode length header; slice out payload bytes.

First Rosetta Stone project where **data is stored *in* pixels rather than *on* pixels**. G123 drew annotations over pixels; G129 hides data inside them.

## Insight: Capacity Is a Pure Function

```
total_bits = pixel_count * 3 * bits_per_channel
total_bytes = total_bits / 8
capacity = total_bytes - 4    # minus 4-byte length header
```

Callers can ask `lsb_capacity(image, bits)` before attempting `embed_lsb`. A 32×32 image at 1 bit/channel holds 380 bytes; at 4 bits/channel, 1532 bytes. The tradeoff is visibility: 4 bits/channel flips up to ±15, which produces visible speckle.

First Rosetta Stone project with **a capacity function that bounds a primary operation**. G120 had bin-packing capacity; G129 has steganographic capacity.

## Insight: 32-Bit Length-Prefix Header

Raw LSB extraction produces a bit stream of length `pixel_count * 3 * bits_per_channel`. Without a header, the decoder wouldn't know where the payload ends. The fix: the first 32 bits (before the payload bits) encode the payload's byte count as a big-endian `u32`.

Extract: read 32 bits → decode as u32 → payload_len. Then read `payload_len * 8` bits → slice into bytes. If either check fails (truncated image or claimed length exceeds available bits), error with `TruncatedData`.

First Rosetta Stone project with **a length-prefix header in a binary-packed format**. G126's RIFF had chunk headers but they were byte-aligned; G129's header is bit-packed within the pixel stream.

## Insight: High Bits Are Preserved

`masked = pixel.channel & ~((1 << bits_per_channel) - 1)`. Then `embedded = masked | chunk_bits`. The top `8 - bits_per_channel` bits of every channel are untouched. A viewer comparing the original to the embedded image sees at most a tiny shift in the low bits — imperceptible at `bits_per_channel = 1`, subtle speckle at 4.

First Rosetta Stone project where **a transformation preserves an explicit bit-range invariant**. Tests assert `pixel.r & 0xFE == 0xF0` after embedding — the top 7 bits of a `0xF0` channel are untouched.

## Insight: Position Is an ADT, Tiling Is a Mode

```
Position::{TopLeft, TopRight, BottomLeft, BottomRight, Center, Tiled}
```

`resolve_position(base, wm, pos) -> (x, y)` computes the top-left coordinate for the watermark. For all positions except Tiled, it's a single placement. Tiled is a *mode*, not a position — the compositor iterates `(x, y)` across the base by watermark dimensions.

First Rosetta Stone project where **a position enum has a "mode" variant that changes the operation shape**. G120's DiscState variants were all uniform (no special semantic); G129's Tiled triggers a loop instead of a single-call.

## Insight: Alpha Blend Shared with G123

The compositing formula is identical to G123's highlight: `out = (base * (255 - α) + wm * α) / 255` channel-wise. Different caller (whole-image overlay vs rectangle highlight), same math. Deliberate reuse — the channel-wise formula is the canonical way to composite.

First Rosetta Stone project to **reuse a numeric formula from a prior project unchanged**. G123 normalized brightness; G129 normalizes opacity; both use the same 8-bit weighted average.

## Insight: Tampered Length Header Fails Gracefully

A corrupted image's length-header bits might decode to a huge length. The extractor checks `bits.len() >= 32 + payload_len * 8` and errors with `TruncatedData` if it can't satisfy. An extraction never reads beyond the image's bit capacity.

Not a cryptographic guarantee (LSB isn't encryption) — but it is a shape-level guarantee: the extractor never panics on bad input.

First Rosetta Stone project where **the extractor has explicit safety checks against a corrupted header**. Previous projects trusted their input formats; G129 treats extraction as a potentially-adversarial decode.

## Choreographic Case: Vault Image Fingerprint

```innate
(@vault-image-fingerprint){
  @original <- @vault/load-image{path: @input-path}
  @metadata <- @vault/build-provenance{image-id: @original.id}

  @fingerprinted <- @wm/embed-lsb{
    image: @original,
    payload: @cbor/encode{data: @metadata},
    bits-per-channel: 2
  }
  @vault/save-image{path: @output-path, image: @fingerprinted}

  ;; Later, verify by extracting
  @loaded <- @vault/load-image{path: @output-path}
  @recovered <- @wm/extract-lsb{image: @loaded, bits-per-channel: 2}
  @recovered-metadata <- @cbor/decode{data: @recovered}

  (if (= @recovered-metadata @metadata)
      @ok
      (@alert "Image tampered or copied"))
}
```

Also applies to visible watermarks for copyright notices:

```innate
@branded <- @wm/apply-visible-watermark{
  base: @photo,
  watermark: @logo,
  position: "bottom-right",
  opacity: 180
}
```

## Structures

```innate
(defenum position
  TOP_LEFT | TOP_RIGHT | BOTTOM_LEFT | BOTTOM_RIGHT | CENTER | TILED)

(defenum watermark-error
  PAYLOAD_TOO_LARGE | INVALID_BITS_PER_CHANNEL | TRUNCATED_DATA)
```

## Resolver Natives

```innate
@wm/lsb-capacity{image, bits-per-channel}                  -> Int
@wm/embed-lsb{image, payload, bits-per-channel}             -> Image | WatermarkError
@wm/extract-lsb{image, bits-per-channel}                    -> Bytes | WatermarkError
@wm/apply-visible-watermark{base, watermark, position, opacity} -> Image
@wm/resolve-position{base, watermark, position}             -> (Int, Int)
```

## Demo

```innate
(@demo){
  @base <- @image-filled{width: 32, height: 32, color: {r: 128, g: 128, b: 128}}

  @wm/lsb-capacity{image: @base, bits-per-channel: 1}   ;; -> 380 bytes
  @wm/lsb-capacity{image: @base, bits-per-channel: 2}   ;; -> 764 bytes
  @wm/lsb-capacity{image: @base, bits-per-channel: 4}   ;; -> 1532 bytes

  @embedded <- @wm/embed-lsb{
    image: @base, payload: "Hello, watermark!", bits-per-channel: 1
  }
  @extracted <- @wm/extract-lsb{image: @embedded, bits-per-channel: 1}
  @extracted   ;; -> "Hello, watermark!"  (roundtrip)

  @wm-img <- @image-filled{width: 8, height: 8, color: {r: 255, g: 0, b: 0}}

  @tl <- @wm/apply-visible-watermark{
    base: @base, watermark: @wm-img, position: "top-left", opacity: 255
  }
  @tl.pixels[0]   ;; -> {r: 255, g: 0, b: 0}  (fully red at 255 opacity)

  @tr <- @wm/apply-visible-watermark{
    base: @base, watermark: @wm-img, position: "top-right", opacity: 128
  }
  @tr.pixels[31]  ;; -> {r: 191, g: 63, b: 63}  (half-blend with base 128,128,128)

  @tiled <- @wm/apply-visible-watermark{
    base: @base, watermark: @wm-img, position: "tiled", opacity: 255
  }
  ;; All 32×32 pixels become red (watermark repeats 4×4 times)
}
```

## Where

LSB embedding MUST preserve the high bits of each channel — zeroing them would destroy the cover image's content; the low bits are the only "safe" bits to overwrite. Length header MUST precede the payload in the bit stream — without it, the extractor has no termination signal. Length MUST be big-endian u32 — little-endian would be valid too, but Rosetta Stone demands one consistent choice across languages. `bits_per_channel` MUST be validated 1..=4 — 0 embeds nothing; >4 flips visible high bits. Capacity MUST subtract the 4-byte header — a function that returned total bits would let callers overflow into the header. Tiled MUST iterate in raster-order — irregular tiling produces gaps; simple row-major avoids them. Alpha blend MUST match G123's formula — two blend formulas in one codebase is a maintenance trap. The extractor MUST verify `bits.len() >= total_bits_needed` — a tampered header with a huge length must fail cleanly, not crash.
