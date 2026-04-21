# G130 — Turtle Graphics

> The Rosetta Stone's seventeenth and **final** Graphics project, closing the 130-project milestone. Models **Logo's turtle** — a programmable cursor that moves and turns in a 2D plane, drawing lines as it goes. The distinctive move: **every detail is designed for cross-language byte-identity** — integer-degree heading (0..=359), integer-scaled sin/cos lookup table (values × 10000), vector-line canvas instead of pixel rasterization, state stack for nested procedures, command-ADT-as-program. The whole milestone's discipline converges here: no floats, pure functions, ADT-as-data, stateful interpreters over immutable programs.

```yaml
id: G130
title: Turtle Graphics
category: graphics
requires: [G115-mind-mapper, G119-bulk-picture, G127-signature-maker]
provides: [integer-degree-heading, integer-sin-cos-table, command-interpreter, state-stack, vector-canvas]
```

## Insight: Integer-Degree Heading with Modulo Wrap

Heading is `u16 ∈ 0..=359`. Turn by `d` degrees: `heading = ((heading + d) % 360 + 360) % 360`. The double-modulo handles negative `d`. SetHeading to a specific absolute angle: `heading = ((h % 360) + 360) % 360`. No floats; no radians; 360 exact values.

First Rosetta Stone project where **a cyclic coordinate (heading) uses exact integer degrees** with modulo wrap. G122's time-of-day was modulo on a monotonic clock; G130's heading is modulo on a turn-accumulator.

## Insight: Integer-Scaled Sin/Cos Table Replaces Trig

Floats are forbidden (cross-language byte-identity). Solution: a 91-entry lookup table of `sin(0..=90) × 10000`, values precomputed and baked in. The other three quadrants are derived by sign flips:

```
0..=90   → table[a]
91..=180 → table[180 - a]
181..=270 → -table[a - 180]
271..=359 → -table[360 - a]
```

`cos(a) = sin((a + 90) mod 360)` — one table does both. Values match exactly across all six languages: `sin(45) = 7071`, `sin(90) = 10000`, `sin(270) = -10000`.

First Rosetta Stone project where **trig is precomputed as an integer-scaled table** for cross-language determinism. G115 used floats for radial layout (accepted the rounding difference); G130 refuses to.

## Insight: Command-ADT-as-Program

```
Command::{Forward(n), Backward(n), Turn(deg), SetHeading(h),
          PenUp, PenDown, SetColor(rgb), SetPenWidth(w),
          PushState, PopState, GoTo(x, y)}
```

A program is `Vec<Command>`. Execution is a fold: `canvas.execute(program)` steps through commands, mutating the turtle and accumulating lines. Programs can be serialized, composed, concatenated, replayed. The Logo "procedure" construct is just `Vec<Command>` at a variable name; calling a procedure is appending its commands.

Same pattern as G119's transform pipeline — program as data, interpreter as fold. Applied at a different domain (turtle rather than image), with state (turtle position) not just input-output.

First Rosetta Stone project where **a program-as-data interpreter drives stateful computation**. G119 was stateless (each op took buffer → buffer); G130 is stateful (each command mutates canvas + turtle).

## Insight: State Stack Enables Logo-Style Subroutines

`PushState` snapshots the turtle (position, heading, pen state). `PopState` restores it. A "subroutine" that draws a leaf then returns to the branch point looks like:

```
Forward(50)         # to branch point
PushState           # save here
Turn(45)
Forward(20)         # draw leaf
PopState            # back to branch point
Forward(30)         # continue main stem
```

Lines from the branch are preserved; only the turtle's identity-state is restored. Nested Push/Pop is a proper stack — a Push inside a Push gives two saved states, popped in LIFO order.

First Rosetta Stone project where **a state stack enables return-to-origin control flow** in a stateful interpreter. G124's breadcrumbs were also a stack, but over passive navigation state; G130's stack saves active computation state.

## Insight: Vector-Line Canvas, Not Pixel Buffer

Every prior graphics project rasterized: G116's pixel grid, G119's transforms, G123's annotations, G129's LSB-embedded pixels. G130 keeps lines as **abstract geometry** — `Line { x1, y1, x2, y2, color, width }` records — for the entire program. Rasterization happens only at `render()`, and only if you want to.

Benefits:
- Zoom/scale is free (scale the coordinates, re-render).
- SVG export is trivial (each `Line` is an `<line>` element).
- Line count is a natural metric (a square has 4 lines, a star has 5).
- The turtle's path is analyzable as geometry.

First Rosetta Stone project with a **vector graphics canvas**. Prior projects lived in raster space; G130 lives in vector space.

## Insight: Pen State Is Part of the Turtle

`pen_down: bool` gates whether `Forward`/`Backward`/`GoTo` emit a line. `pen_color` and `pen_width` are captured per-line at emission time — changing color after emitting a line doesn't retroactively recolor it. This is the classic Logo semantic: commands act immediately, not deferred.

First Rosetta Stone project where **per-emission snapshotting of configuration state** is the semantic (contrast with G123 where the annotation pipeline is explicitly an ordered list).

## Insight: Square Closure as a Determinism Test

A turtle that does `(Forward(100), Turn(90)) × 4` should return to its starting point with heading 0. With integer-scaled trig, this holds exactly at 0°/90°/180°/270°: cos/sin are ±10000 or 0, so Forward×10000/10000 is exact.

At 45° rotation, `cos(45) = sin(45) = 7071`, so `100 × 7071 / 10000 = 70` (integer division). A rotated-45° square closes to within ±2 pixels of origin — a small drift from the 0.000_something-lost-per-step rounding. Tests assert this bound explicitly.

First Rosetta Stone project where **cumulative rounding drift is bounded and asserted**. G119 had `Rotate90^4 = identity`; G130 has "closed polygon at axis-aligned angles; bounded drift otherwise".

## Choreographic Case: Vault Turtle Garden

```innate
(@vault-turtle-garden){
  ;; Define a procedure: branch (recursive tree)
  @branch <- (fn (@depth @length)
    (if (> @depth 0)
        [@tl/forward{n: @length},
         @tl/push-state,
         @tl/turn{deg: -30},
         (@branch (- @depth 1) (/ @length 2)),
         @tl/pop-state,
         @tl/push-state,
         @tl/turn{deg: 30},
         (@branch (- @depth 1) (/ @length 2)),
         @tl/pop-state]
        []))

  @canvas <- @tl/new-canvas{}
  @tl/execute{canvas: @canvas,
               program: [@tl/turn{deg: 90},
                          ...(@branch 5 100)]}

  @svg <- @tl/export-svg{canvas: @canvas}
  @vault/save{path: "tree.svg", content: @svg}
}
```

A fractal tree drawn by a recursive procedure — the state stack is what makes recursion work without losing the trunk position.

## Structures

```innate
(defstruct turtle
  x          : Int
  y          : Int
  heading    : Int    ;; 0..=359
  pen-down   : Bool
  pen-color  : Rgb
  pen-width  : Int)

(defstruct line
  x1 : Int  y1 : Int  x2 : Int  y2 : Int
  color : Rgb  width : Int)

(defenum command-kind
  FORWARD | BACKWARD | TURN | SET_HEADING
  | PEN_UP | PEN_DOWN | SET_COLOR | SET_PEN_WIDTH
  | PUSH_STATE | POP_STATE | GOTO)

(defstruct canvas
  lines       : [Line]
  turtle      : Turtle
  state-stack : [Turtle])
```

## Resolver Natives

```innate
@tl/sin-table{angle}                        -> Int    ;; × 10000
@tl/cos-table{angle}                        -> Int    ;; × 10000
@tl/new-canvas{}                             -> Canvas
@tl/execute{canvas, program}                 -> Unit
@tl/step{canvas, command}                    -> Unit
@tl/forward{n}  ...                          -> Command (constructors)
@tl/render{canvas, width, height, bg}        -> Image
```

## Demo

```innate
(@demo){
  @tl/sin-table{angle: 0}      ;; -> 0
  @tl/cos-table{angle: 0}      ;; -> 10000
  @tl/sin-table{angle: 90}     ;; -> 10000
  @tl/sin-table{angle: 45}     ;; -> 7071   (= cos(45))

  ;; Square
  @c <- @tl/new-canvas{}
  @tl/execute{canvas: @c, program: [
    @tl/forward{n: 100}, @tl/turn{deg: 90},
    @tl/forward{n: 100}, @tl/turn{deg: 90},
    @tl/forward{n: 100}, @tl/turn{deg: 90},
    @tl/forward{n: 100}, @tl/turn{deg: 90}
  ]}
  @c.turtle.x           ;; -> 0
  @c.turtle.y           ;; -> 0
  @c.turtle.heading     ;; -> 0
  (length @c.lines)     ;; -> 4

  ;; State stack: branch off, come back
  @c2 <- @tl/new-canvas{}
  @tl/execute{canvas: @c2, program: [
    @tl/forward{n: 50},
    @tl/push-state,
    @tl/turn{deg: 90}, @tl/forward{n: 30},
    @tl/pop-state
  ]}
  @c2.turtle.x          ;; -> 50  (back where we branched)
  @c2.turtle.heading    ;; -> 0
  (length @c2.lines)    ;; -> 2   (trunk + branch both drawn)

  ;; 45-degree rotated square: bounded drift
  @c3 <- @tl/new-canvas{}
  @tl/execute{canvas: @c3, program: [
    @tl/turn{deg: 45},
    @tl/forward{n: 100}, @tl/turn{deg: 90},
    @tl/forward{n: 100}, @tl/turn{deg: 90},
    @tl/forward{n: 100}, @tl/turn{deg: 90},
    @tl/forward{n: 100}, @tl/turn{deg: 90}
  ]}
  @c3.turtle.x          ;; -> -2  (bounded rounding drift at 45°)
  @c3.turtle.y          ;; -> -2
}
```

## Where

Heading MUST be integer degrees — radians require floats; 360 exact values suffice for any visible drawing. Sin/cos MUST be a lookup table — float trig diverges across languages, defeating byte-identity. Modulo wrap MUST use `((x % 360) + 360) % 360` — single `%` produces negative results for negative inputs in most languages. Program MUST be a `Vec<Command>` — program-as-data enables serialization, composition, replay; method-chain interpreters cannot. State stack MUST be a proper LIFO — FIFO or "only one save" breaks nested procedures; a real stack is the Logo semantic. Lines MUST be vector records — rasterizing eagerly throws away geometry, blocks scaling and SVG export, and prevents "count the lines" as a correctness metric. Pen state MUST be snapshotted per-line at emission time — deferred binding makes every line retroactively recolored when you change the pen; immediate binding is the Logo semantic. Square closure at axis-aligned angles MUST be exact — integer-scaled trig at 0°/90°/180°/270° has no rounding (cos/sin are ±10000 or 0). Off-axis closure is allowed bounded drift — a test asserts `|x| ≤ 2, |y| ≤ 2` for a 45°-rotated square. The six languages MUST produce byte-identical line lists for the same program — this is the milestone's whole point.
