# G060 — Matrix Class

> A 2D value class. The **shape** is part of the value; which operations are legal depends on the shape, not just the values. First non-commutative arithmetic in the Rosetta Stone.

```yaml
id: G060
title: Matrix Class
category: classes
requires: [G057-big-integer, G059-shapes]
provides: [shape-typed-value, non-commutative-arithmetic, two-sided-identity, partial-operations, shape-inference]
```

## Insight: Shape Is Part of the Value

A `Matrix` is not just "a grid of floats." It is a **shape + contents**. A 2×3 matrix and a 3×2 matrix have the same six entries but are different values: one represents a linear map from ℝ³ to ℝ², the other from ℝ² to ℝ³. You cannot substitute one for the other.

G057's BigInt had invariants over its representation (canonical form). G060's Matrix has invariants over a *structural property* — `(rows, cols)` — that gates which operations are legal. This is the first value class where:

- `A + B` requires `A.shape == B.shape`.
- `A × B` requires `A.ncols == B.nrows`.
- `det(A)` requires `A.is_square`.

Two matrices of incompatible shape can't be added. Runtime check in Rust/Go/Python/CL; type error in a dependent-typed system. Either way, the *kind of illegality* is structural — the VALUES could be anything and the operation would still be illegal. That's what makes shape a first-class part of the type.

The noosphere will encounter this constantly. A choreography that expects three participants can't be joined into a two-participant context. A schedule for a week-of-seven-days can't be added to a schedule for a weekend. Whenever a type has a structural parameter that distinguishes sub-types, G060's shape-as-type pattern applies.

## Insight: Multiplication Is Non-Commutative

Every arithmetic operation in the Rosetta Stone so far has been commutative: add integers, multiply numbers, combine tags into sets. G060 introduces an operation where `A × B ≠ B × A` — not just different values, but often different *shapes*. If A is 2×3 and B is 3×2:

- `A × B` is 2×2.
- `B × A` is 3×3.

They are not merely unequal; they are objects of different types. This is qualitatively new. A large fraction of programmers have internalized commutativity as "obvious" because scalar arithmetic taught it. G060 re-teaches: in the real world, order matters for most operations.

This has consequences throughout the noosphere. Function composition is non-commutative: `parse ∘ validate ≠ validate ∘ parse`. Choreography sequencing is non-commutative: `sarah-then-kathryn ≠ kathryn-then-sarah`. Writing a log entry before acquiring a lock is not the same as the reverse. G060 is the Rosetta Stone's first explicit non-commutative algebra; every pipeline that sequences operations inherits the lesson.

**Associative multiplication is the saving grace.** Even though `AB ≠ BA`, `(AB)C == A(BC)`. Associativity is what lets chains of matrices be written without parentheses. It is the same property that makes function composition chainable. When designing the resolver's `then` combinator in InnateScript, the right algebraic law to preserve is associativity, not commutativity.

## Insight: Two-Sided Identity — But the Identity Depends on the Shape

Every commutative operation has a unique identity: 0 for addition, 1 for multiplication, ∅ for union. G060 has **two-sided identities parametrized by shape**:

```
I_n × A == A  (when A is n×anything)
A × I_m == A  (when A is anything×m)
```

There's a different identity for every square size. `I_2` is not a universal neutral element; it's neutral specifically for 2-row or 2-column partners. This is the Rosetta Stone's first taste of **dependent types**: the type of a value (`I_n`) depends on a runtime parameter (`n`).

Lean could encode this at compile time: `identity : (n : Nat) → Matrix n n`. Rust, Go, Common Lisp, Python cannot — the shape parameter lives at runtime, so the identity is a runtime-constructed value. The difference between "dependent type" and "value with a shape parameter" is where the check happens: compile time vs runtime. Both implementations work; they catch different classes of bug.

## Insight: Determinant Is a Partially-Defined Operation

`det(A)` is defined only for square matrices. On a 2×3 matrix, no determinant exists — not "returns zero" or "returns null." **It has no meaning.**

Every prior operation in the Rosetta Stone was *total* — defined on every input. G060 introduces the first **partial** operation: valid on a subset of the domain. In Rust/Go/CL/Python, this is a runtime error; in Lean with dependent types, it would be a compile error.

Partial operations are load-bearing in real systems. `first()` of an empty list has no meaning. `divide` by zero has no meaning. `max` of an empty set has no meaning. Ignoring partiality produces the vast majority of runtime crashes. G060 is the Rosetta Stone's first project where partiality is *principled* — not an oversight, but a fundamental property of the math — and the error-handling approach (returning `Result<f64>` or raising a typed error) sets the template for every future partial operation in the noosphere.

## Insight: Shape Inference for the Result

When you multiply `m×n` by `n×p`, the result is `m×p`. The *inner* dimensions (both `n`) cancel; the *outer* dimensions (`m` and `p`) survive. Shape inference happens automatically:

```
(2×3) × (3×2) = 2×2
(3×2) × (2×3) = 3×3
```

The caller doesn't specify the output shape; it's derived from the operands. This is the first project where a value's shape is *computed* rather than *specified*. Every tensor library, every streaming pipeline, every type-inferring compiler does this — G060 is the minimal teaching example. When InnateScript introduces pipelines of shaped data (matrices, tensors, DataFrames), shape inference will be a primary concern.

## Insight: Cofactor Expansion — Simple, Recursive, Slow

The determinant algorithm implemented here — Laplace/cofactor expansion on the first row — is O(n!). For a 10×10 matrix that's 3.6 million operations; for 20×20 it's 2.4 quintillion. Real-world libraries use LU decomposition (O(n³)) or similar. The Rosetta Stone chooses cofactor expansion for the same reason G057 chose base-10 digits: **the algorithm matches the math you learned on paper**. You can trace the recursion by hand for a 3×3 and see it match the `a(ei - fh) - b(di - fg) + c(dh - eg)` formula from school.

Clarity over speed is a recurring choice in this category. G057's multiplication is schoolbook; G058's line-drawing is step-along-diagonal; G060's determinant is cofactor expansion. Each is the most readable algorithm for its problem; each is asymptotically worse than the production alternative. For a teaching corpus, readability IS the point.

## Choreographic Case: Agent Workload Matrix

```innate
(@agent-workload-transform){
  ;; rows = agents, cols = days; entry = hours allocated
  @workload <- @db/load-matrix{table: "allocations"}    ;; N agents × 7 days
  @priority <- @db/load-matrix{table: "agent-priority"} ;; 7 days × K projects

  ;; Matrix product: N×7 times 7×K = N×K — how much each agent touched each project.
  @agent-project <- @matrix/matmul{a: @workload, b: @priority}

  where {
    shapes_compatible:  @workload.ncols == @priority.nrows
    no_negative_hours:  @agent-project.entries.all(. >= 0)
    result_shape:       @agent-project.shape == (@workload.nrows, @priority.ncols)
  }
}
```

The `where` checks shape explicitly — even though the matmul itself would refuse mismatched shapes, asserting the derived shape in the clause documents the expected pipeline output and catches bugs where `@priority` accidentally loaded in the wrong orientation.

## Structures

```innate
(defstruct matrix
  rows : [[Float]])              ;; rectangular; all inner lists same length
```

## Resolver Natives

```innate
@matrix/new{rows}                              -> Matrix
@matrix/zeros{rows, cols}                      -> Matrix
@matrix/identity{n}                            -> Matrix
@matrix/shape{m}                               -> (Nat, Nat)
@matrix/get{m, i, j}                           -> Float
@matrix/add{a, b}                              -> Matrix         ;; shape-gated
@matrix/sub{a, b}                              -> Matrix
@matrix/scalar-mul{m, k}                       -> Matrix
@matrix/matmul{a, b}                           -> Matrix         ;; inner-dim gated
@matrix/transpose{m}                           -> Matrix
@matrix/determinant{m}                         -> Float          ;; square only
```

## Demo

```innate
(@demo){
  @A <- @matrix/new{rows: [[1,2,3], [4,5,6]]}         ;; 2×3
  @B <- @matrix/new{rows: [[7,8], [9,10], [11,12]]}   ;; 3×2

  @C <- @matrix/matmul{a: @A, b: @B}    ;; 2×2
  @D <- @matrix/matmul{a: @B, b: @A}    ;; 3×3 — different shape, different value

  ;; @A + @B is an error: shape mismatch (2×3 vs 3×2)
  ;; @A's determinant is an error: 2×3 is not square
}
```

## Where

Shape MUST be part of every matrix's identity. Addition MUST require matching shapes; matrix multiplication MUST require inner dimensions to match; determinant MUST require squareness. Operations on mismatched shapes MUST fail explicitly rather than silently producing nonsense. The identity matrix MUST be parameterized by size, and `I_n × A = A = A × I_m` MUST hold. Those four rules are what make a matrix class a matrix class rather than a nested list of numbers.
