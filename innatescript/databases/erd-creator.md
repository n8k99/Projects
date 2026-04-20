# G111 — ERD Creator

> The Rosetta Stone's eleventh **Databases project**. Introduces the **foreign-key dependency graph** and the algorithms that make schemas manageable: **topological sort** (for presentation order — dependencies first), **cycle detection** (for flagging circular references), and **Graphviz DOT export** (for visualisation). An ERD isn't just a picture — it's a **directed graph** with typed nodes and cardinality-annotated edges, and the graph algorithms that work on it are the ones every schema-design tool (dbdiagram, drawSQL, schemaSpy, Mermaid ERD) applies.

```yaml
id: G111
title: ERD Creator
category: databases
requires: [G056-tree, G101-sql-query-analyzer, G102-remote-sql-tool, G110-travel-planner]
provides: [entity-relationship-model, cardinality-as-enum, topological-sort, cycle-detection, graphviz-dot-export]
```

## Insight: A Schema Is a Typed Directed Graph

Entities are **nodes**; relationships are **edges** with typed endpoints (which attribute on each side) and a cardinality label. The ERD is a directed graph: `from_entity --(from_attr, to_attr, cardinality)--> to_entity`.

Every schema tool — dbdiagram.io, drawSQL, SchemaSpy, prisma's ERD generator — treats the schema this way. Once you have the graph, all the standard graph algorithms apply: reachability, topological sort, cycle detection, dependency closure. G111 makes the graph explicit and runs three of those algorithms on it.

First Rosetta Stone project where **the data model is a general directed graph** (with typed endpoints) rather than a tree, sequence, or map. G100's commit chain was a DAG but linearly structured; G111's ERD can have arbitrary shape.

## Insight: Cardinality Is Enum, Not Magic String

`OneToOne`, `OneToMany`, `ManyToMany` — three values, a sealed set. Not a free-form string that could be "1-1", "1:1", "one_to_one", or "1-*" depending on who typed it. A closed enum eliminates a class of bugs and enables exhaustive handling (e.g., rendering the correct DOT arrowhead per variant).

First Rosetta Stone project where **a domain term is constrained to a small finite enum** specifically to prevent string-representation drift. G101 did this for severity; G111 does it for cardinality. The lesson is the same: if there are three values, the type should have three variants.

## Insight: Topological Sort Orders Dependencies First

`topo_sort()` returns entities in an order where each entity appears *after* everything it depends on. `User` before `Order` before `OrderItem`. Produces a valid SQL schema-creation order (`CREATE TABLE User; CREATE TABLE Order; CREATE TABLE OrderItem;`) — inverted dependency order is load order.

G111 uses **Kahn's algorithm**: start with zero-in-degree nodes (no dependencies), remove them, repeat. If any nodes remain after the algorithm completes, there's a cycle.

First Rosetta Stone project with **a named graph algorithm as a primary operation**. G097's ray casting was classical geometry; G108's union-find was classical set theory; G111's Kahn's algorithm is classical graph theory. Each cross-language implementation is a confidence test of the algorithm's definition.

## Insight: Cycle Detection Is DFS With Path-Membership Tracking

Cycles in FK graphs are bad news — they break topological sort, confuse ORMs, complicate schema migrations. G111 detects them via **DFS with on-path tracking**: as you recurse down the graph, maintain the set of nodes currently on the stack; if you encounter one, you've found a back-edge (cycle).

Self-references (`Node.parent_id -> Node.id` for tree structures) are deliberately **excluded** from cycle detection — they're a legitimate pattern, not a structural error.

First Rosetta Stone project with **graph-colouring-style DFS** for cycle detection. Standard textbook algorithm; worth implementing six times for cross-language confidence.

## Insight: Validation Catches Structural Errors Pre-Render

Before emitting DOT, the validator checks:
* Does every `from_entity` / `to_entity` resolve to a declared entity?
* Does every `from_attr` / `to_attr` resolve to a declared attribute on its entity?

Rendering a DOT file that references non-existent entities would produce a broken diagram (Graphviz might crash, or silently omit nodes). Validation catches these before rendering.

First Rosetta Stone project where **pre-render validation is a distinct phase**. G105 validated configs before emitting scripts; G111 validates the ERD before emitting DOT. Same pattern — validate the input data, then emit.

## Insight: DOT Export Is the Canonical Visualisation Target

Graphviz DOT is the de facto format for directed-graph visualisation: every ERD tool, every UML generator, every CI graph view, every kubernetes dependency viewer speaks DOT. G111 emits it directly — `digraph`, node definitions with record shapes, edges with arrowhead attributes.

The output is plain text that feeds into `dot` (the Graphviz renderer), which produces SVG/PNG/PDF. G111 stops at text; rendering is a one-line pipeline step outside the Rosetta Stone.

First Rosetta Stone project where **the export target is DOT**. G090 was ZIP format; G100 was content-addressed tree hashing. DOT is the graph analog — a text-based, tool-pipeline-friendly serialisation of a well-known data shape.

## Choreographic Case: Vault Schema Diagram

```innate
(@vault-schema-diagram){
  @erd <- @erd/new{}
  @tables <- @db/introspect-schema{connection: @db}
  @for t in @tables {
    @erd/add-entity{erd: @erd, entity: @t}
  }
  @fks <- @db/introspect-foreign-keys{connection: @db}
  @for fk in @fks {
    @erd/add-relationship{erd: @erd, relationship: @fk}
  }

  @issues <- @erd/validate{erd: @erd}
  @when (@issues.length == 0){
    @dot <- @erd/to-dot{erd: @erd}
    @vault/save{path: "schema/erd.dot", content: @dot}
    @shell/run{command: "dot -Tsvg schema/erd.dot -o schema/erd.svg"}
  }

  @cycles <- @erd/find-cycles{erd: @erd}
  @when (@cycles.length > 0){
    @ui/warn{message: "Schema has circular FK references", cycles: @cycles}
  }
}
```

Introspect live database → build ERD → validate → render diagram. Cycles surface as warnings for the DBA to review.

## Structures

```innate
(defenum cardinality ONE_TO_ONE | ONE_TO_MANY | MANY_TO_MANY)

(defstruct attribute
  name        : String
  ty          : String
  primary-key : Bool
  nullable    : Bool)

(defstruct entity
  name       : String
  attributes : [Attribute])

(defstruct relationship
  from-entity : String
  from-attr   : String
  to-entity   : String
  to-attr     : String
  cardinality : Cardinality)

(defstruct erd
  entities      : {String -> Entity}
  relationships : [Relationship])
```

## Resolver Natives

```innate
@erd/new{}                               -> Erd
@erd/add-entity{erd, entity}             -> Unit
@erd/add-relationship{erd, relationship} -> Unit
@erd/dependencies-of{erd, name}          -> [String]
@erd/topo-sort{erd}                      -> [String] | null     ;; null = cycle exists
@erd/find-cycles{erd}                    -> [[String]]
@erd/validate{erd}                       -> [String]
@erd/to-dot{erd}                         -> String
```

## Demo

```innate
(@demo){
  @erd <- @erd/new{}
  @erd/add-entity{erd: @erd, entity: {name: "User",
                                        attributes: [{name: "id", ty: "INT", primary-key: true},
                                                     {name: "email", ty: "TEXT"}]}}
  @erd/add-entity{erd: @erd, entity: {name: "Order",
                                        attributes: [{name: "id", ty: "INT", primary-key: true},
                                                     {name: "user_id", ty: "INT"}]}}
  @erd/add-relationship{erd: @erd,
                         relationship: {from-entity: "Order", from-attr: "user_id",
                                        to-entity: "User", to-attr: "id",
                                        cardinality: "one_to_many"}}
  @erd/topo-sort{erd: @erd}      ;; -> ["User", "Order"]
  @erd/find-cycles{erd: @erd}    ;; -> []
  @erd/to-dot{erd: @erd}          ;; -> "digraph ERD { ... }"
}
```

## Where

Cardinality MUST be a closed enum — three values (1:1, 1:N, N:M) cover every normalised relational schema, and allowing free-form strings invites drift. Topological sort MUST return `None`/`null` on cycle, NOT a partial order — downstream consumers need to distinguish "no cycle" from "cycle exists". Cycle detection MUST exclude self-references (`Node.parent_id -> Node.id`) — tree structures using FK to parent are legitimate, not errors. Validation MUST happen before rendering — a DOT file referencing non-existent entities produces a broken diagram. Dependencies MUST be directed — A depends on B if A has an FK into B; reverse dependencies (what *depends on A*) is a different query. DOT output MUST use `digraph`, not `graph` — FK edges are directional, and treating them as undirected loses essential information about which end points where.
