# G101 — SQL Query Analyzer

> The Rosetta Stone's seventeenth project and **first Databases project**. Opens the category with the foundational move: **a query is an AST**. G101 builds a recursive-descent parser for a minimal SELECT grammar, then a static analyser that walks the AST against a known schema. Separating parse from analyse is the contract every real SQL engine (Postgres, SQLite, MySQL, DuckDB) satisfies — and enables every downstream feature the Databases category will build on.

```yaml
id: G101
title: SQL Query Analyzer
category: databases
requires: [G042-lexical-analyser, G071-page-scraper, G085-quiz-maker]
provides: [recursive-descent-parser, query-ast, static-analysis, schema-aware-validation]
```

## Insight: Parse Then Analyse Is the Universal Pipeline

Every compiler, every linter, every SQL engine follows the same shape:
1. **Tokenise** — bytes → tokens.
2. **Parse** — tokens → AST.
3. **Analyse** — AST + context → findings.

G101 implements all three. The tokeniser handles keywords (case-insensitive), identifiers, numbers, strings (with single quotes), operators (`=`, `<=`, `>=`, `!=`, `<`, `>`), and punctuation (`,`, `*`, `;`). The parser is recursive descent — one function per grammar rule, no framework, no parser generator. The analyser walks the AST and emits findings.

First Rosetta Stone project with the full **tokenise → parse → analyse** pipeline as three explicit phases. G071 did forgiving HTML tag extraction but not structural parsing; G085 parsed a line-based custom format but didn't build an AST. G101 is the first where a real grammar produces a typed tree.

## Insight: Recursive Descent Is One Function Per Rule

The grammar:
```
query      ::= 'select' projection 'from' identifier [where] [order_by] [limit]
projection ::= '*' | identifier (',' identifier)*
where      ::= 'where' identifier op value
order_by   ::= 'order' 'by' identifier ['asc' | 'desc']
limit      ::= 'limit' number
```

Becomes:
```
parse_query()        { expect 'select'; projection(); expect 'from'; identifier(); optional where(); ... }
parse_projection()   { if star, consume; else parse_identifier() comma-separated }
parse_where()        { identifier; op; value }
parse_order_by()     { identifier; optional 'asc'/'desc' }
```

One function per nonterminal. Each function consumes tokens from a shared position pointer, returns an AST node (or error). The Rosetta Stone commits to this style because it maps directly into every language — no parser combinators, no lex/yacc, just functions.

First Rosetta Stone project where **grammar rules map 1-to-1 to parser functions**. This is the same organisation used in the Pratt parser in Rust's compiler, Python's ast module, and every hand-written SQL parser in production.

## Insight: AST Is Data, Analysis Is a Second Pass

The `SqlQuery` struct has no methods for "is this correct?" — it's pure data. All validation happens in `analyse(query, schema)`, a separate function. That split is load-bearing:
* Parser errors are **syntax errors**: "I don't know what you meant".
* Analyser errors are **semantic errors**: "I understood but it's wrong (unknown column, etc.)".

Keeping them separate means tooling can choose to parse without analysing (IDE indentation, formatter), analyse without re-parsing (caching), or do both. The same AST feeds every use case.

First Rosetta Stone project where **the data structure has no methods that judge itself**. G093's tag store had a query method; G097's shape had `contains`. G101's `SqlQuery` has no methods — analysis is a free function.

## Insight: Findings Are Data, Not Panics

Every analysis error is a `Finding` — a struct with `severity`, `code`, `message`. Multiple findings per query are normal. Severities ladder: Info (observation), Warning (probably wrong), Error (definitely wrong). Codes are stable identifiers (`UNKNOWN_TABLE`, `SELECT_STAR`, `LIMIT_WITHOUT_ORDER`) so tooling can filter/suppress specific lints.

This is the pattern clippy, ESLint, pylint, SQLFluff, and every real linter uses. The output is data a UI can render however it wants — underline in the editor, flag in CI, group in a report.

First Rosetta Stone project where **errors are first-class values with codes and severities**. G085 returned a single `ParseError`; G101 returns a list of `Finding` with severity levels. That shift — from "one fatal error" to "a prioritised list of issues" — is what every production linter does.

## Insight: Schema Context Turns Syntax Into Semantics

Without a schema, the parser can tell you `SELECT name FROM users WHERE id = 42` is well-formed, but can't tell you whether `users` exists or whether `name` is a real column. The schema provides that context.

G101's `Schema` is a map from table name → column set. The analyser passes both the parsed query and the schema, and findings like `UNKNOWN_TABLE` only fire when the schema says so. This is exactly how Postgres resolves names at planning time, how every ORM validates at query-build time, and how SQL LSPs provide completion.

First Rosetta Stone project where **validation is parameterised by external context**. G092's bulk renamer had a directory as context; G101's schema is more abstract — a dict of what's known. Future Database projects (G102+) will compose over this schema.

## Choreographic Case: Vault Query Linter

```innate
(@vault-query-linter){
  @schema <- @db/introspect-schema{connection: @db}
  @on-user-edits-query (@text){
    @result <- @try { @sql/parse{text: @text} }
    @when (@result.error){ @ui/show-parse-error{error: @result.error}; @return }
    @analysis <- @sql/analyse{query: @result.query, schema: @schema}
    @for finding in @analysis.findings {
      @ui/render-finding{severity: @finding.severity, message: @finding.message}
    }
  }
}
```

As the user types, the vault lints their query: parse errors surface immediately, semantic findings (unknown columns, SELECT *) surface after parse succeeds. No server round-trip — the whole pipeline runs client-side.

## Structures

```innate
(defenum token-kind WORD | NUMBER | STRING | OP | COMMA | STAR | SEMI)

(defstruct token
  kind   : TokenKind
  word   : String
  number : Int
  string : String
  op     : String)

(defenum order ASC | DESC)

(defstruct sql-value
  is-num : Bool
  num    : Int
  text   : String)

(defstruct where-clause
  column : String
  op     : String
  value  : SqlValue)

(defstruct order-by
  column : String
  order  : Order)

(defstruct query
  projection-all     : Bool
  projection-columns : [String]
  table              : String
  where              : WhereClause?
  order-by           : OrderBy?
  limit              : Int?)

(defenum severity INFO | WARNING | ERROR)

(defstruct finding
  severity : Severity
  code     : String
  message  : String)

(defstruct analysis
  tables   : [String]
  columns  : [String]
  findings : [Finding])
```

## Resolver Natives

```innate
@sql/tokenise{input}            -> [Token] | Error
@sql/parse{input}               -> Query | Error
@sql/schema{}                   -> Schema
@sql/schema-add{schema, name, columns} -> Unit
@sql/analyse{query, schema}     -> Analysis
```

## Demo

```innate
(@demo){
  @schema <- @sql/schema{}
  @sql/schema-add{schema: @schema, name: "users",
                   columns: ["id", "name", "email", "created_at"]}

  @q1 <- @sql/parse{input: "SELECT id, name FROM users WHERE id = 42"}
  @sql/analyse{query: @q1, schema: @schema}
  ;; tables=[users], columns=[id, name], findings=[]

  @q2 <- @sql/parse{input: "SELECT * FROM users LIMIT 10"}
  @sql/analyse{query: @q2, schema: @schema}
  ;; findings=[SELECT_STAR, LIMIT_WITHOUT_ORDER]

  @q3 <- @sql/parse{input: "SELECT birthday FROM widgets"}
  @sql/analyse{query: @q3, schema: @schema}
  ;; findings=[UNKNOWN_TABLE, FULL_SCAN]
}
```

## Where

Tokenise MUST be case-insensitive for keywords; identifiers MAY be lowercased too (G101 does, for consistency with most SQL dialects). Parser MUST separate syntax errors (malformed query) from semantic errors (unknown name) — mixing them makes tooling that needs one kind but not the other impossible to build. AST MUST be plain data with no self-validating methods — validation is a second pass by design. Findings MUST include a stable `code` per finding type — codes are the primary key for suppress/filter/aggregate UIs. Severities MUST be ordered (Info < Warning < Error) so UIs can threshold-filter. LIMIT without ORDER BY MUST be flagged Warning, not Error — the query is legal SQL, just likely wrong. SELECT * MUST be Warning, not Error — some tooling legitimately uses it, but most production code shouldn't. Unknown table MUST be Error — there's no legitimate use for referencing a table that doesn't exist.
