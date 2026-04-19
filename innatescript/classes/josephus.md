# G063 — Josephus Problem

> The Rosetta Stone's first **closed-form shortcut** — a problem whose answer can be computed in O(n) by a recurrence without ever materialising the O(n²) simulation. The first **circular data structure**. The first project whose answer is the **fixpoint of an iterated elimination**.

```yaml
id: G063
title: Josephus Problem
category: classes
requires: [G062-vending-machine]
provides: [circular-indexing, modular-position, closed-form-recurrence, iterated-reduction, fixpoint-answer]
```

## Insight: Modular Position Replaces Pointer Chasing

N people stand in a circle, numbered `0 .. n-1`. The next-to-be-eliminated position is `(current + k - 1) mod |ring|`. That single expression is the whole of circular traversal: no linked list, no cycle-detection, no "wrap-around" bookkeeping. Modulo *is* the ring.

This is the first Rosetta Stone project where **arithmetic replaces structure**. A naive implementation might reach for a circular linked list (Common Lisp's `(setf (cdr tail) ring)` trick); that works, but the modular-arithmetic implementation is shorter, faster, and portable across every language with `%`. The decision — arithmetic over structure — is the central design lesson.

Modular position is a primitive the noosphere uses everywhere. Day-of-week (`date.timestamp mod 7`). Hour-of-day. Beat-of-a-measure in music. Ring-buffer writeback in the IPC layer. G063 presents the primitive in its simplest possible domain.

## Insight: The Recurrence Obsoletes the Simulation

The classical derivation:

```
J(1, k) = 0                          -- one person, they survive
J(n, k) = (J(n-1, k) + k) mod n      -- otherwise, the survivor of n is
                                        the survivor of n-1 shifted by k
```

This is O(n). The simulation is O(n·k) (or O(n²) if we use array deletion). For n=1,000,000 the recurrence runs in milliseconds; the simulation takes a trillion operations. The closed form is not an optimisation — it is a qualitative change in what the problem *is*.

Other Rosetta Stone projects where a closed form exists:
- G002 Fibonacci — the golden-ratio formula replaces the recursion (we kept the recursion there for pedagogy, but the closed form exists).
- G057 BigInt — schoolbook multiplication is O(n²); Karatsuba is O(n^1.58); FFT is O(n log n). A ladder of closed-form shortcuts.

G063 is the minimal case where a one-line recurrence obliterates a visible simulation. The simulation stays in the code as a verification oracle — it proves the recurrence computes the right thing — but in production nobody runs it.

## Insight: Two Queries on the Same Process

The problem has two natural questions:

1. **Who survives?** — the recurrence answers this in O(n).
2. **In what order are they eliminated?** — only the simulation answers this, in O(n²).

These are two different queries on the same process, and they have two different optimal algorithms. This pattern recurs throughout the noosphere:

- Git **blame** (who touched this line last?) vs git **log** (the whole history).
- Agent-dispatch: "who gets this job?" (a single decision) vs "how did we assign yesterday's 10,000 jobs?" (the full log).
- Final balance of an account (one number) vs the ledger (every movement).

One answer is a reduction over a process. The other answer *is* the process. Choose the algorithm to match the query, not the query to match the algorithm.

## Insight: The Answer Is the Fixpoint of Iterated Reduction

Simulation viewed as a fold:

```
survivor(n, k) = foldr_until_singleton(
    ring = [0..n),
    step = remove the position (pos + k - 1) mod |ring|,
    pos  = pos advances naturally as elements are removed
)
```

The answer is not derived from the ring; the answer *is* the final state of the ring. This is the first Rosetta Stone project where the output is not a computed property of a fixed input but the terminal point of a transformation.

This generalises. Consensus algorithms iterate until no message changes a node's state; physics simulations iterate until energy stops decreasing; optimisation iterates until the gradient vanishes. G063 is the minimal discrete instance of "keep reducing until you can't any more."

## Insight: Closed-Form Answers Hide the Process

There is a philosophical edge to the recurrence: it tells you *who* survives, but gives no account of *how*. You cannot point at the recurrence and say "she survived because the eighth elimination removed the person three seats to her left who would otherwise have passed her the fatal count." The recurrence is a statement of fact without narrative. The simulation has narrative and no closed form.

The noosphere carries both. Audit logs, choreography traces, and movement ledgers record the narrative. Computed projections (current balance, who owes what, today's pace) drop the narrative and keep the summary. Neither is more correct. They answer different questions. G063 makes this trade explicit at the smallest possible scale.

## Choreographic Case: Round-Robin with Skips

```innate
(@schedule-watch){
  @team <- @agents/active
  @counter <- @state/schedule-watch-counter

  ;; Every hit, skip k members and assign to the k-th one.
  @k <- 3
  @victim-idx <- (@counter + @k - 1) mod (@team.length)
  @victim <- @team[@victim-idx]

  @assign-shift{agent: @victim, shift: @current-shift}
  @counter <- (@counter + @k) mod (@team.length)
  where {
    team_nonempty: @team.length > 0
  }
}
```

The Josephus traversal applied as an assignment policy: rather than always picking the first available agent (a queue), cycle through the roster skipping every k-th member. Useful for distributing unpleasant shifts fairly across a team.

## Structures

Josephus is the first Rosetta Stone project with **no data structure** — the "structure" is the integer `n` and the modular index. The language does not need a new type; it only needs `%`.

## Resolver Natives

```innate
@josephus{n, k}                 -> Int                    ;; survivor (O(n) recurrence)
@josephus/simulate{n, k}        -> (Int, [Int])           ;; (survivor, elimination-order)
```

## Demo

```innate
(@demo){
  ;; Flavius Josephus's legend: 41 men, every 3rd killed.
  @survivor <- @josephus{n: 41, k: 3}              ;; -> 30

  ;; Full trace for a small case.
  (@s, @order) <- @josephus/simulate{n: 7, k: 3}    ;; -> (3, [2, 5, 1, 6, 4, 0])

  ;; Cross-check: the recurrence equals the simulation's survivor for all n,k.
  @ok <- @for n in 1..50 {
    @for k in 1..6 {
      @josephus{n, k} == (first @josephus/simulate{n, k})
    }
  } .all
}
```

## Where

The survivor MUST be 0-indexed throughout (index arithmetic, not 1-indexed counting); conversion to Flavius Josephus's traditional 1-indexed answer happens at the presentation layer, not inside the algorithm. `k=1` MUST eliminate positions in order `[0, 1, ..., n-2]` and survivor `n-1` (the walk never skips). `n=1` MUST return survivor 0 with an empty elimination order (no walk happens). The recurrence MUST agree with the simulation on every `(n, k)` up to a tested bound — they are two implementations of the same specification and drift between them is a defect in one of them, not a difference of models.
