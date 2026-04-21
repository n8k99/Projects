# G123 — Screen Capture Program

> The Rosetta Stone's tenth **Graphics project**. Models the **screenshot tool** every OS ships (macOS Grab / Windows Snipping Tool / grim / flameshot): a region spec (full screen / rectangle / window), an optional delay countdown, an annotation overlay layer (rectangles, highlights, arrows, text), and a ring-buffer history of past captures. The distinctive move: **the capture intent is data, separate from the act of capture**. You arm with a spec; the recorder is a small FSM that advances through countdown and then performs the pure transformation screen → capture.

```yaml
id: G123
title: Screen Capture Program
category: graphics
requires: [G116-grayscale-converter, G119-bulk-picture, G122-wallpaper-manager]
provides: [capture-region-adt, delayed-capture-fsm, annotation-overlays, ring-buffer-history]
```

## Insight: Capture Region Is a Closed ADT

`CaptureRegion::{FullScreen, Rect(x,y,w,h), Window(id)}`. Three shapes cover every screenshot tool's feature set. The act of capture is a pure function `(screen, region, window_lookup) → Image` — `FullScreen` clones, `Rect` crops, `Window` dispatches through a lookup function.

First Rosetta Stone project where **a target selection is encoded as an ADT with per-variant payload**. G120's disc state was an ADT, but its variants were uniform; G123's region variants carry geometry data specific to each shape.

## Insight: Delayed Capture Is a Five-State FSM

`Idle → Armed → Counting → Capturing → Captured`. Arming with `delay_ms > 0` enters `Counting`; with `delay_ms == 0` goes straight to `Capturing`. `tick(ms)` drains the countdown; when it hits 0, state transitions to `Capturing`. `capture(screen)` executes only in `Capturing` state.

Why a separate FSM when countdown could be a number? Because "I armed and the countdown hasn't started yet" is a distinct state from "I haven't armed at all" — the UI shows different things. Making every transition explicit prevents "is the countdown running?" ambiguity.

First Rosetta Stone project with a **five-state FSM centered on a countdown**. G117 had a playback FSM driven by buffer levels; G123's FSM is driven by a single scalar ticking down.

## Insight: Annotations Are an Overlay Pipeline, Not a Transform

G119's pipeline was `Image → Op → Image` (transformation: resize, rotate, filter). G123's annotations are **overlays**: `Image × Annotation → Image` where the base image is unchanged structurally, only *mutated at specific pixels*.

```
apply_annotations(base, [highlight, rect, arrow]) =
    arrow(rect(highlight(base)))
```

The distinction matters:
- Transforms can change dimensions (resize, crop, rotate-90). Overlays cannot.
- Overlays have *spatial targeting* (rectangle at (x,y,w,h)); transforms operate on the whole image.
- Overlays compose by paint-order (later draws over earlier). Transforms compose by functional composition.

First Rosetta Stone project where **the difference between "transform" and "overlay" is made explicit as a different data/operation shape**. Both are lists of ops applied in sequence, but they're not interchangeable.

## Insight: Highlights Are Alpha-Blended, Others Are Opaque

`Highlight { color, opacity }` blends: `out = blend(base, color, opacity)` channel-wise. Every other annotation (rectangle outline, arrow, text) is opaque — the pixel becomes the annotation color exactly.

This matches user intent: a "highlight" should preserve the underlying content (dimmed/tinted), while a "rectangle" is a marker that shouldn't be ambiguous. Alpha blending is per-channel: `(base * (255 - alpha) + color * alpha) / 255`.

First Rosetta Stone project where **some overlays are transparent and others aren't**, and the two are modeled as different annotation variants rather than one variant with `opacity: 255` meaning "opaque".

## Insight: Ring-Buffer History Evicts Oldest

`history: Vec<Capture>`, `history_max: usize`. After each capture, if `len > max`, pop the oldest and emit an `Evicted` event. Users see "last N screenshots" with bounded memory.

The event is important — UI can animate the eviction ("oldest screenshot faded out"), telemetry can count evictions (a burst of captures vs steady usage), tests can assert bounded behavior.

First Rosetta Stone project with **fixed-size oldest-evict semantics as first-class state**. G094's append-only logs were unbounded; G123's history is bounded with explicit eviction events.

## Insight: Bresenham's Line Algorithm

`Annotation::Arrow { x1, y1, x2, y2 }` draws the line via Bresenham — integer-only, no divisions, no float rounding, deterministic across languages. Every RasterOp library ships this: it's the canonical "draw a line from A to B on a pixel grid".

Same implementation across all six languages. Pixel results match byte-for-byte — the algorithm has no language-specific variance.

First Rosetta Stone project to use **Bresenham's line algorithm**. G115's radial layout used floats; G123 uses integer-only math for determinism.

## Insight: Window Lookup Is Dependency-Injected

`capture(screen, window_lookup)` takes a `Fn(window_id) -> Option<Image>` — the window's current pixel buffer is not stored in the recorder; it's fetched at capture time. This lets a Window capture reflect the current window state (not a stale copy from when the spec was created), and lets tests inject mock windows.

First Rosetta Stone project where **a capture-time resolver is injected as a function parameter**. G117 used a stream attach; G123 injects a lookup that could return different results each call.

## Choreographic Case: Vault Screenshot Workflow

```innate
(@vault-screenshot){
  @rec <- @cap/new-recorder{history-max: 10}

  @on-hotkey-shift-3{
    @spec <- {region: {kind: "full-screen"}, delay-ms: 0, include-cursor: true}
    @cap/arm{recorder: @rec, spec: @spec}
    @capture <- @cap/capture{recorder: @rec,
                               screen: @wayland/current-screen{},
                               window-lookup: @wayland/get-window}
    @vault/save{path: "/screenshots/${@capture.captured-at-ms}.png",
                  image: @capture.image}
  }

  @on-hotkey-shift-4{
    @region <- @ui/prompt-for-rect{}
    @spec <- {region: {kind: "rect", x: @region.x, y: @region.y,
                        width: @region.width, height: @region.height},
               delay-ms: 3000,
               annotations: [
                 {kind: "rectangle", x: 0, y: 0,
                  width: @region.width, height: @region.height,
                  color: "#FF0000", stroke: 2}
               ]}
    @cap/arm{recorder: @rec, spec: @spec}
    @ui/show-countdown{recorder: @rec}
  }

  @on-countdown-tick (@elapsed-ms){
    @cap/tick{recorder: @rec, ms: @elapsed-ms}
    (when (@rec.state == "capturing"){
      @cap/capture{recorder: @rec,
                    screen: @wayland/current-screen{},
                    window-lookup: @wayland/get-window}
    })
  }
}
```

Hotkey triggers arm a spec; ticks advance the countdown; capture fires when ready; the image goes to the vault with full annotation data preserved.

## Structures

```innate
(defenum region-kind FULL_SCREEN | RECT | WINDOW)

(defstruct capture-region
  kind      : RegionKind
  x         : Int
  y         : Int
  width     : Int
  height    : Int
  window-id : Int)

(defenum annotation-kind RECTANGLE | HIGHLIGHT | TEXT | ARROW)

(defstruct annotation
  kind     : AnnotationKind
  x        : Int
  y        : Int
  width    : Int
  height   : Int
  x1       : Int
  y1       : Int
  x2       : Int
  y2       : Int
  color    : Rgb
  stroke   : Int
  opacity  : Int
  text     : String)

(defstruct capture-spec
  region         : CaptureRegion
  delay-ms       : Int
  include-cursor : Bool
  annotations    : [Annotation])

(defenum recorder-state IDLE | ARMED | COUNTING | CAPTURING | CAPTURED)

(defstruct capture
  id             : Int
  spec           : CaptureSpec
  image          : Image
  captured-at-ms : Int)

(defstruct recorder
  state                   : RecorderState
  pending-spec            : CaptureSpec?
  pending-id              : Int
  countdown-remaining-ms  : Int
  history                 : [Capture]
  history-max             : Int
  elapsed-ms              : Int
  events                  : [RecorderEvent])
```

## Resolver Natives

```innate
@cap/new-recorder{history-max}                    -> Recorder
@cap/arm{recorder, spec}                          -> Int (capture id)
@cap/tick{recorder, ms}                           -> Unit
@cap/capture{recorder, screen, window-lookup}     -> Capture?
@cap/reset{recorder}                              -> Unit
@cap/apply-annotations{image, annotations}        -> Image
```

## Demo

```innate
(@demo){
  @rec <- @cap/new-recorder{history-max: 3}
  @cap/arm{recorder: @rec,
            spec: {region: {kind: "full-screen"}, delay-ms: 1000}}
  @rec.state                          ;; -> COUNTING
  @rec.countdown-remaining-ms         ;; -> 1000

  @cap/tick{recorder: @rec, ms: 400}
  @rec.countdown-remaining-ms         ;; -> 600

  @cap/tick{recorder: @rec, ms: 700}
  @rec.state                          ;; -> CAPTURING

  @screen <- @make-gray-image{10, 8, color: {r: 100, g: 100, b: 100}}
  @cap <- @cap/capture{recorder: @rec, screen: @screen,
                         window-lookup: (fn (_) nil)}
  @cap.image.width     ;; -> 10
  @cap.image.height    ;; -> 8

  @cap/reset{recorder: @rec}
  @spec <- {region: {kind: "rect", x: 2, y: 1, width: 6, height: 5},
             annotations: [
               {kind: "highlight", x: 0, y: 0, width: 6, height: 5,
                color: {r: 255, g: 255, b: 0}, opacity: 128},
               {kind: "rectangle", x: 1, y: 1, width: 4, height: 3,
                color: {r: 255, g: 0, b: 0}, stroke: 1},
               {kind: "arrow", x1: 0, y1: 0, x2: 5, y2: 4,
                color: {r: 0, g: 255, b: 0}}
             ]}
  @cap/arm{recorder: @rec, spec: @spec}
  @cap2 <- @cap/capture{recorder: @rec, screen: @screen,
                          window-lookup: (fn (_) nil)}
  @cap2.image.pixels[0]   ;; -> {r: 0, g: 255, b: 0}  (arrow start)

  ;; Ring buffer: 5 more captures keep only last 3
  (repeat 5 @cap/arm-and-capture{...})
  @rec.history.length      ;; -> 3 (constant)
  (map (fn (c) c.id) @rec.history)   ;; -> [5, 6, 7]  (oldest evicted)
}
```

## Where

Region MUST be a closed ADT — FullScreen / Rect / Window cover every screenshot tool's feature set; an open "bring your own" extension breaks the pure-function-of-region semantics. FSM MUST be five states with explicit transitions — conflating Idle and Counting produces ambiguous UX. Countdown MUST drain via `tick(ms)` — a real timer is non-deterministic; tests need atomic advancement. Annotations MUST be overlays, not transforms — they preserve base dimensions and compose by paint-order, not functional composition. Highlight MUST alpha-blend — a "highlight" that replaces underlying content is a Rectangle, not a Highlight; these are distinct user intents. Rectangle, Arrow, Text MUST be opaque — a faint rectangle/arrow is ambiguous UX. History MUST be a fixed-size ring — unbounded memory growth is unacceptable; explicit eviction events let UI and telemetry observe boundedness. Window capture MUST use a lookup function — window pixel buffers change; stale buffers produce stale screenshots. Bresenham MUST be used for lines — float math has language-specific variance; the whole point of Rosetta Stone is byte-identical output across languages.
