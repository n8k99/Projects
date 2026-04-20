# G119 — Bulk Picture Manipulator

> The Rosetta Stone's sixth **Graphics project**. Models the **transformation pipeline** every batch photo tool ships (ImageMagick, Lightroom's sync, Preview's "batch process"): a recipe of operations applied to each image in a collection. The distinctive move: the pipeline is **data, not code** — a list of `Op` values. Same recipe on same image always yields the same output. Batch application **isolates errors per-image**; dry-run **predicts dimensions** without touching pixels.

```yaml
id: G119
title: Bulk Picture Manipulator
category: graphics
requires: [G068-bulk-thumbnail, G092-bulk-renamer, G116-grayscale-converter]
provides: [transformation-pipeline, pipeline-as-data, dry-run-preview, per-image-error-isolation]
```

## Insight: Pipeline Is Data, Not Code

A pipeline is `list[Op]` where `Op` is a closed enum/ADT: Resize, Grayscale(method), Rotate90/180/270, FlipH/V, Crop, Brightness(delta), Contrast(factor). This is the **recipe-as-value** pattern. Two pipelines with identical ops are equal by value; pipelines can be serialized, cached, diffed, replayed, or sent over the wire.

Contrast with a "fluent-chain" API where `.grayscale().rotate(90).crop(...)` is *code* that executes immediately — you can't inspect, snapshot, or compare it.

First Rosetta Stone project where **a user-facing workflow is encoded as a data structure**, not as a method chain. G104's macro system had command lists; G119 has transformation lists. Both are programs-as-data, but G119 is the first where the *user* composes the program via UI.

## Insight: Ops Are Pure

`apply_op(img, op) -> Result<Image, OpError>` — no side effects, deterministic, testable in isolation. The pipeline is a fold: `apply_pipeline = images.foldl(apply_op)`. Nothing more.

First Rosetta Stone project where **the entire operational surface is `A → Op → Result<A>`**. G094 had pure log appends; G119 has pure image transforms. The difference: G119's result type can be the *same shape* as the input (Image → Image), so pipelines compose trivially.

## Insight: Dry-Run Predicts Dimensions Without Running

Every Op has a well-defined effect on image dimensions (resize sets new dims; rotate 90/270 swaps them; crop narrows them; grayscale/brightness/contrast leave them unchanged). `dry_run_dims(start_dims, pipeline)` computes the dimensions after each step **without ever touching pixels**.

Why it matters: UIs can preview "what will this pipeline produce" before running on a 4K image; bad crops fail at dry-run, not during execution.

First Rosetta Stone project where **a pipeline's metadata effect is computable independently of its data effect**. G111's topo sort had cycle detection as metadata; G119 has dimension lineage.

## Insight: Batch Isolates Per-Image Errors

`run_batch(images, pipeline) -> [BatchResult]` — one bad image doesn't abort the batch. Each input yields an `Ok(image)` or `Err(error_reason)` tagged with its index. Callers can filter, retry, or report.

Contrast with a "fail-fast" batch: one bad image → exception → rest of the batch untouched. For 200 photos, that's the difference between "fix one and retry" and "restart from scratch".

First Rosetta Stone project where **errors are first-class values in a batch result**, not exceptions that terminate the loop. G098's bulk copier had per-file events; G119's batch result is the same pattern applied to pure transforms.

## Insight: Op Order Matters

`Brightness(50) → Contrast(2.0)` ≠ `Contrast(2.0) → Brightness(50)`. Contrast uses 128 as the pivot; brightness is an additive shift. Applying the shift first moves pixels away from 128, so contrast amplifies the moved values differently than it would amplify the pre-shifted ones.

This isn't a bug — it's the shape of the op algebra. Pipelines are **non-commutative**. Users need to know this (most photo editors document it); tests assert it (`pipeline_order_matters`).

First Rosetta Stone project that **exposes non-commutativity of ops as a property to be asserted**. G107 had budget-variance; G119 has pipeline-order-variance. Both are "the specific numeric result depends on sequencing".

## Insight: Four Rotate-90s Is Identity; Two Flips Is Identity

Each rotation op has a mathematical identity: `Rotate90^4 = id`, `Rotate180^2 = id`, `FlipH^2 = id`, `FlipV^2 = id`. Tests exploit this — any implementation bug in the rotation math shows up as a failed identity check, without needing pixel-exact oracles.

First Rosetta Stone project where **algebraic identities serve as self-verifying test oracles**. G096's LCG was tested via "same seed = same output"; G119's rotates are tested via "doing it `n` times = original".

## Choreographic Case: Vault Photo Batch

```innate
(@vault-photo-batch){
  @photos <- @vault/query{type: "image", tag: "unprocessed"}

  @pipeline <- [
    @bulk/op-resize{width: 1920, height: 1080},
    @bulk/op-grayscale{algo: "luminosity"},
    @bulk/op-brightness{delta: 10},
  ]

  @preview <- @bulk/dry-run-dims{start: @first-photo.dims, pipeline: @pipeline}
  @ui/show-preview{steps: @preview}

  @on-user-confirms{
    @results <- @bulk/run-batch{images: @photos, pipeline: @pipeline}
    (for @r in @results{
      (if @r.ok @vault/save{image: @r.output, replace: @photos[@r.index]}
              @ui/log-error{index: @r.index, error: @r.error})
    })
  }
}
```

The vault's batch-processing shell is a thin wrapper: user builds a recipe (pipeline), previews dims, confirms, runs, handles per-image results. The pipeline is the unit of work — it can be saved as a "recipe" vault note for reuse.

## Structures

```innate
(defenum op-kind
  RESIZE | GRAYSCALE | ROTATE_90 | ROTATE_180 | ROTATE_270
  | FLIP_H | FLIP_V | CROP | BRIGHTNESS | CONTRAST)

(defstruct op
  kind               : OpKind
  width              : Int
  height             : Int
  x                  : Int
  y                  : Int
  algorithm          : Algorithm
  brightness-delta   : Int
  contrast-factor    : Float)

(defenum op-error CROP_OUT_OF_BOUNDS | ZERO_DIMENSION)

(defstruct batch-result
  index    : Int
  output   : Image?    ;; present on success
  error    : OpError?  ;; present on failure
)
```

## Resolver Natives

```innate
@bulk/op-resize{width, height}                    -> Op
@bulk/op-grayscale{algo}                           -> Op
@bulk/op-rotate-90{} | @bulk/op-rotate-180{} | @bulk/op-rotate-270{}
@bulk/op-flip-h{} | @bulk/op-flip-v{}
@bulk/op-crop{x, y, width, height}                 -> Op
@bulk/op-brightness{delta}                         -> Op
@bulk/op-contrast{factor}                          -> Op
@bulk/apply-op{image, op}                          -> Image | OpError
@bulk/apply-pipeline{image, pipeline}              -> Image | OpError
@bulk/run-batch{images, pipeline}                  -> [BatchResult]
@bulk/dry-run-dims{start, pipeline}                -> [(Int, Int)] | OpError
```

## Demo

```innate
(@demo){
  @img <- @make-checker{width: 4, height: 4}
  @pipeline <- [
    @bulk/op-rotate-90{},
    @bulk/op-brightness{delta: 10},
    @bulk/op-grayscale{algo: LUMINOSITY},
  ]

  @out <- @bulk/apply-pipeline{image: @img, pipeline: @pipeline}
  @dims <- @bulk/dry-run-dims{start: (4, 4), pipeline: @pipeline}
  ;; dims -> [(4,4), (4,4), (4,4), (4,4)]  (rotate swaps but checker is square)
  ;; out.pixels[0] -> (107, 107, 107) — deterministic across all six languages

  @batch <- [@filled{6, 6, gray128}, @filled{2, 2, gray128}]
  @crop-pipe <- [@bulk/op-crop{x: 0, y: 0, width: 3, height: 3}]
  @results <- @bulk/run-batch{images: @batch, pipeline: @crop-pipe}
  ;; results[0] -> ok: 3x3
  ;; results[1] -> err: CROP_OUT_OF_BOUNDS
}
```

## Where

Pipeline MUST be data — recipes need to be serializable, comparable, previewable; method-chain APIs forbid this. Ops MUST be pure — `apply_op(img, op)` with no side effects is the only way to make batch processing deterministic and testable. Dry-run MUST compute dimensions without pixels — UIs need cheap preview on large images; failing on bad crops during dry-run beats failing during execution. Batch MUST isolate errors per image — one bad image in 200 shouldn't force a restart; each result is independent. Op order MUST be treated as significant — non-commutativity is real (brightness+contrast is order-dependent); UIs must show the ordered list. Rotations and flips MUST satisfy algebraic identities — `rotate90^4 = id`, `flip^2 = id`; these self-verify the implementation without needing pixel oracles. `Op` MUST be a closed enum — open extension means new code paths not in the pipeline-as-data contract; if a new op is needed, add it to the enum.
