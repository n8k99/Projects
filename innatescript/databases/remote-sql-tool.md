# G102 — Remote SQL Tool

> The Rosetta Stone's second **Databases project**. Natural companion to G101 (the parser): G102 is the **executor**. Introduces the **in-memory table engine** that every embedded database (SQLite, DuckDB, H2, in-memory Postgres) runs under the hood. A `Connection` owns tables; tables have typed columns; rows are inserted with arity + type validation; `SELECT` filters, projects, sorts, and limits. **Transactions** buffer pending rows until COMMIT (discard via ROLLBACK) — making COMMIT/ROLLBACK atomic at the row level.

```yaml
id: G102
title: Remote SQL Tool
category: databases
requires: [G085-quiz-maker, G088-sort-file-records, G101-sql-query-analyzer]
provides: [in-memory-table-engine, typed-columns, row-storage, transaction-buffer, select-executor]
```

## Insight: Parser + Executor = Database

G101 built the parser — text into AST. G102 builds the executor — AST into rows. Together they form the minimum viable database: take a SQL string, get back a result set. That's what every embedded DB does.

Splitting parse from execute lets each phase be tested independently. The parser doesn't need a database; the executor doesn't need text. They meet at the AST, which is just data.

First Rosetta Stone project where **two prior projects compose into a complete system**. G097 composed with G074 would be a drawable map; G102 composed with G101 is a database. Future Database projects (G103+) layer over this same AST+executor foundation.

## Insight: Typed Columns Prevent Silent Corruption

Every column has a declared type — `Int` or `Text` for G102's minimal engine. INSERT validates: if a cell's type doesn't match its column, the insert is rejected with a structured error. No silent coercion, no "1" stored where 1 was meant.

Real databases extend this with NUMERIC, DATE, JSON, arrays, etc. The **principle** doesn't change: the table declares a shape; inserts that don't match the shape are rejected; SELECTs can rely on the shape.

First Rosetta Stone project where **type safety is enforced at the storage boundary**, not just at the language level. G093's tag store had typed Option fields but didn't reject inserts; G102 rejects with `TypeMismatch` when the promise is broken.

## Insight: Row Storage Is Positional, Not Keyed

A row is a `Vec<Cell>` positioned to match the table's column list. Column name → position via `column_index(name)`. Insert provides values in column order; SELECT rewrites them into the same order.

Why positional? Because:
1. **Fast access** — O(1) by index, no hash lookup per cell.
2. **Compact storage** — no per-row map overhead.
3. **Order is stable** — the table's schema is the canonical order.

Real databases use positional row storage for the same reasons. Column stores (Parquet, Arrow) go further — each column is a separate array, enabling columnar scans.

First Rosetta Stone project where **storage layout is chosen explicitly for access pattern**. G087's filesystem was key-path based; G102 uses positional because rows are the unit of work.

## Insight: Transactions Are a Buffer That Collapses on Commit

`begin()` starts a pending-buffer. INSERTs during the transaction go to the buffer, not the base table. `commit()` moves the buffer into the table's rows; `rollback()` discards it.

SELECTs *inside* the transaction see both the base rows and the pending rows — that's the "read-your-own-writes" guarantee every SQL transaction provides. After COMMIT the pending rows become normal rows; after ROLLBACK they vanish.

This is a minimalist transaction: no isolation levels, no conflict detection, no logging. But the shape — buffer, commit-or-discard — is the kernel every real transaction system extends. Postgres's WAL, SQLite's journal, Redis's MULTI/EXEC all bolt isolation + durability on top of a buffer.

First Rosetta Stone project with **atomic rollback of data mutations**. G082's CMS had revisions (append-only); G092's bulk rename had undo logs (reverse ops). G102 is the first with proper "abandon these changes entirely" semantics.

## Insight: SELECT Is Filter-Project-Sort-Limit in Order

The executor runs SELECT as a pipeline:
1. **Scan** rows of the table (base + pending if in a txn).
2. **Filter** by the WHERE predicate (if any).
3. **Sort** by ORDER BY column (if any).
4. **Project** to the chosen columns.
5. **Limit** to the first N rows (if any).

Sort happens **before** project because the ORDER BY column may not appear in the projection (e.g., `SELECT name ORDER BY age`). Real engines that optimise this (push projection before sort when safe) do so via the query planner — a layer G102 doesn't have.

First Rosetta Stone project where **a multi-stage pipeline has a canonical ordering**. Changing the order (e.g., project then sort) breaks queries that sort on non-projected columns.

## Insight: Result Set Is Data, Not an Iterator

SELECT returns a `ResultSet` — a struct with column names and row list, all in memory. Not a cursor, not a generator, not a streaming iterator.

Why materialise? Because for the Rosetta Stone's purposes, the result fits in memory trivially, and having the full result as data means tests can compare against expected values without special-casing iteration state. Real engines stream for large results; the trade-off is predictability vs. memory.

First Rosetta Stone project where **the output of a query is eager, fully-materialised data**. Consistent with G091's rendered-document model, G094's event lists, G098's copy-events — data the caller owns and can process however.

## Choreographic Case: Vault Notes Database

```innate
(@vault-notes-db){
  @conn <- @db/connect{}
  @db/create-table{conn: @conn, name: "notes", columns: [
    {name: "id", type: "int"},
    {name: "title", type: "text"},
    {name: "body", type: "text"},
    {name: "created_at", type: "int"}
  ]}

  @on-user-saves (@title @body){
    @db/begin{conn: @conn}
    @db/insert{conn: @conn, table: "notes",
                values: [@clock/now, @title, @body, @clock/now]}
    @db/commit{conn: @conn}
  }

  @on-search (@query){
    @db/select{conn: @conn,
               query: {project-all: false, columns: ["id", "title"],
                       table: "notes",
                       order-by: {column: "created_at", desc: true},
                       limit: 20}}
  }
}
```

The vault's note store is a database table; saves are transactional; searches return ordered result sets. No ORM needed — the table engine *is* the note store.

## Structures

```innate
(defenum col-type INT | TEXT)

(defstruct cell
  is-null : Bool
  is-int  : Bool
  int-val : Int
  text-val: String)

(defstruct column
  name : String
  ty   : ColType)

(defstruct table
  name    : String
  columns : [Column]
  rows    : [[Cell]])

(defstruct connection
  tables  : {String -> Table}
  pending : {String -> [[Cell]]}?)

(defstruct where-expr
  column : String
  op     : String          ;; =, !=, <, <=, >, >=
  value  : Cell)

(defstruct order-by
  column : String
  desc   : Bool)

(defstruct select-query
  project-all : Bool
  columns     : [String]
  table       : String
  where       : WhereExpr?
  order-by    : OrderBy?
  limit       : Int?)
```

## Resolver Natives

```innate
@db/connect{}                                   -> Connection
@db/create-table{conn, name, columns}            -> Unit | ExecError
@db/insert{conn, table, values}                  -> Unit | ExecError
@db/begin{conn}                                  -> Unit
@db/commit{conn}                                 -> Unit | ExecError
@db/rollback{conn}                               -> Unit | ExecError
@db/select{conn, query}                          -> ResultSet | ExecError
```

## Demo

```innate
(@demo){
  @c <- @db/connect{}
  @db/create-table{conn: @c, name: "users",
                    columns: [{name: "id", type: "int"},
                              {name: "name", type: "text"},
                              {name: "age", type: "int"}]}
  @db/insert{conn: @c, table: "users", values: [1, "Alice", 30]}
  @db/insert{conn: @c, table: "users", values: [2, "Bob",   25]}
  @db/insert{conn: @c, table: "users", values: [3, "Carol", 35]}

  @db/select{conn: @c,
             query: {project-all: false, columns: ["name"],
                     table: "users",
                     where: {column: "age", op: ">", value: 26},
                     order-by: {column: "age", desc: true},
                     limit: 2}}
  ;; -> [[Carol], [Alice]]

  @db/begin{conn: @c}
  @db/insert{conn: @c, table: "users", values: [4, "Dave", 40]}
  @db/rollback{conn: @c}
  ;; Dave never appears in subsequent SELECTs.
}
```

## Where

Column types MUST be validated on INSERT — silent coercion from text to int is a classic source of production bugs that only surface during SELECT. Transactions MUST support read-your-own-writes — a SELECT inside a transaction MUST see pending INSERTs; this is the weakest isolation level SQL provides and failing it breaks caller expectations. ROLLBACK MUST leave the table in its pre-transaction state — partial cleanup is worse than no cleanup. ORDER BY MUST run before projection — ordering on non-projected columns is legal SQL and must be supported. SELECT on an unknown column MUST error, NOT silently return NULL — typos in column names are the most common source of SELECT bugs. Result sets MUST be fully materialised data — streaming adds complexity Rosetta Stone tests can't verify portably. COMMIT/ROLLBACK without an open transaction MUST error — implicit auto-commits hide state-machine bugs; explicit no-transaction errors surface them.
