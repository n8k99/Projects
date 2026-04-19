# G059 — Shape Area and Perimeter

> One interface, many implementations. The same operation (area, perimeter) dispatches across concrete shapes. The noosphere's polymorphism primitive, rendered in six different dispatch mechanisms.

```yaml
id: G059
title: Shape Area and Perimeter
category: classes
requires: [G057-big-integer]
provides: [interface-dispatch, open-vs-closed-polymorphism, heterogeneous-collections, cross-language-dispatch-comparison]
```

## Insight: One Interface, Many Implementations

The simplest possible shape-polymorphism problem exists because it is the cleanest demonstration of *the* big idea: **different types of things obey the same protocol**. A Circle, a Rectangle, a Triangle, and a Regular Polygon don't share any data. They share a *behavioral contract*: each can answer `area()` and `perimeter()` using its own geometry.

The collection operations (`total_area`, `largest_by_area`, `sort_by_perimeter`) are the payoff. They take a list of shapes and work uniformly — they never inspect the concrete type. Heterogeneous data, homogeneous algorithms. That is what polymorphism buys.

## Insight: Six Dispatch Mechanisms, Same Result

The Rosetta Stone's six languages each provide a fundamentally different way to express this protocol. Studying them side by side is a lesson in what "polymorphism" even means:

| Language | Mechanism | Key feature | Open? |
|---|---|---|---|
| **Python** | ABC + duck typing | `Shape(ABC)` with `@abstractmethod`; any duck-typed class works too | Open |
| **Rust** | Traits | `impl Shape for Circle`; `Box<dyn Shape>` for heterogeneous lists | Open |
| **Go** | Structural interfaces | `Shape` is implicitly satisfied by any type with the right methods | Open, structural |
| **Common Lisp** | CLOS generic functions | `defgeneric`/`defmethod`; can dispatch on multiple args, not just self | Open, multimethod |
| **Lean** | Inductive sum type | Every variant enumerated; pattern match with exhaustivity | **Closed** |
| **InnateScript** | Resolver dispatch | `@shape/area{shape}` pattern-matches on `shape.kind` | Either, configurable |

Five of six are "open" — adding a new shape means writing a new module, nowhere else. Lean's inductive-type approach is "closed" — the compiler enforces exhaustivity, refusing to compile if you add a case and forget a function's pattern match. Both are correct; they optimize for different things.

## Insight: Open Polymorphism Enables Extension Without Modification

In Rust, adding a `Pentagon` type is:

```rust
pub struct Pentagon { ... }
impl Shape for Pentagon { ... }
```

Done. The `Shape` trait is not modified. The `total_area` function still works — it takes `&[Box<dyn Shape>]` and Pentagon is a Shape.

In Go, even less ceremony: define `Pentagon` with the three methods, and it automatically satisfies `Shape`. No import, no declaration, no registration. Structural typing is the most open form.

In Common Lisp, adding a new shape means writing `defclass`, then three `defmethod`s. The generic functions `area`, `perimeter`, and `shape-name` accept the new class without modification — CLOS looks up the method at call time based on the argument's class.

Open polymorphism is the norm in most runtime-dispatched systems. The noosphere needs it: new agent types, new document formats, new choreography kinds should be addable as new modules without touching the protocol definitions. This is why the vault uses string-tagged kinds (`kind: agent`, `kind: project`, `kind: goal`) — anyone can define a new kind and provide its resolvers.

## Insight: Closed Polymorphism Enables Exhaustivity Checking

Lean's inductive `Shape` type says: there are EXACTLY four shapes. If someone adds a fifth:

```lean
inductive Shape where
  | circle (radius : Float)
  | rectangle (width height : Float)
  | triangle (a b c : Float)
  | regularPolygon (sides : Nat) (sideLength : Float)
  | ellipse (major minor : Float)    -- NEW
```

every function that pattern-matches on `Shape` **fails to compile** until it handles `ellipse`. The compiler has enumerated the cases and confirms none are missing.

This is a different kind of power. Open polymorphism lets you add without touching the core. Closed polymorphism refuses to compile incomplete code. Every choreography in InnateScript must choose: do I want the system to stop me when I forget a case (closed), or do I want to ship a module into the wild and have callers handle unknown cases (open)? The answer depends on whether the set of cases is *stable* or *evolving*.

For the Rosetta Stone's geometry problem, both approaches work. For real software: stable taxonomies (payment_method: credit | debit | cash) benefit from closed polymorphism; evolving ecosystems (plugins, document formats, agent kinds) need open polymorphism.

## Insight: Heterogeneous Collections Are the Payoff

All six languages can hold a list of mixed shapes and compute the total area:

```python
total_area([Circle(5), Rectangle(4, 6), Triangle(3, 4, 5)])
```

```rust
total_area(&vec![
    Box::new(Circle::new(5.0)?) as Box<dyn Shape>,
    Box::new(Rectangle::new(4.0, 6.0)?) as Box<dyn Shape>,
    Box::new(Triangle::new(3.0, 4.0, 5.0)?) as Box<dyn Shape>,
])
```

The Rust version is noisier because Rust forces you to be explicit about heap allocation and dynamic dispatch (`Box<dyn Shape>`). The Python version is the cleanest because Python boxes everything by default. Go lands between them — no syntax, but a pointer indirection under the hood.

The *semantic* operation is identical across all six: fold over a mixed list, calling `.area()` on each. This is the "polymorphism in one sentence" test: if you can write `shapes.map(&.area).sum`, you have polymorphism. Whether the dispatch is a vtable lookup, a class-method lookup, a generic-function lookup, or a pattern match is implementation detail.

## Insight: Validation Is a Smart-Constructor Pattern, Not an Invariant Check

Triangle has a non-trivial precondition: the triangle inequality (`a + b > c`, etc.). If violated, no valid triangle exists. Every implementation handles this at construction:

- Python: `__post_init__` raises `ValueError`.
- Rust: `Triangle::new` returns `Result<Triangle, String>`.
- Go: `NewTriangle` returns `(Triangle, error)`.
- Common Lisp: `make-triangle` signals an error via `error`.
- Lean: `Shape.mkTriangle` returns `Except String Shape`.

The pattern is **smart constructor**: the type allows only valid values because the constructor refuses to build invalid ones. Once you have a `Triangle`, you know its sides are valid — you don't re-check inside `area()`. This generalizes: every domain-constrained value class uses smart constructors. A `Date` type refuses February 30. A `Percentage` refuses -5. G059 introduces the pattern with one of its clearest examples.

## Choreographic Case: Asset Library with Mixed Shape Metadata

```innate
(@gallery-metadata){
  @assets <- @gallery/images
  @bounding-shapes <- @assets.map(asset => asset.bounding-region)
  ;; bounding-region is a Shape: circle for portraits, rectangle for landscapes

  @total-coverage <- @shape/total-area{shapes: @bounding-shapes}
  @densest <- @shape/largest-by-area{shapes: @bounding-shapes}

  where {
    all_valid:       @bounding-shapes.every(@shape/valid)
    coverage_sane:   @total-coverage > 0 && @total-coverage < @canvas-size
  }
}
```

The choreography doesn't know whether each bounding region is a circle or a rectangle. It calls `area` on each and `total_area` on the collection. Polymorphism handles the dispatch.

## Structures

```innate
;; closed sum type — Lean style
(defunion shape
  circle         { radius : Float }
  rectangle      { width : Float, height : Float }
  triangle       { a : Float, b : Float, c : Float }
  regular-polygon{ sides : Nat,   side-length : Float })
```

## Resolver Natives

```innate
@shape/area{shape: Shape}                     -> Float
@shape/perimeter{shape: Shape}                -> Float
@shape/name{shape: Shape}                     -> String
@shape/total-area{shapes: [Shape]}            -> Float
@shape/largest-by-area{shapes: [Shape]}       -> Shape
@shape/sort-by-perimeter{shapes: [Shape]}     -> [Shape]
@shape/valid{shape: Shape}                    -> Bool
```

## Demo

```innate
(@demo){
  @shapes <- [
    @circle{radius: 5},
    @rectangle{width: 4, height: 6},
    @rectangle{width: 3, height: 3},          ;; will name itself "Square"
    @triangle{a: 3, b: 4, c: 5},
    @triangle{a: 5, b: 5, c: 5},
    @regular-polygon{sides: 6, side-length: 2}
  ]
  @total <- @shape/total-area{shapes: @shapes}
  @biggest <- @shape/largest-by-area{shapes: @shapes}
  ;; @biggest might be the hexagon: area ≈ 10.39
}
```

## Where

Every concrete shape MUST implement `area`, `perimeter`, and `name`. Validation MUST happen at construction — invalid shapes (triangle-inequality violations, negative dimensions) MUST be unconstructible. Collection operations (`total_area`, `largest_by_area`, `sort_by_perimeter`) MUST be defined in terms of the interface only — they MUST NOT pattern-match on concrete types. Those three rules are the cost of admission to polymorphism.
