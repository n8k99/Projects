# G104 — Report Generator

> The Rosetta Stone's fourth **Databases project**. Introduces the **structured report** — a list of typed rows (Header, Data, Subtotal, Total) derived from a stream of records. This is the output shape every accounting package, analytics dashboard, and BI tool produces: data rows under group headers, subtotals beneath each group, grand totals at the bottom. G104 formalises the row-kind tagging + column-definition + aggregation pipeline that makes the pattern reusable.

```yaml
id: G104
title: Report Generator
category: databases
requires: [G088-sort-file-records, G089-transaction-averages, G102-remote-sql-tool, G103-card-collector]
provides: [structured-report, row-kind-tagging, grouped-subtotals, column-with-formatter]
```

## Insight: Report Is a List of Kind-Tagged Rows

A report isn't a 2D array. It's a **list of rows**, where each row has a `kind` tag — `Header` (column names), `Data` (one record), `Subtotal` (aggregation of a group), `Total` (grand total). UIs dispatch on kind: bold the header, indent the data, emphasise the subtotal, underline the total.

First Rosetta Stone project where **the output format is a sequence of typed rows, not homogeneous data**. G094's logs had uniform entries; G091's PDF had lines but all the same kind. G104's rows each carry their semantic role — the consumer doesn't guess.

## Insight: Column Definitions Are First-Class

A `ColumnDef` declares:
* `name` — the header label.
* `field` — the record key to pull from.
* `kind` — plain or summable.
* `format_cents` — render integers as `$X.XX`.

This is the template that every report system ships — pandas' column specs, Excel's pivot table column config, Tableau's field bindings. The report engine is fixed; the columns are data passed in.

First Rosetta Stone project where **the output schema is declarative**. G095's Excel had cells-by-address; G104 has columns-as-objects. The declarative approach scales — adding a new column is adding a `ColumnDef`, not modifying engine code.

## Insight: Subtotals Are Per-Group Aggregations Emitted Inline

Naïve grouping produces one row per group with aggregates. G104's reports emit **data rows under each group PLUS a subtotal row** — the user sees both the detail and the summary in one pass. This is `GROUP BY WITH ROLLUP` in SQL, `pandas.groupby().sum()` with the data intact, every accounting ledger's layout.

Subtotals accumulate per-group; grand totals accumulate across all groups. If no grouping is specified, subtotals are skipped but the grand total still appears (as long as aggregate columns exist).

First Rosetta Stone project where **an aggregate is emitted alongside the data it aggregates**. G089's group-by returned only aggregates; G104 preserves the source rows AND adds aggregates — the hybrid view.

## Insight: Grand Total Label Is in the First Cell

When the first column is a grouping or label column, the Total row puts "TOTAL" in the first cell and the summed values in aggregate columns. Non-aggregate columns in the middle are blank. This matches spreadsheet conventions: the label anchors the row, the numbers line up under their headers.

First Rosetta Stone project with **label-plus-values row layout**. G091's PDF had rendered lines with no structural roles; G104's row cells map 1:1 to columns and carry labels in their natural position.

## Insight: Currency Formatting Is a Column Property

A `ColumnDef` with `format_cents=true` renders `1500` as `$15.00`. Without the flag, the same integer renders as `1500`. This is per-column because only some columns are money (revenue, price) while others are raw counts (qty, row_id).

The formatting is the **column's** concern, not the data's or the caller's. The integer stays integer in the record; the formatter runs once per cell during report generation.

First Rosetta Stone project where **formatting is attached to the column schema**. G094's logs had per-row formatting; G104's formatting is per-column, applied uniformly across the column.

## Choreographic Case: Vault Sales Dashboard

```innate
(@vault-sales-dashboard){
  @records <- @vault/query{table: "orders", year: @current-year}
  @spec <- @report/spec{
    title: "Sales by Region",
    columns: [
      {name: "Region", field: "region", kind: "plain"},
      {name: "Product", field: "product", kind: "plain"},
      {name: "Qty", field: "qty", kind: "sum_aggregate"},
      {name: "Revenue", field: "revenue", kind: "sum_aggregate", format_cents: true}
    ],
    group_by: "region"
  }
  @report <- @report/generate{records: @records, spec: @spec}
  @ui/render-report{report: @report}

  @on-export-csv {
    @vault/save{path: "exports/sales-${@current-year}.txt",
                 content: @report/render{report: @report}}
  }
}
```

The vault's sales dashboard generates a grouped report; UI renders it with kind-aware styling; export writes the tab-delimited text. Same report data feeds both views.

## Structures

```innate
(defenum value-kind INT | TEXT | NULL)

(defstruct value
  kind     : ValueKind
  int-val  : Int
  text-val : String)

(defstruct record
  fields : {String -> Value})

(defenum column-kind PLAIN | SUM_AGGREGATE)

(defstruct column-def
  name          : String
  field         : String
  kind          : ColumnKind
  format-cents  : Bool)

(defenum row-kind HEADER | DATA | SUBTOTAL | TOTAL)

(defstruct report-row
  kind        : RowKind
  cells       : [String]
  group-label : String?)

(defstruct report-spec
  title     : String
  columns   : [ColumnDef]
  group-by  : String?)

(defstruct report
  title : String
  rows  : [ReportRow])
```

## Resolver Natives

```innate
@report/spec{title, columns, group-by}             -> ReportSpec
@report/generate{records, spec}                    -> Report
@report/render{report}                             -> String
```

## Demo

```innate
(@demo){
  @records <- [
    {region: "East", product: "Widget", qty: 3, revenue: 1500},
    {region: "East", product: "Gadget", qty: 2, revenue: 2000},
    {region: "West", product: "Widget", qty: 5, revenue: 2500},
    {region: "West", product: "Gizmo",  qty: 1, revenue: 750}
  ]
  @spec <- @report/spec{
    title: "Sales Report",
    columns: [
      {name: "Region", field: "region", kind: "plain"},
      {name: "Product", field: "product", kind: "plain"},
      {name: "Qty", field: "qty", kind: "sum_aggregate"},
      {name: "Revenue", field: "revenue", kind: "sum_aggregate", format_cents: true}
    ],
    group_by: "region"
  }
  @report/render{report: @report/generate{records: @records, spec: @spec}}
  ;; Sales Report
  ;; # Region  Product  Qty  Revenue
  ;;   East    Widget   3    $15.00
  ;;   East    Gadget   2    $20.00
  ;; ~ East subtotal    5    $35.00
  ;;   West    Widget   5    $25.00
  ;;   West    Gizmo    1    $7.50
  ;; ~ West subtotal    6    $32.50
  ;; = TOTAL            11   $67.50
}
```

## Where

Every output row MUST carry its kind (Header/Data/Subtotal/Total) — UIs need to style them differently, and deriving the kind from position is fragile (subtotals may be nested, totals may appear more than once in multi-level reports). Subtotal MUST include the group key in a visible cell — readers scanning a long report need to know what a subtotal summarises. Grand total MUST appear iff at least one column is a summable aggregate — no aggregates means no meaningful total. Cents formatting MUST be per-column via `format_cents` flag — mixed-type reports need revenue as `$X.XX` and qty as raw integer in the same output. Groups MUST be ordered by key for determinism — otherwise test output flaps. Missing records (no matching group-by field) MUST fall into a `(none)` bucket, NOT crash — real data has holes. Empty record list MUST produce Header-only output (and Total if aggregates declared with zero values), NOT a null report.
