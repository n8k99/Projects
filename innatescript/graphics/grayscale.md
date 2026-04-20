# G116 — Grayscale Converter

> The Rosetta Stone's third **Graphics project**. Introduces the **pixel buffer** — row-major `Vec<Rgb>` with width/height — and the **named-strategy** pattern for RGB→gray conversion. Six algorithms cover the useful space: **Luminosity** (Rec.709 perceptual), **Average** (arithmetic mean), **Lightness** (max+min midpoint), and the three single-channel isolations (Red/Green/Blue). Adds **histogram**, **mean brightness**, and **standard deviation** as basic image-metrics that downstream enhancement or filtering will reuse.

```yaml
id: G116
title: Grayscale Converter
category: graphics
requires: [G039-primes, G089-transaction-averages, G115-mind-mapper]
provides: [pixel-buffer-abstraction, named-conversion-strategies, luminosity-weights-Rec709, image-histogram-metrics]
```

## Insight: Pixel Buffers Are Flat Row-Major Vectors

A 2D image is stored as a 1D `Vec<Rgb>` of length `width * height`, indexed by `y * width + x`. Row-major. Trivial to allocate, trivial to iterate, and cache-friendly because each row is contiguous.

Every pixel-level image library on earth uses this layout — PIL, OpenCV, Rust's `image` crate, browser `ImageData`, OpenGL textures. The Rosetta Stone starts there.

First Rosetta Stone project with a **2D grid stored as a flat array**. G087's filesystem was nested; G113's forum trees were hash-keyed. G116's pixel buffer is contiguous positional storage — different shape, different access pattern.

## Insight: Grayscale Conversion Is a Closed Set of Named Strategies

RGB → gray is **lossy** (3 channels → 1), so "the right answer" depends on what you're optimising for:
* **Luminosity** (0.2126 R + 0.7152 G + 0.0722 B) — the **perceptual** answer. Matches how human eyes weight colours. The Rec.709 coefficients are the standard for HD/SDR video.
* **Average** — the arithmetic mean. Mathematically simple, visually wrong (blue looks too bright).
* **Lightness** — (max + min) / 2. Matches HSL's L axis. Useful when you care about how "lit" something is, not how bright.
* **Single-channel** (Red/Green/Blue) — isolate one channel. Useful for false-colour imaging, specific filter emulation.

All six are data; conversion is one function with a switch. No magic numbers buried in the code except the Rec.709 coefficients, which are standardised.

First Rosetta Stone project where **a closed set of named strategies** parameterises one conversion function. G112's dialects parameterised DDL emission; G116's algorithms parameterise gray conversion — same pattern, different domain.

## Insight: Luminosity Weights Are a Standard, Not a Choice

ITU-R BT.709 (1990) established the coefficients 0.2126 / 0.7152 / 0.0722 for HD video luminance. Older NTSC used slightly different weights (0.299 / 0.587 / 0.114, BT.601). Newer HDR uses BT.2020.

G116 picks BT.709 because it's the most common for web/SDR content. Sticking to a named standard means the output is **predictable across implementations** — Rust, Python, CL, Go, Lean, InnateScript all compute the same pixel values because they all use the same weights.

First Rosetta Stone project with **an external standard as a correctness premise**. G100's FNV-1a hash was a named algorithm; G116's luminosity is a named colour science standard. Both anchor cross-language equivalence.

## Insight: Histogram Is the Shape of the Image's Brightness Distribution

A grayscale image has 256 possible pixel values. The histogram counts how many pixels have each value. A dark image clusters near 0; a blown-out image clusters near 255; a balanced image spreads across the range.

G116's `histogram(gray)` returns a `[u32; 256]` — fixed-size array (or list in other languages). The shape of that array tells you everything about the image's tonal distribution. Photography apps call this the "tone curve" input; image-processing algorithms use it for auto-levels, histogram equalisation, thresholding.

First Rosetta Stone project where **a derived array is the primary signal** for downstream analysis. G089's transaction averages aggregated to scalars; G116's histogram aggregates to a distribution.

## Insight: Mean and Stddev Are Contrast Metrics

`mean_brightness` (average of all pixel values) tells you how bright the overall image is. `contrast_stddev` (standard deviation) tells you how much variance there is — a uniform image has stddev 0, a black-and-white checkerboard has stddev 128.

These are the starting point for auto-contrast features: "if stddev < 30, boost contrast"; "if mean > 200, darken midtones"; "if mean < 50, lift shadows". G116 computes them; downstream projects (future Graphics category items) can act on them.

First Rosetta Stone project where **statistical metrics on pixel data** are first-class. G089's transaction stats were domain-specific; G116's are visually meaningful.

## Insight: Empty Images Return Defined Zero Metrics

A 0×0 image has no pixels. `mean_brightness` returns 0.0, `contrast_stddev` returns 0.0, `dynamic_range` returns `None`. No crashes, no division by zero, no spurious metric values. Empty is a valid state of the data model and should be handled gracefully.

First Rosetta Stone project where **empty pixel data has explicit zero/None semantics**. G089 had empty-window stats; G116 extends to empty dimensions.

## Choreographic Case: Vault Thumbnail Pipeline

```innate
(@vault-thumbnail-pipeline){
  @images <- @vault/find{path: "images/*.png"}
  @for img-path in @images {
    @rgb <- @img/load{path: @img-path}
    @gray <- @gray/convert{image: @rgb, algorithm: "luminosity"}

    @mean <- @gray/mean-brightness{gray: @gray}
    @stddev <- @gray/contrast-stddev{gray: @gray}
    @when (@stddev < 30){
      @ui/flag{path: @img-path, reason: "low contrast"}
    }

    @hist <- @gray/histogram{gray: @gray}
    @vault/save-metadata{path: @img-path,
                          metadata: {mean: @mean, stddev: @stddev,
                                     histogram: @hist}}
  }
}
```

The vault walks its image directory, converts each to grayscale, computes metrics, flags low-contrast images, and stores the histogram as sidecar metadata. Pure data transformations chained into one pipeline.

## Structures

```innate
(defstruct rgb
  r : Int
  g : Int
  b : Int)

(defstruct image
  width  : Int
  height : Int
  pixels : [Rgb])

(defstruct gray-image
  width  : Int
  height : Int
  pixels : [Int])

(defenum algorithm
  LUMINOSITY | AVERAGE | LIGHTNESS | RED | GREEN | BLUE)
```

## Resolver Natives

```innate
@img/new{width, height}                     -> Image
@img/from-pixels{width, height, pixels}     -> Image | null
@img/get{image, x, y}                       -> Rgb | null
@img/set{image, x, y, pixel}                -> Bool
@gray/pixel-to-gray{pixel, algorithm}       -> Int
@gray/convert{image, algorithm}             -> GrayImage
@gray/histogram{gray}                       -> [Int]   ;; 256 bins
@gray/mean-brightness{gray}                 -> Float
@gray/contrast-stddev{gray}                 -> Float
@gray/dynamic-range{gray}                   -> (Int, Int) | null
```

## Demo

```innate
(@demo){
  @img <- @img/from-pixels{width: 2, height: 2,
                             pixels: [{r: 0, g: 0, b: 0},
                                      {r: 255, g: 0, b: 0},
                                      {r: 0, g: 255, b: 0},
                                      {r: 0, g: 0, b: 255}]}

  @lum <- @gray/convert{image: @img, algorithm: "luminosity"}
  ;; pixels=[0, 54, 182, 18]  — green is perceptually brightest

  @avg <- @gray/convert{image: @img, algorithm: "average"}
  ;; pixels=[0, 85, 85, 85]   — red/green/blue all equal (255/3)

  @gray/contrast-stddev{gray: @lum}  ;; ≈ 71
}
```

## Where

Pixel storage MUST be row-major (index = y * width + x) — every image library on earth uses this layout. Conversion algorithms MUST be a closed enum — open-ended strings invite drift and miss the common cases. Luminosity coefficients MUST be Rec.709 (0.2126/0.7152/0.0722) — that's the SDR/HDR standard; any other values produce different outputs on the same input. Histogram MUST be exactly 256 bins for 8-bit grayscale — one bin per possible value, no more no less. Empty images MUST return defined zero/None metrics, NOT crash — edge-case handling is mandatory when dimensions can be zero. Conversion MUST be pure (no side effects, no mutation of input) — the input image should remain usable after conversion for other processing.
