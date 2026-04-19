# G074 — Online White Board

> The Rosetta Stone's first project with **spatial data**. 2D coordinates, bounding boxes, distance queries, z-ordered rendering — all primitives of spatial systems, introduced at minimal scale so they can be reused throughout Graphics (G114–G130) and anywhere the noosphere needs location-aware queries.

```yaml
id: G074
title: Online White Board
category: web
requires: [G056-image-gallery, G069-wysiwyg]
provides: [spatial-coordinates, bounding-boxes, distance-query, z-order-rendering, first-class-strokes]
```

## Insight: Spatial Data Is a New Coordinate System

Every prior Rosetta Stone project with "coordinates" had 1D coordinates: byte offsets in text (G069), seconds in a schedule (G065), URLs in a history stack (G070). G074 is the first project with **2D spatial coordinates**. Distance is `sqrt((dx)² + (dy)²)`, bounding boxes are `(min_x, min_y, max_x, max_y)`, queries are "near this point" rather than "at this offset."

The jump from 1D to 2D is load-bearing. 1D has a natural total order (one thing can be "before" another); 2D does not (neither of `(3, 5)` and `(4, 4)` is "before" the other). Everything that assumed total order — sorted storage, binary search, before/after predicates — doesn't generalise. The operations have to be redesigned around **nearness**, not order.

First Rosetta Stone project where **order is replaced by proximity** as the organising principle. The noosphere's spatial compositor (see memory `project_spatial_compositor.md`) depends on this primitive: rooms are 2D regions, agents are at positions, queries are "what's nearby." G074 is the minimal case.

## Insight: Strokes Are First-Class Entities with Identity

A stroke isn't a pixel path drawn onto a buffer. It's an **entity** with an id, an author, a color, a thickness, and a list of points. You can remove a specific stroke (`remove_stroke(id)`); you can filter strokes by author; you can find strokes near a point; you can render just one stroke.

This is the **retained-mode** versus **immediate-mode** graphics distinction. Immediate-mode draws into a buffer and loses the structure; retained-mode keeps the structure and renders from it on demand. G074 is retained-mode, and that's what makes every interesting operation possible: undo (remove a stroke), edit (replace a stroke), query (who drew this?), export (render to SVG).

Every real drawing application from Illustrator to Figma to whiteboard apps is retained-mode for exactly this reason. G074 presents the minimum viable version.

## Insight: Z-Order Is Insertion Order

Strokes draw in the order they were added. The first stroke is underneath; the last stroke is on top. There is no per-stroke "z-index" field, no complicated depth calculation — the storage order IS the rendering order.

This is the simplest possible z-model and it's what Figma, Sketch, Illustrator, and every 2D vector tool uses at the layer level. Operations that change z-order (bring-to-front, send-to-back, move-up) are implemented as list rearrangements, not as field mutations. G074's `remove_preserves_z_order_of_remaining` test verifies this: removing a middle stroke doesn't renumber anything; the surviving strokes keep their relative order.

First Rosetta Stone project where **storage order is semantic**. G067 Chat App's log order was chronological (meaningful); G070 Browser's tab list order was just UI arbitration (not meaningful to the protocol); G074's stroke order is **rendering-critical**: reorder the list and the image changes.

## Insight: Spatial Queries Are Linear Scans (Until They Aren't)

`strokes_near(point, radius)` walks every stroke and checks distance. This is O(n × m) where n = stroke count, m = average points per stroke. For a whiteboard with hundreds of strokes, this is fine. For a CAD system with millions, it's catastrophic.

The production solution is a **spatial index** — quadtree, R-tree, grid-based hash, k-d tree — that answers spatial queries in O(log n) or better. G074 doesn't implement one; the linear scan is *correct*, and the spatial index is a performance optimisation that should be added only when profiling says it matters.

First Rosetta Stone project where **the naive algorithm and the optimised algorithm have the same interface**. You can swap linear scan for R-tree without changing callers. This is a property worth protecting in every geometric API — don't expose the index implementation; only the queries.

Same pattern appears throughout systems: hash maps over flat arrays for lookup, B-trees over sorted arrays for range queries, vector indexes over flat arrays for nearest-neighbour search. The queries are the API; the index is an implementation detail.

## Insight: Render Is a Different Format Projection

`to_svg` walks strokes in z-order and emits `<polyline>` tags. Same pattern as G069's `to_html` — structured model, walk it, emit markup. The content is different (SVG vs HTML); the *shape* of the code is identical: iterate, escape, emit.

Other formats are straightforward: `to_png` by rasterising strokes to a pixel buffer, `to_pdf` by emitting PDF path commands, `to_latex_tikz` for scientific publishing, `to_ascii_art` for terminal preview. All are 20-line walks because the model did the hard work up front.

First Rosetta Stone project where **the render format is genuinely parameterised**. G069 rendered to HTML only; G074 could render to SVG, PNG, PDF, ASCII with equal ease because the model doesn't care what the output format is.

## Insight: Per-Author Attribution Enables Collaboration Queries

Every stroke carries its author. `strokes_by_author(name)` filters; `strokes_near(point)` can be composed with the author filter; the bounding box of just-alice's-work is well-defined. First Rosetta Stone project where **every atom of the data structure carries its provenance** — G067 Chat App had author on messages, but the chat log was a linear sequence; G074 has author on every geometric element, enabling rich collaboration-aware queries.

The noosphere's agent-provenance tracking will use this pattern: every choreography output, every vault edit, every deliverable carries the agent/user that produced it. "Show me only what Alice's agents produced this week" becomes a filtered walk over the provenance-annotated data.

## Choreographic Case: Collaborative Diagram Session

```innate
(@diagram-session){
  @board <- @whiteboard/new
  @room <- @chat/new-room         ;; for discussion alongside drawing

  @on-stroke-from-client (@client @stroke-data){
    @id <- @whiteboard/add-stroke{board: @board,
                                   author: @client.name,
                                   color: @stroke-data.color,
                                   thickness: @stroke-data.thickness,
                                   points: @stroke-data.points}
    @broadcast/to-all-clients {type: "stroke-added", id: @id}
  }

  @on-user-request-undo (@client){
    @my-strokes <- @whiteboard/strokes-by-author{board: @board, author: @client.name}
    @latest <- @my-strokes.last
    when (@latest){
      @whiteboard/remove-stroke{board: @board, id: @latest.id}
      @broadcast/to-all-clients {type: "stroke-removed", id: @latest.id}
    }
  }

  @on-export-request {
    @svg <- @whiteboard/to-svg{board: @board, width: 800, height: 600}
    @vault/save{path: "diagram-${@timestamp}.svg", content: @svg}
  }
}
```

A collaborative whiteboard falls out naturally: clients contribute strokes, the board accumulates, per-author undo is just "remove the author's last stroke," export is "render to SVG and save." The noosphere will use this shape for collaborative diagramming, map annotation, music composition (each note is a stroke in time-frequency space), and any other 2D collaborative canvas.

## Structures

```innate
(defstruct point x : Float, y : Float)

(defstruct stroke
  id         : Int
  author     : String
  color      : String
  thickness  : Float
  points     : [Point])

(defstruct whiteboard
  strokes    : [Stroke]        ;; insertion order = z-order
  next-id    : Int)
```

## Resolver Natives

```innate
@whiteboard/new                                            -> Whiteboard
@whiteboard/add-stroke{board, author, color, thick, pts}   -> StrokeId
@whiteboard/remove-stroke{board, id}                       -> Bool
@whiteboard/clear{board}                                   -> Unit
@whiteboard/strokes-by-author{board, author}               -> [Stroke]
@whiteboard/strokes-near{board, point, radius}             -> [Stroke]
@whiteboard/bounding-box{board}                            -> (Point, Point)?
@whiteboard/to-svg{board, width, height}                   -> String
```

## Demo

```innate
(@demo){
  @b <- @whiteboard/new
  @whiteboard/add-stroke{board: @b, author: "alice", color: "#f00", thickness: 2,
                          points: [{x:10,y:10}, {x:50,y:10}, {x:50,y:50}, {x:10,y:50}, {x:10,y:10}]}
  @whiteboard/add-stroke{board: @b, author: "bob", color: "#00f", thickness: 3,
                          points: [{x:60,y:20}, {x:100,y:60}]}
  @whiteboard/strokes-by-author{board: @b, author: "alice"}.length   ;; -> 1
  @whiteboard/bounding-box{board: @b}                                ;; -> ({x:10,y:10}, {x:100,y:60})
  @whiteboard/to-svg{board: @b, width: 200, height: 100}             ;; -> <svg> ... </svg>
}
```

## Where

Strokes MUST maintain insertion order — this is the z-order, and reordering the list reorders the rendering. Stroke ids MUST be unique and monotonically increasing — reusing ids after removal creates stale-reference bugs. The bounding box MUST be None/null for an empty board — returning `(0, 0, 0, 0)` is wrong because that's a single-point box at the origin, not "no strokes." Spatial queries (distance, near, bbox) MUST treat the empty set as the absence of a point-cloud, not a zero-sized one. Per-point distance is an approximation; a production implementation would consider segments between points (point-to-line-segment distance) for more accurate "near the ink" behaviour — G074 uses vertex distance for simplicity and because at small point-density it's adequate. Z-order operations (bring-to-front, send-to-back) are list rearrangements of `strokes`, not field mutations — there is no per-stroke z-index, which keeps the model canonical. Rendering MUST visit strokes in storage order and MUST NOT reorder for any reason (colour, thickness, author) — the user's intentional ordering is authoritative.
