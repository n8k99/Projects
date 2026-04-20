# G112 — Database Translation

> The Rosetta Stone's twelfth **Databases project**. Introduces **dialect-aware DDL emission** — the problem every team migrating between MySQL, Postgres, and SQLite hits on day one. The *logical* schema is the same across dialects, but the *syntax* differs: `TINYINT` vs `SMALLINT` vs `INTEGER`, `AUTO_INCREMENT` vs `GENERATED ALWAYS AS IDENTITY` vs `AUTOINCREMENT`, `NOW()` vs `CURRENT_TIMESTAMP`. G112 captures the translation rules as pure data — abstract types, dialect enum, render functions per kind — so the same schema definition emits valid DDL for any supported target.

```yaml
id: G112
title: Database Translation
category: databases
requires: [G101-sql-query-analyzer, G105-database-backup-script-maker, G111-erd-creator]
provides: [abstract-type-system, dialect-aware-rendering, cross-dialect-ddl-emission, default-value-dialect-mapping]
```

## Insight: Abstract Types Are Dialect-Neutral

A column's *logical* type (`BigInt`, `Boolean`, `Timestamp`, `Varchar(255)`, `Decimal(10, 2)`) is the portable part. Its *rendering* is dialect-specific. G112 separates the two: `ColumnType` is the abstract kind (plus parameters for Varchar length, Decimal precision/scale); `render(type, dialect)` is the function that turns it into SQL text.

This is the standard ORM pattern — SQLAlchemy's `Column(Integer)`, Django's `IntegerField`, Diesel's `schema!` macro — all store the abstract type and emit dialect-specific SQL. G112 strips that pattern down to its essence.

First Rosetta Stone project where **a domain type has one abstract representation and multiple rendering targets**. G091's rendered document was similar (IR + renderer), but G112 has multiple *render* functions for the same type, selected by a dialect parameter.

## Insight: Type Mapping Loses Information In Some Dialects

Not every mapping is lossless. Postgres distinguishes `SMALLINT` / `INTEGER` / `BIGINT` (2, 4, 8 bytes). SQLite collapses all integers to its single `INTEGER` type (signed 64-bit). Postgres has `BOOLEAN`; SQLite doesn't (stores 0/1 as `INTEGER`). Postgres has `DATE` and `TIMESTAMP`; SQLite stores both as `TEXT`.

G112 accepts this: the logical type expresses intent, the renderer does what the dialect supports. SQLite users get their booleans stored as ints — that's how SQLite works. Applications that need strict boolean semantics must enforce them in application code.

First Rosetta Stone project where **the abstract type is richer than some targets** and the renderer does the best-available mapping. G097's HTML export was richer than what the image-map spec could express (polygons on a rect-only viewer would be lossy); G112 makes lossiness explicit per dialect.

## Insight: Auto-Increment Has Three Different Syntaxes

Every team that migrates between MySQL and Postgres hits this:

| Dialect  | Syntax                                |
|----------|---------------------------------------|
| MySQL    | `INT PRIMARY KEY AUTO_INCREMENT`      |
| Postgres | `INT PRIMARY KEY GENERATED ALWAYS AS IDENTITY` |
| SQLite   | `INTEGER PRIMARY KEY AUTOINCREMENT`   |

And SQLite's syntax only works when the column is the primary key *and* of type `INTEGER`. The renderer knows all three conventions and the SQLite-specific constraint; the caller just says "auto-increment".

First Rosetta Stone project where **a single logical flag produces a conditional syntax** that depends on both dialect *and* other column properties. G105's shell quoting was dialect-like but singular; G112 has conditional interactions (SQLite AUTOINCREMENT only on PK).

## Insight: Default Values Have Their Own Mini-Language

`DEFAULT NOW()` is MySQL. `DEFAULT CURRENT_TIMESTAMP` is Postgres / SQLite. `DEFAULT 'active'` works everywhere. G112 models defaults as an enum: `Literal(string)` for passes-through values, `Now` for the auto-timestamp macro (rendered per dialect).

Adding new default kinds is a new variant plus rendering per dialect. Nothing in the rest of the system changes. This is the **reified-operation** pattern — the default is data (not a string), so the renderer can vary its output by dialect.

First Rosetta Stone project where **an operation that looks like a string is modelled as a tagged value**. G101 had token kinds; G112 has default-value kinds. Same pattern applied to different domains.

## Insight: Column Metadata Composes Via Builder

`DbColumn::new("id", Integer).pk().auto_increment()` — each call returns `self` and sets one flag. Multi-flag columns read naturally: `.not_null().unique()`. This is the fluent-builder pattern from every query library (Diesel, SQLAlchemy, Knex.js).

The Rust version returns `Self` via `mut self`; Python mutates and returns `self`; CL uses `setf` on the struct. Same conceptual result: column construction is declarative, flags compose.

First Rosetta Stone project where **the builder pattern is load-bearing for readability**. Without it, columns become positional constructor calls with eight boolean arguments — unreadable and error-prone.

## Insight: Emission Is Just String Concatenation With Per-Column Rendering

`emit_create_table` is three lines: header, columns joined by `,\n`, footer. Each column rendered independently. No global state, no multi-pass — one deterministic function of `(table, dialect)`.

This makes it trivial to test: same inputs always yield same bytes. The dialect-specific logic lives in `render_type`, `render_default`, `render_column`; `emit_create_table` is just glue.

First Rosetta Stone project where **the outer emission function is pure glue** with all the logic pushed into subordinates. Same as G111's DOT emission; same as G105's bash script emission. The pattern is reliable.

## Choreographic Case: Vault Schema Migrator

```innate
(@vault-schema-migrator){
  @schema <- @erd/to-translator-tables{erd: @vault.schema-erd}

  @on-user-picks-target (@dialect){
    @ddl <- @dbt/emit-schema{tables: @schema, dialect: @dialect}
    @ui/preview-ddl{text: @ddl}
    @when (@user-confirms){
      @db/execute-script{connection: @target-db, script: @ddl}
    }
  }
}
```

Vault's schema migrator converts an ERD into a list of translator tables, lets the user pick a target dialect, previews the DDL, executes on confirmation. Same schema → three dialects → three valid DDL scripts, no branching in user code.

## Structures

```innate
(defenum dialect MYSQL | POSTGRES | SQLITE)

(defenum column-kind
  SMALLINT | INTEGER | BIGINT | TEXT | VARCHAR | BOOLEAN
  | TIMESTAMP | DATE | FLOAT | DECIMAL)

(defstruct column-type
  kind      : ColumnKind
  length    : Int         ;; VARCHAR
  precision : Int         ;; DECIMAL
  scale     : Int)

(defenum default-kind LITERAL | NOW)

(defstruct default-value
  kind    : DefaultKind
  literal : String)

(defstruct db-column
  name            : String
  ty              : ColumnType
  nullable        : Bool
  primary-key     : Bool
  unique          : Bool
  auto-increment  : Bool
  default         : DefaultValue?)

(defstruct table
  name    : String
  columns : [DbColumn])
```

## Resolver Natives

```innate
@dbt/type-small-int{}             -> ColumnType
@dbt/type-integer{}               -> ColumnType
@dbt/type-varchar{length}         -> ColumnType
@dbt/type-boolean{}               -> ColumnType
@dbt/type-timestamp{}             -> ColumnType
@dbt/default-literal{str}         -> DefaultValue
@dbt/default-now{}                -> DefaultValue
@dbt/column{name, ty}             -> DbColumn
@dbt/table{name, columns}         -> Table
@dbt/render-type{type, dialect}   -> String
@dbt/emit-create-table{table, dialect}  -> String
@dbt/emit-schema{tables, dialect}       -> String
```

## Demo

```innate
(@demo){
  @users <- @dbt/table{
    name: "users",
    columns: [
      @dbt/column{name: "id", ty: @dbt/type-integer{}}.pk.auto-inc,
      @dbt/column{name: "email", ty: @dbt/type-varchar{length: 255}}.not-null.unique,
      @dbt/column{name: "active", ty: @dbt/type-boolean{}}.not-null.default(@dbt/default-literal{"1"}),
      @dbt/column{name: "created_at", ty: @dbt/type-timestamp{}}.not-null.default(@dbt/default-now{})
    ]
  }

  @dbt/emit-create-table{table: @users, dialect: "mysql"}
  ;; -> "CREATE TABLE users (
  ;;       id INTEGER PRIMARY KEY AUTO_INCREMENT,
  ;;       email VARCHAR(255) NOT NULL UNIQUE,
  ;;       active TINYINT(1) NOT NULL DEFAULT 1,
  ;;       created_at DATETIME NOT NULL DEFAULT NOW()
  ;;     );"

  @dbt/emit-create-table{table: @users, dialect: "postgres"}
  ;; Same logical schema, Postgres syntax — BOOLEAN, TIMESTAMP, GENERATED IDENTITY, CURRENT_TIMESTAMP.
}
```

## Where

Abstract types MUST be dialect-neutral — the same `ColumnType.Integer` renders correctly for every supported dialect. Lossy mappings (BigInt → SQLite's single INTEGER type) MUST be documented and accepted — SQLite users understand their storage; forcing lossless would break SQLite support entirely. Auto-increment syntax MUST be selected from a closed set of three — `AUTO_INCREMENT` / `GENERATED ALWAYS AS IDENTITY` / `AUTOINCREMENT` — and SQLite's constraint (PK + INTEGER only) MUST be respected. Default `NOW()` MUST render per-dialect, NOT pass through verbatim — MySQL uses `NOW()`, Postgres and SQLite use `CURRENT_TIMESTAMP`. Primary key MUST imply `NOT NULL` without emitting both — SQL allows `PRIMARY KEY NOT NULL` but it's redundant and ugly. Column construction MUST use the fluent-builder pattern — positional constructors with many booleans are unreadable and error-prone.
