# G115 — Mind Mapper

> The Rosetta Stone's second **Graphics project**. Introduces the **radial auto-layout algorithm** every mind-mapping tool (XMind, MindMup, SimpleMind, iThoughts) uses — depth maps to radius, sibling index maps to angular position within the parent's arc. Each recursion subdivides the arc further. Also introduces **SVG emission** — the Rosetta Stone's first visual output format. Together they turn a tree of ideas into a diagram a browser can render directly.

```yaml
id: G115
title: Mind Mapper
category: graphics
requires: [G056-tree, G097-image-map-generator, G111-erd-creator, G114-slide-show]
provides: [radial-polar-layout, recursive-tree-model, collapsed-subtree-state, svg-emission]
```

## Insight: Polar Coordinates Are Natural for Trees

A tree mapped to Cartesian (x, y) forces choices: left-to-right? top-to-bottom? Both waste space on unbalanced trees. **Polar** coordinates sidestep it: depth → radius, angle → sibling position. Every level is a concentric ring; siblings share a wedge that's subdivided among their own children.

Conversion to Cartesian happens at the leaves: `x = cx + r * cos(angle)`, `y = cy + r * sin(angle)`. The tree's branching structure is preserved naturally because an arc can be divided recursively. A node with two children splits its arc in half; one with five, in fifths.

First Rosetta Stone project with **polar-to-cartesian coordinate conversion** as its core algorithm. G097's hit testing and G111's ERD layout both used Cartesian; G115 demonstrates why polar wins for trees.

## Insight: Tree-of-Children vs Parent-Pointer

G113's forum used parent-pointer storage (flat list, `parent_id` references). G115's mind map uses the opposite: each node has `children: Vec<MindNode>` — nested structure.

Why the difference? Mind maps are **built and edited as trees** — "add child", "collapse subtree", "reorder children". Parent-pointer storage makes those operations awkward (filter-and-rebuild). Forum threads are **persisted flat** and rebuilt on read because that's how SQL stores them naturally.

First Rosetta Stone project where **the choice of tree representation is driven by dominant operation**. Build-time edits favour nested children; read-time joins favour parent pointers.

## Insight: Collapsed State Travels With the Node

Each node has `collapsed: bool`. When true, layout, traversal, and counts all skip its subtree. Collapsed state is per-node, carried with the node, not a separate "collapsed set" the layout has to consult.

This matches every UI pattern: clicking a collapse triangle toggles the node's own state; the surrounding layout reads that state without needing a side-channel.

First Rosetta Stone project where **UI state is a field on the data object** rather than an external set. G109 separated library (canonical) from watchlist (per-user); G115 pulls UI state onto the node itself because there's no multi-user concern here.

## Insight: Layout Emits a Flat List of Placed Nodes

`radial_layout(tree) → Vec<LaidOutNode>`. Input: nested tree. Output: flat list where each entry has (id, label, color, depth, x, y, parent_id). The SVG emitter walks the flat list; it doesn't know about trees.

This is the **IR separation** pattern from G091 (PDF) and G095 (spreadsheet): the semantic model (tree) is one thing; the laid-out, positioned, ready-to-render IR is another. Same data, two representations, linked by a pure transformation.

First Rosetta Stone project where **a tree is flattened into a positioned list** as part of the rendering pipeline.

## Insight: SVG Is the First Visual Format

Prior projects rendered text (G091 pagination, G094 logs, G095 CSV, G099 snippets, G111 DOT). G115 emits SVG — XML with `<circle>`, `<line>`, `<text>` elements. A browser opens the output directly; no external tool required.

SVG is the de facto vector-graphics interchange format. Every diagramming tool, every data-viz library (D3, Vega, matplotlib's svg backend) exports SVG. G115 joins them.

First Rosetta Stone project where **the output opens in a browser as-is**. G111's DOT needs Graphviz to render; G115's SVG is self-rendering.

## Insight: XML Escape Is Non-Optional

Labels can contain `&`, `<`, `>`, `"` — all of which are XML metacharacters. Without escaping, a label like `"A & B"` breaks the SVG. G115's `escape_xml` handles all four, consistently across the six languages.

First Rosetta Stone project where **XML/HTML escape is a mandatory preprocessing step**. G097's HTML was controlled content; G115's labels come from user input and must be sanitised.

## Choreographic Case: Vault Idea Map

```innate
(@vault-idea-map){
  @tree <- @vault/build-tag-tree{root-tag: "ideas"}
  @nodes <- @mm/radial-layout{tree: @tree, cx: 500, cy: 500, radius-step: 150}
  @svg <- @mm/to-svg{nodes: @nodes, width: 1000, height: 1000, radius: 30}
  @ui/render-svg{content: @svg}

  @on-user-click-node (@node-id){
    @mm/toggle-collapse{tree: @tree, id: @node-id}
    @ui/refresh{}
  }
}
```

Vault reads tagged notes into a tree, radial-lays them out, emits SVG the UI renders. Clicks toggle collapse; re-layout re-render in one round trip.

## Structures

```innate
(defstruct mind-node
  id         : Int
  label      : String
  color      : String      ;; hex like "#3b82f6"
  collapsed  : Bool
  children   : [MindNode])

(defstruct laid-out-node
  id         : Int
  label      : String
  color      : String
  depth      : Int
  x          : Float
  y          : Float
  parent-id  : Int?
  collapsed  : Bool)
```

## Resolver Natives

```innate
@mm/node{id, label}                              -> MindNode
@mm/with-child{node, child}                      -> MindNode
@mm/with-color{node, color}                      -> MindNode
@mm/find{root, id}                               -> MindNode | null
@mm/toggle-collapse{root, id}                    -> Bool | null
@mm/visible-count{node}                          -> Int
@mm/radial-layout{root, cx, cy, radius-step}     -> [LaidOutNode]
@mm/traverse-dfs{root}                           -> [Int]
@mm/traverse-bfs{root}                           -> [Int]
@mm/to-svg{nodes, width, height, radius}         -> String
```

## Demo

```innate
(@demo){
  @tree <- @mm/node{id: 1, label: "Rosetta Stone"}
           .with-child{child: @mm/node{id: 2, label: "Languages"}
                              .with-child{child: @mm/node{id: 5, label: "Rust"}}
                              .with-child{child: @mm/node{id: 6, label: "Python"}}}
           .with-child{child: @mm/node{id: 3, label: "Categories"}
                              .with-child{child: @mm/node{id: 8, label: "Graphics"}}}

  @nodes <- @mm/radial-layout{root: @tree, cx: 500, cy: 500, radius-step: 150}
  @svg <- @mm/to-svg{nodes: @nodes, width: 1000, height: 1000, radius: 30}
  ;; SVG contains root circle at (500, 500), children at radius 150, grandchildren at 300.
}
```

## Where

Layout MUST be polar (depth → radius, sibling arc → angle) — Cartesian layouts don't generalise to unbalanced trees. Tree representation MUST be nested children, NOT parent pointers — mind maps are tree-shaped in usage; parent pointers make subtree operations awkward. Collapsed state MUST be per-node, carried with the node — no side-channel collapsed-set that can drift from the tree. Layout output MUST be a flat list of positioned nodes — emission layer shouldn't know about trees. XML metacharacters (`& < > "`) in labels MUST be escaped — unescaped user input breaks SVG. SVG MUST include both edges AND nodes — edges are drawn first so nodes render on top.
