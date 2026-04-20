# G097 — Image Map Generator

> The Rosetta Stone's thirteenth **Files-category** project. Introduces **shape polymorphism with hit testing** and the canonical **ray-casting point-in-polygon algorithm**. An image map is a list of shaped regions (rect, circle, polygon) on a 2D canvas; given a click at (x, y), find the region containing it. Overlapping regions resolve via **z-order** (topmost wins). Exports HTML `<map>` elements — a 25-year-old HTML feature that remains the quickest way to make parts of an image clickable.

```yaml
id: G097
title: Image Map Generator
category: files
requires: [G059-shape-polymorphism, G074-whiteboard, G076-bookmarks]
provides: [point-in-polygon-ray-casting, z-ordered-hit-testing, shape-polymorphism-via-contains, html-map-export]
```

## Insight: Hit Testing Is One Method on Every Shape

Every shape (rect, circle, polygon) answers the same question: `contains(x, y) → bool`. The shapes differ in their internal data and their test logic, but from the caller's perspective they're interchangeable. That's **shape polymorphism** with one method, dispatched by kind tag in Python/Go/CL, enum variant in Rust/Lean.

This is G059's polymorphism applied to geometry. G059 had `shape_area(Shape) → f64`; G097 has `contains(Shape, x, y) → bool`. Same pattern: one method, many shapes, dispatched on kind.

First Rosetta Stone project where **the polymorphic method produces a boolean, not a value**. Hit testing is a **predicate**, not a function. That subtle shift — "does this shape claim this point?" — is what enables the composable hit-testing pipeline (filter regions, take topmost).

## Insight: Point-in-Polygon Is Ray Casting

The textbook algorithm: shoot a horizontal ray from `(x, y)` going right to infinity; count how many polygon edges it crosses; **odd count = inside, even count = outside**. Works for convex and concave polygons, even self-intersecting ones (with the even-odd rule).

Each edge from `(xi, yi)` to `(xj, yj)` is tested: does the ray cross the edge? The edge spans `y` if and only if `(yi > y) != (yj > y)` — one endpoint above, one below. Compute the x-intersection; if it's to the right of `x`, the ray crosses there, toggle the inside flag.

This is the same algorithm every GIS library (PostGIS, GEOS), every CAD tool, every SVG renderer uses. First published in 1962; hasn't been improved on for general polygons.

First Rosetta Stone project with **a named classical algorithm** as the payload. Ray casting is worth implementing six times because the cross-language consistency verifies the algorithm's definition, not just our understanding of it.

## Insight: Z-Order Resolves Overlaps

Two rects stacked — which one does the click land on? Answer: whichever has higher `z`. `hit_test(x, y)` sorts regions by `z` descending and returns the first that contains the point.

Same model as every UI framework (CSS z-index, tkinter raise/lower, CAD layer stacking). The z value is a small integer; the semantic is "bigger z is on top". G097 doesn't enforce uniqueness — two regions can share a z and rely on insertion order (stable sort).

First Rosetta Stone project with **explicit z-ordering as a contract**. G074's whiteboard had z-order for drawing; G097 applies it to hit testing. Both use stable sort so that insertion order breaks z ties deterministically.

## Insight: HTML Image Maps Are the Simplest Output Target

Before CSS and SVG were universal, `<map>` + `<area>` was how you made parts of an image clickable:

```html
<map name="nav">
  <area shape="rect" coords="0,0,100,100" href="/home" alt="Home">
  <area shape="circle" coords="50,50,20" href="/center" alt="Centre">
  <area shape="poly" coords="200,0,250,0,225,50" href="/tri" alt="Tri">
</map>
```

G097 generates this exact syntax. The format is 25 years old, universally supported, and can be pasted into any HTML document. SVG would be richer (gradients, animations) but requires more scaffolding. CSS would need positioning logic. `<map>` just works.

First Rosetta Stone project where **the export target is an old-but-universal standard** chosen for ubiquity over features. Every browser understands `<map>`; nobody needs a dependency to read it.

## Insight: Rect Coords for HTML Are Different Than for Hit Testing

G097's internal rect is `(x, y, w, h)` — top-left plus width/height. HTML `<area shape="rect">` takes `(x1, y1, x2, y2)` — top-left plus bottom-right. The converter emits `(x, y, x+w, y+h)`.

Circles are the same in both (centre + radius). Polygons are the same (list of coords). Only rects need translation. This is a tiny thing but illustrates the general principle: **internal representation and export representation can differ**, and the exporter mediates.

First Rosetta Stone project with an **explicit coordinate-system translation at export time**. G090's archive serialised bytes as-is; G094's logs serialised fields as-is. G097 transforms some fields to match the target format's expectations.

## Choreographic Case: Vault Dashboard Hot Spots

```innate
(@vault-dashboard){
  @map <- @im/new{}
  @im/add{map: @map,
          region: {shape: @im/rect{x: 0, y: 0, w: 200, h: 100},
                    action: "/tasks", alt: "Tasks"}}
  @im/add{map: @map,
          region: {shape: @im/rect{x: 200, y: 0, w: 200, h: 100},
                    action: "/projects", alt: "Projects"}}
  @im/add{map: @map,
          region: {shape: @im/circle{cx: 150, cy: 150, r: 30},
                    action: "/important", alt: "Urgent", z: 10}}

  @on-user-clicks (@x @y){
    @region <- @im/hit-test{map: @map, x: @x, y: @y}
    @when (@region){ @navigate{to: @region.action} }
  }

  @html <- @im/to-html{map: @map, name: "dashboard"}
  @vault/save{path: "ui/dashboard.html", content: @html}
}
```

A static dashboard with rectangular panels and a circular urgent-item overlay. Click dispatches to navigation; the same map exports as HTML for server-rendered views.

## Structures

```innate
(defenum shape-kind RECT | CIRCLE | POLYGON)

(defstruct shape
  kind   : ShapeKind
  ;; rect
  x, y, w, h : Int
  ;; circle
  cx, cy, r  : Int
  ;; polygon
  points     : [(Int, Int)])

(defstruct region
  id     : Int
  shape  : Shape
  action : String
  alt    : String
  z      : Int)
```

## Resolver Natives

```innate
@im/rect{x, y, w, h}                    -> Shape
@im/circle{cx, cy, r}                   -> Shape
@im/polygon{points}                     -> Shape
@im/contains{shape, x, y}               -> Bool
@im/bbox{shape}                         -> (Int, Int, Int, Int)
@im/new{}                               -> ImageMap
@im/add{map, region}                    -> Unit
@im/hit-test{map, x, y}                 -> Region | null
@im/hit-test-all{map, x, y}             -> [Region]
@im/to-html{map, name}                  -> String
```

## Demo

```innate
(@demo){
  @m <- @im/new{}
  @im/add{map: @m, region: {id: 1, shape: @im/rect{0, 0, 100, 100},
                             action: "/home", alt: "Home", z: 0}}
  @im/add{map: @m, region: {id: 2, shape: @im/circle{50, 50, 20},
                             action: "/center", alt: "Centre", z: 1}}
  @im/hit-test{map: @m, x: 50, y: 50}
  ;; -> {id: 2, action: "/center"}  (circle wins on z)

  @im/hit-test{map: @m, x: 90, y: 90}
  ;; -> {id: 1, action: "/home"}  (only rect contains (90,90))
}
```

## Where

Every shape MUST answer the same `contains(x, y)` predicate — polymorphism is the contract that makes the hit-testing pipeline work uniformly. Point-in-polygon MUST use ray casting with the even-odd rule — other algorithms exist (winding number) but ray casting is the canonical choice and what every downstream library expects. Z-order MUST break overlap ties deterministically via stable sort — flaky hit-testing is worse than no hit-testing. HTML rect coords MUST be `(x1, y1, x2, y2)` (top-left + bottom-right), NOT `(x, y, w, h)` — that's what the HTML spec demands, even though our internal representation uses width/height. Empty polygons MUST return false for contains, NOT crash — boundary cases matter when users programmatically build maps from data. The `<area>` element MUST include `alt` text — accessibility is non-negotiable, and screen readers depend on it.
