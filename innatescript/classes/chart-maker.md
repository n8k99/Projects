# G058 — Chart Making Class / API

> The chart is a **function**, not a container. Five composable stages — data → scale → encode → layout → render. The renderer is a pluggable backend.

```yaml
id: G058
title: Chart Making Class / API
category: classes
requires: [G057-big-integer, G055-recipe-manager]
provides: [pipeline-as-class, scale-as-function, declarative-spec, pluggable-backend]
```

## Insight: The Chart Is a Function, Not a Product

Traditional OO charts have a `Chart` class with `add_series`, `set_color`, `draw`. State accumulates. Mutation everywhere. G058 takes the grammar-of-graphics approach: the chart is a *specification* + *data*; rendering is a pure function from that pair to an output string.

```
render : (Spec, Dataset) → Rendering
```

No hidden state. No incremental mutation. The same spec + data always produces the same output. This is how Vega, Vega-Lite, D3, and ggplot2 work. It separates the *what* from the *how* and makes charts composable: combine two specs into a layered chart; parameterize a spec over a dataset to get small multiples; swap one renderer for another to change output format.

Every prior Classes project was a container with state: products accumulate in inventory, rentals accumulate in a movie store, appointments accumulate in a scheduler. G058 inverts this — the spec is static, the data flows through, the output is computed. The class is a **pipeline definition**, not a state machine.

## Insight: Scales Are Functions, Made Explicit

The core primitive of data visualization is the **scale**: a function from a data domain to a visual range. `LinearScale(0, 100, 0, 40)` is literally a function — feed it `50`, get `20`. The class exists to *name the function* and carry its parameters alongside an application operator.

```
LinearScale.at(50) = 0 + (50 - 0) / (100 - 0) * (40 - 0) = 20
```

Three scale variants cover most charts:

| Scale | Domain | Range | Use |
|---|---|---|---|
| Linear | continuous numbers | continuous pixels | positions, sizes |
| Ordinal | discrete categories | evenly spaced positions | categorical axes |
| Log | positive numbers | pixels | orders-of-magnitude data |

Every visualization question reduces to "which scales, on which fields?" The chart spec is the binding — `y = linear_scale(temperature) on height_range`. This is the first project in the Rosetta Stone where the domain model is explicitly **higher-order**: the spec contains functions, not just values.

The noosphere will use scales constantly. Project progress maps 0–100% onto a visual bar; a pace_check score maps [-1, +1] onto a color (red-green). Every dashboard widget is a chart spec in miniature: data source, a scale or two, a visual encoding, a renderer.

## Insight: Each Stage Is Independently Testable

The pipeline decomposes into five pure functions:

1. **Data** — the dataset. A list of points or rows. Testable: does it parse correctly?
2. **Scale** — domain → range. Testable: does `scale.at(domain_min) == range_min`? Does linear scaling actually compose linearly?
3. **Encode** — field → visual channel. Testable: does a 100-value row produce a 100-unit bar?
4. **Layout** — positioned primitives on a canvas. Testable: do axes, labels, and bars occupy non-overlapping regions?
5. **Render** — primitives → output string/SVG/canvas. Testable: does the output contain the expected marks?

Every prior Classes project had a monolithic `make_happen` method — schedule the appointment, record the grade, check out the book. G058 is the first project where the operation **decomposes into stages you can test in isolation**. If the chart looks wrong, you can pinpoint the bad stage: bad data, bad scale, bad encoding, bad layout, or bad renderer.

This generalizes to every pipeline in the noosphere. A choreography has stages (parse, validate, project, execute, report). A build has stages (fetch, compile, test, package, deploy). Each stage is independently testable if it's a pure function. G058 is the Rosetta Stone's first *composed-pipeline* class, and the pattern scales up without modification.

## Insight: Declarative Spec vs Imperative Drawing

The chart spec doesn't say "draw a rectangle at (100, 200) with width 50, height 80, filled with blue." It says "bar chart with `x = category`, `y = value`". The renderer figures out the rectangles, widths, colors.

This is the first separation in the Rosetta Stone between **what to show** and **how to draw it**. The noosphere needs this separation everywhere. A daily-note template says "list today's completed tasks" (declarative); the renderer decides checkbox style, font, spacing. A project summary says "show blockers, goals, current context" (declarative); the renderer decides whether it's a card, a table, an expanded section.

Declarative specs are **interchangeable** — swap the renderer and the same spec produces SVG, PDF, or a printed receipt. G058's ASCII output is one backend; a terminal with box-drawing characters is another; an HTML `<canvas>` is another. The spec does not change.

## Insight: The Renderer Is Pluggable

We render to ASCII here because every target language has a string type and a terminal. The same `BarChart` or `LineChart` spec could render to:

- **SVG** — write `<rect>` elements, wire up the scales to viewport pixels.
- **Canvas** — `ctx.fillRect(...)` for each bar.
- **PDF** — emit drawing operators in PDF syntax.
- **TikZ / LaTeX** — emit `\draw` commands.
- **Terminal with truecolor** — upgrade ASCII with ANSI color codes.

The spec doesn't care. Neither does the data. Only the final stage changes. **Backend-as-strategy** is the pattern, and it's the only reason a grammar-of-graphics library can support ten output formats without rewriting the user-facing API ten times.

The Rosetta Stone's ASCII rendering is not a limitation — it's the deliberate choice that the **pipeline shape matters, not the drawing tech**. The same pipeline serves the vault dashboards, the terminal `dpnbar` widgets, the emailed weekly reports, and any future web UI.

## Choreographic Case: Weekly Dashboard

```innate
(@weekly-dashboard){
  @revenue      <- @sql/query{...}
  @pace-scores  <- @daily-notes.map(.pace_score)

  @bar-chart <- @chart/bar-chart{
    title:  "Weekly Revenue",
    data:   @revenue,
    encode: {x: "day", y: "total"}
  }
  @line-chart <- @chart/line-chart{
    title:  "Pace Score Trend",
    data:   @pace-scores,
    encode: {x: "day", y: "score"}
  }

  @report <- @chart/render-all{
    charts:  [@bar-chart, @line-chart],
    backend: "ascii"          ;; or "svg", or "html"
  }

  where {
    backend_supported:  @backend ∈ ["ascii", "svg", "html"]
    charts_non_empty:   @charts.every(.data.length > 0)
    render_succeeded:   @report.length > 0
  }
}
```

Swap `backend: "ascii"` for `backend: "svg"` and the same spec produces a web dashboard. The choreography doesn't change. Only the final render stage does.

## Structures

```innate
(defstruct data-point
  label  : String
  value  : Float
  series : String)

(defstruct linear-scale
  domain-min : Float
  domain-max : Float
  range-min  : Float
  range-max  : Float)

(defstruct ordinal-scale
  categories : [String]
  range-min  : Int
  range-max  : Int)

(defstruct bar-chart
  title  : String
  points : [DataPoint]
  width  : Nat)

(defstruct line-chart
  title  : String
  points : [DataPoint]
  width  : Nat
  height : Nat)
```

## Resolver Natives

```innate
@scale/linear{domain-min, domain-max, range-min, range-max}   -> LinearScale
@scale/linear/at{scale, x}                                    -> Float
@scale/linear/fit{values, range-min, range-max, include-zero?} -> LinearScale
@scale/ordinal{categories, range-min, range-max}              -> OrdinalScale
@chart/bar-chart{title, points, width}                        -> BarChart
@chart/line-chart{title, points, width, height}               -> LineChart
@chart/render{chart, backend?}                                -> String
```

## Demo

```innate
(@demo){
  @pts <- [{label: "Jan", value: 12000}, {label: "Feb", value: 15500}, ...]
  @bar <- @chart/bar-chart{title: "Revenue", points: @pts, width: 30}
  @out <- @chart/render{chart: @bar, backend: "ascii"}
  ;; @out is a multi-line string suitable for printing, logging, or emailing
}
```

## Where

Scales MUST be pure functions — same input, same output, no hidden state. Chart specs MUST be data-class records — no mutation, no incremental building. Rendering MUST take `(spec, data) → output` as a pure function. The renderer MUST be a parameter, not a hardcoded backend. Those four rules keep the pipeline composable, testable, and retargetable.
