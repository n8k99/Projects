# G095 — Excel Spreadsheet Exporter

> The Rosetta Stone's eleventh **Files-category** project. Introduces the **spreadsheet calculation model**: sparse (row, col)-addressed cells with typed values, **lazy formula evaluation with cycle detection**, **range expansion** (`A1:A5`), and a small set of aggregation functions (`SUM`, `AVG`, `MIN`, `MAX`, `COUNT`). The defining novel move: formulae evaluate on read, not on write — which is what makes `=SUM(A1:A10)` update automatically as the underlying cells change. Export format is CSV.

```yaml
id: G095
title: Excel Spreadsheet Exporter
category: files
requires: [G047-dictionary, G071-page-scraper, G088-sort-file-records, G089-transaction-averages]
provides: [cell-addressed-sparse-grid, lazy-formula-evaluation, cycle-detection, range-expansion]
```

## Insight: The Cell Address Is the Primary Key

Everything in a spreadsheet is keyed by `(row, col)`. Not by position in a list, not by name, not by pointer — by coordinates. `A1` means `(0, 0)`. `Z99` means `(98, 25)`. The address is a compound primary key, and every operation either uses one (get a cell) or uses many (expand a range).

This is what makes spreadsheets **forgiving to sparse data**. Most cells are empty; the sheet stores only non-empty cells in a hash map keyed by `(row, col)`. A spreadsheet with 1000 filled cells scattered across a million addresses costs 1000 entries, not a million. Dense grids would be wasteful here.

First Rosetta Stone project where **the primary data structure is a hash keyed by a compound address**. G093's tag store keyed by single string path; G095 uses `(Nat, Nat)` tuples. The compound key is what enables range queries — `A1:A5` is "all addresses (r, 0) for r in 0..=4", expressible only if the address decomposes cleanly.

## Insight: Lazy Evaluation Is What Makes Formulae Formulae

If `=A1+B1` were computed at write time, changing `A1` wouldn't update `C1`. So every spreadsheet ever built **stores the formula, not the result**, and re-evaluates on read.

That's a bigger architectural decision than it looks. It means:
1. **State is minimal** — only leaf values are stored; derived values are computed.
2. **Updates are cheap** — change one cell, everything downstream updates automatically on next read.
3. **Dependencies are implicit** — the formula text mentions `A1` so reading `C1` knows to consult `A1`; no dependency graph is built explicitly.

First Rosetta Stone project where **computation happens at query time, not at write time**. G089's transaction aggregation was eager (compute stats when asked, once); G094's log filter was eager (apply threshold at write). G095 is the first where a cell's value is "compute this right now" every single time. Every reactive framework (React, Vue, signals) generalises this move.

## Insight: Cycle Detection Is a Visit-Set

`A1 = B1 + 1`, `B1 = A1 + 1` would loop forever if evaluated naïvely. The standard trick: **track the set of cells currently being evaluated**; if you re-enter one, return an error.

Each evaluation pushes its cell onto a `HashSet`/`Set` of "visiting" cells before recursing; pops on return. If the lookup says "already visiting", that's a cycle. The set is **per evaluation tree**, not per sheet — two independent `evaluate()` calls each start fresh.

This is the same algorithm used in topological sort, garbage-collection mark phases, Prolog occurs-check, and most cycle-detection problems in computer science. G095 makes it load-bearing.

First Rosetta Stone project where **a visit-set prevents infinite recursion**. G087's filesystem had no cycles (paths can't loop); G079's room graph could loop in principle but the room lookup was simple. G095's formulae are a cyclic dependency graph waiting to happen, and cycle detection is mandatory.

## Insight: Range Expansion Is Address Arithmetic

`SUM(A1:A5)` expands into five evaluations. The range parser reads two addresses, computes all `(r, c)` in the bounding box, and evaluates each. The key move: **ranges are notation, not a new primitive** — they unroll into the existing cell-evaluation path.

This is the same way SQL's `IN (1,2,3)` unrolls into three `=` comparisons, and how array languages (APL, NumPy) treat `a[0:5]` as five independent slot operations. The range is a compact notation for a loop.

First Rosetta Stone project where **notation sugar expands into a uniform operation**. G073's telnet commands dispatched on kind; G085's quiz questions varied kind. G095's ranges are a new kind of argument but reduce to the existing operand-evaluation mechanism. Adding new function names costs nothing — the expansion logic is already there.

## Insight: CSV Is the Universal Interop Format

Excel's native format is XLSX: a zip of XML files, fonts, styles, pivot tables, macros, digital signatures. Implementing it is a multi-year project. CSV is comma-separated values: every program on earth can read it, none of the above features exist.

G095 exports CSV. Why: the Rosetta Stone job is the **calculation model**, not the presentation format. An XLSX writer layers over a CSV-capable sheet, not the other way around. Every real spreadsheet project that wants to export CSV already has a cell grid with formula evaluation — G095 builds that grid.

The quoting rules are small but exact: fields containing `,`, `"`, or `\n` get wrapped in double quotes, with internal `"` doubled (`""`). Anything else passes through. Five lines of code per language.

First Rosetta Stone project where **the export format is chosen for interop breadth over fidelity**. G090's archive was the full payload; G095's CSV is a lossy projection (no formulae, no formatting) because lossiness buys universal readability.

## Choreographic Case: Vault Ledger View

```innate
(@vault-ledger-view){
  @sheet <- @xl/new{}
  @txns <- @vault/read-ledger{year: 2026}
  @for (i, t) in @txns.enumerate {
    @xl/set-number{sheet: @sheet, addr: "A${i+2}", n: @t.amount-cents}
    @xl/set-text{sheet: @sheet, addr: "B${i+2}", s: @t.category}
  }
  @xl/set-text{sheet: @sheet, addr: "A1", s: "amount (cents)"}
  @xl/set-text{sheet: @sheet, addr: "B1", s: "category"}
  @xl/set-text{sheet: @sheet, addr: "D1", s: "Total:"}
  @xl/set-formula{sheet: @sheet, addr: "E1", f: "=SUM(A2:A${@txns.length+1})"}

  @ui/render-grid{sheet: @sheet}

  @on-user-exports {
    @csv <- @xl/to-csv{sheet: @sheet}
    @vault/save{path: "exports/ledger-2026.csv", content: @csv}
  }
}
```

A ledger view populated from the vault: each transaction becomes two cells (amount, category), header cells label the columns, the total cell is a `SUM` formula that updates whenever the user edits a row. Export writes CSV that Excel, Numbers, Google Sheets all open identically.

## Structures

```innate
(defenum cell-kind BLANK | NUMBER | TEXT | FORMULA)

(defstruct cell
  kind    : CellKind
  number  : Float
  text    : String
  formula : String)

(defenum value-kind EMPTY | NUMBER | TEXT | ERROR)

(defstruct value
  kind   : ValueKind
  number : Float
  text   : String)

(defstruct sheet
  cells : {(Int, Int) -> Cell})
```

## Resolver Natives

```innate
@xl/new{}                                    -> Sheet
@xl/set-number{sheet, addr, n}               -> Unit
@xl/set-text{sheet, addr, s}                 -> Unit
@xl/set-formula{sheet, addr, f}              -> Unit
@xl/evaluate{sheet, addr}                    -> Value
@xl/to-csv{sheet}                            -> String
@xl/parse-address{addr}                      -> (Int, Int) | null
@xl/format-address{row, col}                 -> String
```

## Demo

```innate
(@demo){
  @s <- @xl/new{}
  @xl/set-number{sheet: @s, addr: "A1", n: 10}
  @xl/set-number{sheet: @s, addr: "A2", n: 20}
  @xl/set-number{sheet: @s, addr: "A3", n: 30}
  @xl/set-formula{sheet: @s, addr: "B1", f: "=SUM(A1:A3)"}
  @xl/set-formula{sheet: @s, addr: "B2", f: "=AVG(A1:A3)"}
  @xl/set-formula{sheet: @s, addr: "B3", f: "=B1 * 2"}

  @xl/evaluate{sheet: @s, addr: "B1"}   ;; -> 60
  @xl/evaluate{sheet: @s, addr: "B2"}   ;; -> 20
  @xl/evaluate{sheet: @s, addr: "B3"}   ;; -> 120 (depends on B1)

  @xl/to-csv{sheet: @s}
  ;; -> "10,60\n20,20\n30,120\n"
}
```

## Where

Formulae MUST evaluate lazily (on read), NOT eagerly (on write) — what makes `=SUM(...)` update automatically when the underlying cells change is that the formula itself is stored, and evaluation happens every query. Cycle detection MUST be a per-evaluation visit-set — reused evaluations in a single tree recursion must each find their own ancestors, and independent `evaluate()` calls must each start with an empty set. Ranges (`A1:A5`) MUST expand into the existing operand-evaluation path — they are notation sugar; they do NOT need new primitives. Cell storage MUST be sparse (hash of (row, col) → cell), NOT a dense 2D array — spreadsheets are almost always mostly-empty, and a dense array wastes memory on nothing. CSV export MUST quote fields containing `,`, `"`, or `\n` — missing this makes the output unparseable. CSV MUST project evaluated values, NOT raw formula text — the downstream consumer expects data, not `=A1+B1` strings they can't interpret.
