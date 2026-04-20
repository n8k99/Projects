# G088 — Sort File Records Utility

> The Rosetta Stone's fourth **Files-category** project. Treats a file as a **table** — delimited lines become typed rows. Introduces the **sort spec**: a list of `(field, order, kind)` tuples that composes single-axis sorts into multi-key sort. Stable sort is the contract; ties preserve original order. Round-trip still holds without sorting.

```yaml
id: G088
title: Sort File Records Utility
category: files
requires: [G007-list-sorter, G076-bookmarks, G085-quiz-maker, G086-quick-launcher]
provides: [record-as-table, typed-fields, multi-key-sort-spec, stable-sort-contract]
```

## Insight: A File Can Be a Table

The quiz file (G085) and launcher state file (G086) were structured blocks. G088 is the first project where the file is a **uniform tabular dataset** — every row has the same fields, interpreted the same way. This is the contract CSV and TSV have served for fifty years: fixed columns, one record per line, delimited by a single character.

No schema is declared; the parser infers types per field value. That is both cheap and fragile. It's cheap because anyone can read any delimited file without writing a schema; it's fragile because embedded commas or quoted strings break it. G088 accepts the fragility — real CSV handling needs a library. The point here is the tabular *abstraction*, not a production-ready parser.

First Rosetta Stone project where **each row has a uniform type** rather than a polymorphic kind. G085's quiz had three question kinds dispatched by tag; G088 rows are identical structures. That shift — from polymorphic items to uniform rows — is what enables `sort_by(spec)` to work over every row with the same logic.

## Insight: Sort Spec Composes Primitives

Single-key sort is obvious: sort by one column, ascending. Multi-key sort is the useful part: "order by age, break ties by score descending, final tiebreak by name". The spec is a list of `(field, order, kind)` tuples; the comparator walks the spec and returns the first non-zero comparison.

That composition is the same pattern SQL's `ORDER BY` clause has. It works because:
1. Comparison is a total order (returns -1/0/1).
2. Composition of comparisons is a total order.
3. Stable sort preserves insertion order on ties — so unspecified axes don't randomly reshuffle.

First Rosetta Stone project where **a data structure describes a sort operation**. Previous projects had implicit sort keys (G076 used axis constants). G088 makes the sort explicit and configurable — a spec is data, which means it can be stored, passed between layers, and serialised if needed.

## Insight: Stable Sort Is the Contract, Not a Convenience

A stable sort preserves the relative order of records with equal keys. The spec `[age asc, score desc]` applied to a list that's already ordered by `name asc` will, within ties on both age and score, preserve name order. An unstable sort would randomise that final order on every run.

For a file utility, stability is the difference between "run it twice, get the same file" and "run it twice, get different files". Version control, diffs, audit logs all depend on deterministic output. **Stable sort is correctness**, not an optimisation choice.

First Rosetta Stone project where **determinism of output** is an explicit contract. Each language uses its stable-sort primitive (Rust `sort_by`, Python stable sort, Go `sort.SliceStable`, CL `stable-sort`, Lean `mergeSort`). The contract is identical across all six.

## Insight: Missing Fields Sort First

Real datasets have holes. A record with only two fields when the spec asks for field 3 must still sort — and it can't crash. The convention: missing fields sort first in ascending order (and last in descending). Equivalent to `ORDER BY col ASC NULLS FIRST` in SQL.

That decision is minor but load-bearing. It means a sort over partial data still produces a total ordering; it means partial records cluster together so the UI can style them (faded, flagged); it means the invariant "sort is total" never breaks.

First Rosetta Stone project where **a default policy for missing data** is part of the contract. G085 returned ParseError; G086 returned LaunchError. G088's choice is different — missing is valid, just orders specially. Sometimes the right answer is to define the behaviour, not error out.

## Choreographic Case: Vault Record Table

```innate
(@vault-record-table){
  @text <- @vault/read-string{path: "tables/people.csv"}
  @table <- @sfr/parse{text: @text, has-header: true}

  @on-user-clicks-column (@col-name){
    @idx <- @sfr/field-index{table: @table, name: @col-name}
    @sfr/sort-by{table: @table, spec: [{field: @idx, order: "asc", kind: "str"}]}
    @ui/render-rows{rows: @table.rows}
  }

  @on-user-shift-clicks-column (@col-name){
    ;; Add to spec — multi-key sort
    @idx <- @sfr/field-index{table: @table, name: @col-name}
    @sfr/sort-by{table: @table, spec: [...@current-spec,
                                        {field: @idx, order: "asc", kind: "str"}]}
    @ui/render-rows{rows: @table.rows}
  }
}
```

The vault's tabular views layer a UI over the sort primitive. Column click replaces the spec; shift-click appends. The sort itself is oblivious to UI — it just takes a spec, reorders rows, emits result. Every feature users expect (multi-column sort, direction indicators, null ordering) composes from the spec data structure.

## Structures

```innate
(defenum order ASC | DESC)
(defenum kind STR | INT)

(defstruct sort-key
  field : Int
  order : Order
  kind  : Kind)

(defstruct record
  fields : [Field])           ;; Field = Int | String

(defstruct table
  header : [String] | null
  rows   : [Record])
```

## Resolver Natives

```innate
@sfr/parse-record{line}                          -> Record
@sfr/serialise-record{record}                    -> String
@sfr/parse{text, has-header}                     -> Table
@sfr/serialise{table}                            -> String
@sfr/sort-by{table, spec}                        -> Unit
@sfr/field-index{table, name}                    -> Int | null
```

## Demo

```innate
(@demo){
  @sample <- "name,age,score\nAlice,30,85\nBob,25,92\nCarol,30,78\nDave,25,88\n"
  @t <- @sfr/parse{text: @sample, has-header: true}

  @sfr/sort-by{table: @t, spec: [
    {field: 1, order: "asc",  kind: "int"},    ;; age asc
    {field: 2, order: "desc", kind: "int"}     ;; score desc
  ]}
  @t.rows
  ;; -> [Bob,25,92  Dave,25,88  Alice,30,85  Carol,30,78]

  @sfr/serialise{table: @sfr/parse{text: @sample, has-header: true}} == @sample
  ;; -> true — parse+serialise round-trips when unsorted
}
```

## Where

Sort MUST be stable — ties MUST preserve insertion order, because determinism of output is a hard contract for any file utility that feeds version control or audit logs. Multi-key spec MUST be evaluated in order with short-circuit: the comparator returns the first non-zero comparison, NOT a hash of all keys. Missing fields MUST sort first in ascending order (last in descending) — partial records still belong in the output, and clustering them is the least-surprise behaviour. Round-trip MUST hold when unsorted — `serialise(parse(text)) == text` for any valid input, because the parser's job is to losslessly represent the file. Type inference per field is lossy — `"123"` becomes `Int 123` and round-trips as `"123"`, which is fine when interpretation is numeric. The parser does NOT preserve leading-zero distinctions (`"007"` round-trips as `"7"`); projects needing that fidelity would disable inference per column.
