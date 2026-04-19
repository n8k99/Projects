# G057 — Handle Large Numbers

> The class IS the number. Operations are algebraic laws, not methods on a container. The first project in Classes whose entity is a value, not a domain object.

```yaml
id: G057
title: Handle Large Numbers
category: classes
requires: [G003-prime-factorization, G008-binary-to-decimal]
provides: [value-entity, canonical-form-invariant, schoolbook-algorithms, law-governed-operations]
```

## Insight: The Class Is a Value, Not a Container

Every prior project in Classes modeled a *thing in the world*: a product, a reservation, a grade, a recipe, an image. The entity was a **container** with domain-specific fields. G057 is qualitatively different: a BigInt doesn't contain anything external. The BigInt **is** its own value. `BigInt.parse("12345")` is the number twelve thousand three hundred forty-five, end of story.

This is the distinction between:

- **Domain entities** — Product, Appointment, Recipe. Shaped by the world they model. Fields are the vocabulary of their domain.
- **Value classes** — BigInt, Date, Vec2, Color. Shaped by algebra. Fields are the representation, not the meaning.

Every programming language has both kinds. The noosphere will have both: a project is a domain entity; a pace score is a value class. A conversation is a domain entity; a token count is a value class. G057 is the Rosetta Stone's first value class, and every future numeric or symbolic type will follow the pattern it establishes.

## Insight: Operations Are Laws, Not Just Methods

A BigInt's `add` method is not just "the function that adds." It must satisfy **commutativity**:

```
∀ x, y : x.add(y) == y.add(x)
```

and **associativity**:

```
∀ x, y, z : x.add(y).add(z) == x.add(y.add(z))
```

`mul` must distribute over `add`:

```
∀ x, y, z : x.mul(y.add(z)) == x.mul(y).add(x.mul(z))
```

These are not suggestions. They are the **specification** of what makes the class a correct number system. A BigInt implementation that had a working `add` method but failed commutativity would be wrong, regardless of whether the method "worked" on any individual case.

Every prior project had operations defined by their contract with the world ("rent fails if no copies available"). G057 is the first project where operations are defined by their contract with **mathematics**. Laws-govern-operations generalizes to every future value class: a Date's `add_days` has algebraic structure too; a color-space's `blend` has compositional laws; a pace-score's combinator has associativity requirements. The RosettaStone introduces the pattern here.

Property-based tests are the right shape for these. In the Rust tests, `addition_is_commutative` samples two numbers and checks `a + b == b + a`. That one test, with a good property test generator, replaces hundreds of example-based tests. Every value class in the Rosetta Stone going forward should ship with property tests alongside example tests.

## Insight: Canonical Form Is a Class Invariant

A BigInt's representation is `{negative: bool, digits: [little-endian base-10]}`. Multiple representations can encode the same value:

```
{negative: false, digits: []}          == 0
{negative: false, digits: [0]}         == 0   (with a leading zero)
{negative: true,  digits: []}          == 0   ("negative zero")
{negative: false, digits: [5, 0, 0]}   == 5   (with two leading zeros)
```

Only ONE of these is **canonical**: `{negative: false, digits: []}`. All constructors — `from_int`, `parse`, `add`, `sub`, `mul` — MUST produce canonical form. Once you commit to canonical form, equality is structural: two BigInts represent the same value iff their representations are identical. Without canonical form, equality would require normalization on every comparison, and the type becomes a bug farm.

This is a class invariant: an assertion that must hold at the boundary of every public method. Invariants are the contract between the class and its users. The more subtle the invariant, the more important it is to test — `parse("0007")` should produce the same BigInt as `from_int(7)`, and the test catches the leading-zero bug at its source.

Every noosphere value class will have invariants of this shape. A Date must normalize February 30 to March 2. A Duration must not overflow. A Color must clamp channels to [0, 1]. Canonical form is how a value class enforces that it actually represents what it claims to represent.

## Insight: Schoolbook Algorithms Are the Rosetta Stone's First Non-Trivial Algorithm in Classes

G048–G056 had essentially no algorithms — they were data-shape exercises. Sum, filter, group-by, map. G057 is the first project in Classes where writing correct code requires **actually implementing an algorithm**:

- `addition` — digit + digit + carry, loop over the longer operand.
- `subtraction` — digit − digit − borrow, assuming the minuend is larger.
- `multiplication` — each digit of one operand times every digit of the other, shifted and summed.

These are the algorithms you learned in elementary school. Writing them clearly in six languages is the point. The Rust implementation runs `999 * 999 = 998001` by literal carry propagation through three digits; the carry chain is visible in the loop. `30! = 265252859812191058636308480000000` requires thirty multiplications, each propagating carries through up to 33 digits.

Performance is not the concern here — the Rosetta Stone is explicit about clarity over speed. In production, any serious BigInt uses base 2^32 or 2^64 limbs and Karatsuba multiplication or better. The base-10 implementations here are pedagogically clear because every intermediate value matches what a human would write on paper. The algorithm IS the code.

## Insight: Signed Arithmetic Splits Into Two Cases, Cleanly

Addition of two numbers with the **same sign** is magnitude addition:

```
 (+5) + (+3)  =  +(5+3)  = +8
 (-5) + (-3)  =  -(5+3)  = -8
```

Addition of two numbers with **different signs** is magnitude subtraction, with the result's sign taken from the larger-magnitude operand:

```
 (+5) + (-3)  =  +(5-3)  = +2       (left is larger)
 (-5) + (+3)  =  -(5-3)  = -2       (left is larger)
 (+3) + (-5)  =  -(5-3)  = -2       (right is larger)
```

Four demo cases above cover every signed-addition path. Once you see the structure, signed subtraction is just `x - y == x + (-y)`, and signed multiplication is "XOR the signs, multiply the magnitudes." The sign logic is tiny; the magnitude arithmetic is the real work.

This split — "same-sign: add magnitudes; different-sign: subtract magnitudes, pick sign of larger" — is the template for every signed numeric type. Every future class in the noosphere that adds an abelian sign (debit/credit, gain/loss, inflow/outflow) will use the same case analysis. G052's bank account already did this implicitly in its transfer logic. G057 makes it explicit and primitive.

## Choreographic Case: Cumulative Token Audit Across All Conversations

```innate
(@token-audit){
  @conversations <- @db/conversations
  @total <- @conversations.fold(
    start: @bigint/zero,
    step:  (acc, conv) => acc.add(@bigint/from-int(conv.token_count))
  )
  where {
    no_overflow:   true     ;; BigInt can't overflow
    matches_daily: @total.equal(@daily_totals.fold(:add, start: @bigint/zero))
    non_negative:  ¬ @total.negative
  }
}
```

Token counts across a year of conversations can exceed 64-bit range. Summing them natively would silently overflow. BigInt addition can't overflow — the class's *entire point* is unbounded precision. This is exactly when a value class earns its place: when the laws it enforces (in this case, no silent overflow) are things the domain assumes but the language cannot guarantee.

## Structures

```innate
(defstruct big-int
  negative : Bool
  digits   : [Nat])                  ;; little-endian base-10; [] = zero, canonical form
```

## Resolver Natives

```innate
@bigint/zero                                            -> BigInt
@bigint/from-int{n: Int}                                -> BigInt
@bigint/parse{s: String}                                -> BigInt
@bigint/to-string{x: BigInt}                            -> String
@bigint/neg{x}                                          -> BigInt
@bigint/abs{x}                                          -> BigInt
@bigint/add{x, y}                                       -> BigInt
@bigint/sub{x, y}                                       -> BigInt
@bigint/mul{x, y}                                       -> BigInt
@bigint/cmp{x, y}                                       -> Int     ;; -1 / 0 / +1
@bigint/equal{x, y}                                     -> Bool
```

## Demo

```innate
(@demo){
  @a <- @bigint/parse{"123456789012345678901234567890"}
  @b <- @bigint/parse{"-98765432109876543210"}

  @sum  <- @bigint/add{x: @a, y: @b}
  @prod <- @bigint/mul{x: @a, y: @b}

  ;; factorial of 30 — 33 digits, no overflow possible
  @fact <- [1..=30].fold(
    start: @bigint/from-int{n: 1},
    step:  (acc, k) => @bigint/mul{x: acc, y: @bigint/from-int{n: k}}
  )
  ;; @fact == @bigint/parse{"265252859812191058636308480000000"}
}
```

## Where

The class MUST represent one mathematical integer of arbitrary size. Every constructor MUST return canonical form — no leading zeros, no negative zero. Operations MUST satisfy their algebraic laws (commutativity, associativity, distributivity); violating a law is a bug regardless of whether any specific case works. Multiplication MUST implement the schoolbook carry — the arithmetic you learned in elementary school, written down precisely.
