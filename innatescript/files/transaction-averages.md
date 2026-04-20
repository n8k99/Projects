# G089 — Transaction Averages

> The Rosetta Stone's fifth **Files-category** project. Introduces **grouped aggregation** — the `GROUP BY` of the Rosetta Stone. One pass over records, a hash accumulates per-group running totals, a second pass finalises means. Output order is always sorted by key — determinism via explicit sort, not map iteration order. Integer cents preserve exact sums; floats appear only at final mean reporting.

```yaml
id: G089
title: Transaction Averages
category: files
requires: [G001-fibonacci, G076-bookmarks, G088-sort-file-records]
provides: [group-by-aggregation, running-sum-accumulator, integer-cents-arithmetic, per-group-stats]
```

## Insight: Group-By Is a Single-Pass Fold

The naïve aggregation does three passes: extract keys, group rows, aggregate each group. The one-pass version is a single fold: for each record, look up (or create) the group accumulator, observe the value, move on. At the end, finalise each accumulator (compute means, etc). Two passes total, both linear.

The accumulator has two jobs: **running state** (count, sum, min, max updated incrementally) and **finalised output** (mean derived from sum/count at the end). Separating the two is what lets the aggregation be single-pass — min/max/sum are monoids, combinable pairwise without reprocessing; mean is not a monoid but can be computed from sum and count at the end.

First Rosetta Stone project where **observation (`observe`) and finalisation (`finalise`) are explicit separate methods** on the same object. G008 summed with a fold; G065 accumulated progress with atomics. G089 formalises the pattern: an accumulator with an update-step and a finalise-step, together enabling single-pass aggregation.

## Insight: Output Must Be Sorted, Not Map-Ordered

A hash map's iteration order is unspecified (Python 3.7+ guarantees insertion order; most languages don't). Tests that depend on output order become flaky; diffs show spurious reshuffles. Every `group_by` in the Rosetta Stone **sorts group keys** before emitting results — at the final step, after all observation is done.

That's a choice, not a given. Returning an unordered hash would be faster; returning an insertion-ordered list would be cheaper. But the contract — "same input produces identical output every run" — trumps microseconds. Stability is correctness, as it was for G088's sort.

First Rosetta Stone project where **a deterministic wrapper around a non-deterministic data structure** (hash map) is the explicit design. The vault's query layer will use this pattern: internally whatever's fast, externally always sorted.

## Insight: Integer Cents Throughout Kills Float Error

Financial amounts in floating point are a classic bug. `0.1 + 0.2 != 0.3`. Summing a million transactions each off by a fraction of a cent produces a visible error. Every transaction processor since the early 90s stores amounts as integer cents (or integer thousandths, or fixed-point decimal) for this reason.

G089 uses `i64` / `Int` / `int64` throughout. Sum is exact. Min/max are exact. Only `mean = sum / count` produces a float — and only at the reporting step, where the precision loss is bounded by `1/count` cents. Downstream code that re-uses the mean must accept that imprecision; code that needs exact means can compute `(sum, count)` tuples instead.

First Rosetta Stone project with **explicit fixed-point arithmetic for correctness**. G003 and G024 converted between number bases; G013 approximated pi; G089 is the first where the choice of representation matters for *accounting* correctness, not just precision.

## Insight: Key Function, Not Key Field

Grouping could be keyed on a specific field (category, date, name). Instead, `group_by` takes a **key function** — a closure that extracts the key from a record. That means "group by category" and "group by month" (YYYY-MM prefix of date) are the same operation; only the key function changes.

This is the pattern every functional-language `groupBy` uses, and what Python's `itertools.groupby` and SQL's `GROUP BY expression` both codify. It pushes the flexibility up one level: the aggregator is fixed, the extractor is pluggable. Multi-axis grouping, derived keys (year+category), custom bucketing — all expressible as different key functions on the same `group_by`.

First Rosetta Stone project that **parameterises over a function, not a value**. G008's fold took a combiner function; G089's `group_by` takes a key-extractor function. The functional-programming primitive of "take a function as input" is now table stakes.

## Choreographic Case: Vault Finance Ledger

```innate
(@vault-finance-ledger){
  @text <- @vault/read-string{path: "ledger/2026.csv"}
  @txns <- @ta/parse{text: @text}

  @ui/render-summary{
    total:       @ta/total{txns: @txns},
    by-category: @ta/by-category{txns: @txns},
    by-month:    @ta/by-month{txns: @txns}
  }

  @on-user-picks-category (@cat){
    @filtered <- @txns.filter(.category == @cat)
    @ui/render-detail{txns: @filtered, stats: @ta/total{txns: @filtered}}
  }
}
```

The vault's finance pane ingests a ledger CSV, aggregates along multiple axes, and renders summary cards. Detail view filters and re-aggregates. No database needed — the aggregation is cheap enough for a file-scan per request.

## Structures

```innate
(defstruct transaction
  date         : String        ;; YYYY-MM-DD
  category     : String
  amount-cents : Int)

(defstruct stats
  count      : Int
  sum-cents  : Int
  mean-cents : Float
  min-cents  : Int
  max-cents  : Int)
```

## Resolver Natives

```innate
@ta/parse-transaction{line}                 -> Transaction | null
@ta/parse{text}                             -> [Transaction]
@ta/group-by{txns, key-fn}                  -> [(String, Stats)]
@ta/by-category{txns}                       -> [(String, Stats)]
@ta/by-month{txns}                          -> [(String, Stats)]
@ta/total{txns}                             -> Stats
@ta/top-n-by-mean{groups, n}                -> [(String, Stats)]
```

## Demo

```innate
(@demo){
  @sample <- "2026-01-03,groceries,4500\n
              2026-01-10,groceries,3200\n
              2026-01-12,rent,120000\n
              2026-02-01,rent,120000\n
              2026-02-14,groceries,5500\n
              2026-02-22,books,2200\n"
  @txns <- @ta/parse{text: @sample}

  @ta/by-category{txns: @txns}
  ;; -> [("books", count=1 mean=2200),
  ;;     ("groceries", count=3 mean=4400),
  ;;     ("rent", count=2 mean=120000)]

  @ta/top-n-by-mean{groups: @ta/by-category{txns: @txns}, n: 2}
  ;; -> [("rent", 120000), ("groceries", 4400)]
}
```

## Where

Amounts MUST be integer cents, NOT floats — `0.1 + 0.2 != 0.3` and summing thousands of floats visibly drifts. Only the final mean MAY be a float, because the imprecision is bounded and the reporting step can tolerate it. Group output MUST be sorted by key before emission — hash-map iteration order is undefined in most languages and the "same input → same output" contract is more important than saving the sort. The accumulator MUST separate observation from finalisation: single-pass over input, second pass for means. Malformed lines MUST be dropped, NOT error out the whole parse — real ledgers have blank lines, header lines, comments; aborting on one bad line fails the common case. Parse-reject criteria MUST be explicit (missing fields, empty date/category, non-integer amount) — silent acceptance of garbage produces zeros-that-look-like-real-data, which is worse than a skipped row.
