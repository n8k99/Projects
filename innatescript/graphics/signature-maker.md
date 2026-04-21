# G127 — Signature Maker

> The Rosetta Stone's fourteenth **Graphics project**. Models the **ink-capture kernel** every touchscreen signature pad ships: pen-down → stream of `(x, y, pressure, t_ms)` points → pen-up, a *stroke*; many strokes → a *signature*. The distinctive move: **variable-width rendering driven by pressure**, **Catmull-Rom spline smoothing** of raw input points, **bounding-box normalization** so two signatures of different sizes become directly comparable, and a **point-to-point similarity metric** for rough signature-match scoring. Integer-scaled math throughout for cross-language byte-identity.

```yaml
id: G127
title: Signature Maker
category: graphics
requires: [G115-mind-mapper, G119-bulk-picture, G123-screen-capture]
provides: [stroke-model, catmull-rom-smoothing, variable-width-render, signature-normalization, similarity-metric]
```

## Insight: Pressure + Timestamp Are First-Class Per-Point

```
Point { x: i32, y: i32, pressure: u16 (0..=1024), t_ms: u64 }
```

Pressure drives line thickness; timestamp captures stroke dynamics (fast signatures differ from slow ones). Both persist through smoothing (linear-interpolated between input points) and normalization (preserved as-is).

First Rosetta Stone project where **points carry non-spatial metadata per sample**. G117's segments had per-bitrate sizes; G127 has per-point pressure + time.

## Insight: Catmull-Rom Smoothing Is a Local 4-Point Interpolant

Given four consecutive input points `p0, p1, p2, p3`, Catmull-Rom produces a smooth curve that passes through `p1` and `p2` (not `p0` or `p3` — those are "influence" points). For `t ∈ [0, 1]`, the formula:

```
out(t) = 0.5 * (2*p1
              + (-p0 + p2) * t
              + (2p0 - 5p1 + 4p2 - p3) * t²
              + (-p0 + 3p1 - 3p2 + p3) * t³)
```

Integer-scaled: `t` is 0..=1000. Everything multiplied by powers of 1000, final division by 2_000_000. Byte-identical across all six languages.

First Rosetta Stone project to use **Catmull-Rom splines** as the smoothing primitive. G119 had pixel transforms (rotate, flip, crop); G127 has a polynomial curve fit over points.

## Insight: Smoothing Preserves Endpoints

Raw stroke has N points; smoothed stroke has `1 + (N - 3) * steps + 1` points. The first and last raw points are preserved exactly (the interior is smoothed). This matters for signature verification: the user lifts the pen at specific points, and those are signal, not noise.

First Rosetta Stone project where **endpoint fidelity is a property of the smoothing algorithm**, explicitly tested.

## Insight: Variable-Width Rendering via Disk Stamps

Bresenham produces a line trajectory. At each point on the line, we stamp a **filled disk** of radius `width / 2` (derived from average pressure of the segment's endpoints). Disks overlap, producing a continuous variable-width stroke. Thick at high pressure, thin at low.

G123's Bresenham was single-pixel arrow; G127's is disk-stamped variable-width ink. Same trajectory algorithm, different per-pixel operation.

First Rosetta Stone project with **disk-stamped line rendering**. The algorithmic novelty isn't Bresenham (we've seen it); it's that the ink is radial, not linear.

## Insight: Bounding-Box Normalization Enables Direct Comparison

Different signature pads produce different coordinate ranges. `normalize(sig, target_w, target_h)` translates to origin (min_x, min_y) → (0, 0) and scales to (target_w, target_h). After normalization, two signatures of any original size are in the same coordinate frame.

`scale_x = target_w * 1000 / (max_x - min_x)` (integer-scaled). Last-point rounding can land at `target_w - 1` instead of `target_w` exactly — a known integer-math artifact preserved across all six languages.

First Rosetta Stone project where **affine normalization (translate + scale) is the preprocessing for comparison**. G122 had aspect-ratio fit scoring; G127 actually performs the transform.

## Insight: Similarity as Sum of Nearest-Point Distances

For each point in signature A, find the nearest point in signature B; sum the squared distances. Low = similar. Zero = identical. Same shape at different sizes → low after both are normalized to the same target.

This is much simpler than Dynamic Time Warping (DTW) but captures the core intuition: signatures that trace similar shapes produce points close to each other's points, regardless of speed or spacing.

First Rosetta Stone project with **a similarity metric over point sets** (as opposed to vectors of scalars, which G086 frecency had).

## Insight: Same Shape ≠ Same Size Before Normalization

```
small: [(0,0), (10,0), (10,10), (0,10)]   # 10x10 square
big:   [(0,0), (100,0), (100,100), (0,100)]  # 100x100 square
```

`similarity_distance(small, big)` is large (points are spatially far apart). But `similarity_distance(normalize(small, 1000, 1000), normalize(big, 1000, 1000))` is 0 — same shape after size correction.

This is the payoff: normalization is the precondition for similarity; similarity without normalization is nearly useless.

First Rosetta Stone project where **two operations compose to produce a meaningful composite** (normalize+compare), and the composition is essential for the result.

## Choreographic Case: Vault Signature Capture

```innate
(@vault-signature-capture){
  @sig <- @sig/new-signature{}

  @on-pen-down (@x @y @pressure @t){
    @current-stroke <- @sig/start-stroke{}
    @sig/add-point{stroke: @current-stroke, x: @x, y: @y,
                     pressure: @pressure, t-ms: @t}
  }

  @on-pen-move (@x @y @pressure @t){
    @sig/add-point{stroke: @current-stroke, x: @x, y: @y,
                     pressure: @pressure, t-ms: @t}
  }

  @on-pen-up{
    @smoothed <- @sig/smooth-stroke{stroke: @current-stroke, steps: 8}
    @sig/add-stroke{signature: @sig, stroke: @smoothed}
    @current-stroke <- nil
  }

  @on-save{
    @normalized <- @sig/normalize{signature: @sig, target-w: 1000, target-h: 1000}
    @img <- @sig/render{signature: @normalized, width: 500, height: 200,
                          max-width: 6, bg: white, ink: black}
    @vault/save{path: @output-path, image: @img}
  }

  @on-verify (@reference-sig){
    @norm-input <- @sig/normalize{signature: @sig, target-w: 1000, target-h: 1000}
    @norm-ref <- @sig/normalize{signature: @reference-sig, target-w: 1000, target-h: 1000}
    @distance <- @sig/similarity-distance{a: @norm-input, b: @norm-ref}
    (if (< @distance 50000) @ok "no match")
  }
}
```

## Structures

```innate
(defstruct point
  x         : Int
  y         : Int
  pressure  : Int     ;; 0..1024
  t-ms      : Int)

(defstruct stroke
  points : [Point])

(defstruct signature
  strokes : [Stroke])
```

## Resolver Natives

```innate
@sig/new-signature{}                               -> Signature
@sig/add-stroke{signature, stroke}                 -> Unit
@sig/point-count{signature}                        -> Int
@sig/bounding-box{signature}                       -> (Int, Int, Int, Int)?
@sig/catmull-rom{p0, p1, p2, p3, t}                -> (Int, Int)
@sig/smooth-stroke{stroke, steps}                  -> Stroke
@sig/normalize{signature, target-w, target-h}      -> Signature
@sig/similarity-distance{a, b}                     -> Int
@sig/render{signature, width, height, max-width, bg, ink} -> Image
```

## Demo

```innate
(@demo){
  @sig <- @sig/new-signature{}
  @sig/add-stroke{signature: @sig, stroke: {
    points: [{x: 10, y: 20, pressure: 512, t-ms: 0},
             {x: 30, y: 40, pressure: 768, t-ms: 100},
             {x: 50, y: 30, pressure: 1024, t-ms: 200},
             {x: 70, y: 50, pressure: 512, t-ms: 300}]}}
  @sig/add-stroke{signature: @sig, stroke: {
    points: [{x: 80, y: 30, pressure: 600, t-ms: 400},
             {x: 100, y: 50, pressure: 700, t-ms: 500}]}}

  @sig/point-count{signature: @sig}       ;; -> 6
  @sig/bounding-box{signature: @sig}      ;; -> (10, 20, 100, 50)

  @smoothed <- @sig/smooth-stroke{stroke: @sig.strokes[0], steps: 5}
  (length @smoothed.points)               ;; -> 7

  @norm <- @sig/normalize{signature: @sig, target-w: 1000, target-h: 1000}
  @sig/bounding-box{signature: @norm}     ;; -> (0, 0, 999, 999)
  ;; 999 not 1000 because (100-10)*11111/1000 = 999 (integer rounding)

  @sig/catmull-rom{p0: (0, 0), p1: (10, 10), p2: (20, 10), p3: (30, 0), t: 0}
  ;; -> (10, 10)  (passes through p1)
  @sig/catmull-rom{p0: (0, 0), p1: (10, 10), p2: (20, 10), p3: (30, 0), t: 1000}
  ;; -> (20, 10)  (passes through p2)

  ;; Same shape, different size → similarity 0 after normalize
  @small <- @sig-from [(0,0), (10,0), (10,10), (0,10)]
  @big   <- @sig-from [(0,0), (100,0), (100,100), (0,100)]
  @sig/similarity-distance{
    a: @sig/normalize{sig: @small, target-w: 1000, target-h: 1000},
    b: @sig/normalize{sig: @big,   target-w: 1000, target-h: 1000}
  }   ;; -> 0
}
```

## Where

Pressure MUST be per-point, not per-stroke — pen dynamics vary within a single stroke; the width is the feature. Timestamp MUST be captured — signature verification uses stroke timing, not just geometry. Catmull-Rom MUST be integer-scaled — float polynomial evaluation diverges across languages; Rosetta Stone demands byte-identity. Smoothing MUST preserve endpoints — pen-up/pen-down positions are signal; interpolating through them destroys user intent. Variable-width rendering MUST use disk stamps — pixel-perfect variable-width with anti-aliasing is language-specific; disks are integer-only. Normalization MUST be translate-then-scale — scale-then-translate produces different numbers due to integer division. Similarity MUST be computed after normalization — comparing raw coordinates across devices measures pad size, not signature shape. The nearest-point metric MUST use squared distance — the square root is a float; comparison thresholds scale accordingly.
