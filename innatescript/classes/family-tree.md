# G064 — Family Tree Creator

> The Rosetta Stone's first **self-referential graph** — a type whose most important field is a list of references to other instances of the same type. Not a tree. A DAG. And the first project where an invariant must be enforced across the **entire transitive closure** rather than only the directly-touched entities.

```yaml
id: G064
title: Family Tree Creator
category: classes
requires: [G053-library-catalog, G056-image-gallery, G063-josephus]
provides: [self-referential-graph, transitive-closure-invariant, graph-bfs, bounded-traversal, set-algebra-over-traversals]
```

## Insight: The Class References Itself

Every prior Classes project had a root type whose fields were primitives, collections, or references to *other* types. Person is the first type in the milestone whose most important field is `List<PersonId>` — a reference back into the same collection. The type is self-referential; the graph is in the data, not beside it.

This is load-bearing. Conversations reference other conversations (reply chains). Documents reference other documents (citations, wiki-links, transclusions). Tasks reference other tasks (blockers, subtasks, supersedes). Every recursive structure in the vault has the same shape as `Person.parents: List<PersonId>`. G064 isolates the primitive.

Two implementation choices available, one taken:

- **Reference fields on the node** (what G064 does): `Person.parents: List<PersonId>`. Direct. Duplicative if both directions are stored (parents AND children). Risk of inconsistency if one direction drifts from the other.
- **External edge list** (not taken here): a separate `ParentChild` relation. Normalised. Adds query cost but removes duplication. This is the SQL-shaped answer and the one a production genealogy service would use.

G064 takes the first option and *derives* the reverse index (`children`) as a maintained cache, reconciled on every `set_parents`. This is a deliberate teaching choice: one representation is primary, one is secondary, and the secondary must stay consistent with the primary. Every implementation language rebuilds the children index the same way.

## Insight: "Family Tree" Is a Misnomer — It's a DAG

A tree has one root and one path to each node. A family pedigree has multiple roots (the original immigrants, the earliest recorded ancestors) and multiple paths to shared ancestors (cousins marrying, for instance, or large families joined by generations). The correct data structure is a **directed acyclic graph**, and the correct term is "pedigree" or "family pedigree graph." Everyone still calls it a family tree.

The code has to deal with the DAG reality: BFS must track seen-set to avoid re-visiting the same ancestor twice (an ancestor can be reachable through multiple paths). Set algebra is the natural query language — intersection for common ancestors, symmetric difference for "relatives of A but not B," and so on.

This is the same lesson as the noosphere's wiki-links: a link graph is not a tree, and tools that assume tree structure (breadcrumbs, "parent" folders) fail silently on the real shape of the data. G064 presents the lesson minimally.

## Insight: Cycle Prevention as a Transitive Closure Invariant

When the user tries to make P a parent of C, the code must check: is P already a descendant of C? Equivalently, is C already an ancestor of P? The answer requires walking the entire reachable graph — not just the proposed edge, not just the immediate neighbours — and refusing if the proposed edge would close a cycle.

This is the first Rosetta Stone project where an invariant is enforced across the **transitive closure** of the structure, not across its direct neighbours. G052 checked that a transfer's entries summed to zero (local invariant on a single transaction). G061 checked that a cart belonged to a valid customer (local invariant on one edge). G064 checks that an edge addition does not create a cycle anywhere in the reachable graph (global invariant on the whole structure).

Transitive invariants are expensive. G064's check is O(n) per `set_parents`; if the graph has millions of nodes the cost is visible. Real genealogy databases maintain precomputed ancestor/descendant sets, invalidated on edit — the price of cheap reads is expensive writes. The choice between "compute on read" and "materialise and maintain" is one every graph-backed system makes. G064 does "compute on read" for teaching clarity.

Analogous invariants in the noosphere:
- Task blockers: adding "A blocks B" when B already transitively blocks A would deadlock the project. Must be refused.
- Citation graphs: self-citation chains are meaningless; if a paper cites itself through intermediaries, the graph is broken.
- Choreography dependencies: `@step/foo requires @step/bar` where `@step/bar` requires `@step/foo` is a logical error.

All three have the same shape as G064's cycle check, and all three demand the same transitive closure walk.

## Insight: Traversal Queries Are the Natural API

Family trees aren't queried by field lookups. They're queried by traversal:

- `ancestors(me)` — transitive closure over parents.
- `descendants(me)` — transitive closure over children.
- `common_ancestors(a, b)` — intersection of two ancestor sets.
- `siblings(me)` — children of parents, minus self.
- `cousins(me, n)` — descendants of grandparents-at-distance-n, minus direct ancestors/descendants.

Each query is a BFS with a particular expansion rule. The expansion rules are independent of the search algorithm — G064's BFS is parameterised on a successor function and a depth bound. This is the first Rosetta Stone project where **search is the primary interface**, not indexed lookup.

The noosphere uses the same shape for wiki-link traversal, conversation reply-chain walking, and task-blocker resolution. The Rust implementation's `bfs` function and Go's `bfs` method can be copied verbatim into every graph-shaped domain. G064 presents the primitive.

## Insight: Bounded Traversal as Cost Budget

`ancestors(me, max_depth=3)` is the first traversal in Classes with a **cost budget built into the query**. The user asks not "who are all my ancestors?" but "who are my ancestors, but only back three generations." The search terminates at depth 3 even if more ancestors exist beyond.

Cost-budgeted traversal is the default in any system with unbounded graphs:
- Web crawls bounded by max-depth or max-page count.
- Federated social feed walks bounded by hop distance.
- Git `log --max-count` and `log --max-depth` on merge commits.
- LLM context-window limits (a crude depth bound on conversation history walks).

G064 introduces the primitive at minimal scale. The `max_depth` parameter is optional — pass `None` / `-1` / `nil` for unbounded — but its presence in the API signals "this query can be expensive, and the caller may cap the cost."

## Insight: Set Algebra Over Computed Sets

`common_ancestors(a, b) = ancestors(a) ∩ ancestors(b)` is the first Rosetta Stone project where **set algebra operates on sets that come from algorithms, not from pre-materialised fields**. G056 Image Gallery did set algebra on tag sets — but the tag sets were already there, as data. G064's ancestor sets don't exist until the BFS runs; set algebra is a composition layer over traversal.

This is how most real graph queries work. "Authors I've co-written with AND who've cited Paper X": the first set is a BFS on the co-authorship graph; the second is a BFS on the citation graph; the intersection answers the query. Neither set is stored. Both are computed on demand. G064 makes the compositionality explicit.

## Choreographic Case: Notification of Mutual Connections

```innate
(@notify-mutual-connection){
  @me <- @user/self
  @them <- @user/target

  @my-tree <- @people/ancestors{id: @me, max_depth: 4}
  @their-tree <- @people/ancestors{id: @them, max_depth: 4}
  @common <- @my-tree ∩ @their-tree

  where {
    actually_related: @common.size > 0
    not_too_distant: @common.size >= 1 and @common.size <= 20
  }

  @draft-message{
    to: @them,
    body: @template/mutual-ancestor{common: @common.names}
  }
}
```

The choreography is natural because the domain concepts and the code concepts are the same: ancestors, common ancestors, related-ness as a set-size threshold.

## Structures

```innate
(defstruct person
  id       : Int
  name     : String
  birth    : Int?
  death    : Int?
  parents  : [PersonId])    ;; at most 2 distinct ids

(defstruct family-tree
  persons  : {PersonId -> Person}
  children : {PersonId -> [PersonId]}       ;; maintained reverse index
  next-id  : Int)
```

## Resolver Natives

```innate
@people/new                                          -> FamilyTree
@people/add{name, birth?, death?}                    -> (FamilyTree, PersonId)
@people/set-parents{child, parents}                  -> FamilyTree   ;; throws on cycle
@people/ancestors{id, max_depth?}                    -> [PersonId]
@people/descendants{id, max_depth?}                  -> [PersonId]
@people/common-ancestors{a, b}                       -> [PersonId]
@people/siblings{id}                                 -> [PersonId]
```

## Demo

```innate
(@demo){
  @t <- @people/new
  (@t, @alice) <- @t.add{name: "Alice", birth: 1930}
  (@t, @bob)   <- @t.add{name: "Bob",   birth: 1928}
  (@t, @carol) <- @t.add{name: "Carol", birth: 1935}
  (@t, @dave)  <- @t.add{name: "Dave",  birth: 1933}
  (@t, @eve)   <- @t.add{name: "Eve",   birth: 1955}
  (@t, @frank) <- @t.add{name: "Frank", birth: 1960}
  (@t, @gina)  <- @t.add{name: "Gina",  birth: 1985}

  @t <- @t.set-parents{child: @eve,   parents: [@alice, @bob]}
  @t <- @t.set-parents{child: @frank, parents: [@carol, @dave]}
  @t <- @t.set-parents{child: @gina,  parents: [@eve, @frank]}

  @t.ancestors{id: @gina}                        ;; -> [Eve, Frank, Alice, Bob, Carol, Dave]
  @t.ancestors{id: @gina, max_depth: 1}          ;; -> [Eve, Frank]
  @t.common-ancestors{a: @eve, b: @gina}         ;; -> [Alice, Bob]

  ;; Cycle prevention
  @t.set-parents{child: @alice, parents: [@gina]}   ;; -> error: would create cycle
}
```

## Where

A person MUST have at most 2 parents — biology's limit, enforced at the edge. A person MUST NOT be their own parent (direct self-parent refused; indirect self-parent caught by the cycle check). `set_parents` MUST check the proposed parent list against the child's entire descendant set — any parent already reachable downward from the child means the edge would close a cycle, and the operation MUST fail. The children index MUST be reconciled on every `set_parents` — removing child from the old parents' children lists, adding to the new parents'. Traversal queries MUST deduplicate via a seen-set — the graph is a DAG, and the same ancestor can be reachable through multiple paths. Bounded traversal (`max_depth`) MUST cut off BFS expansion, not post-filter the result — the cost budget exists to bound work, not just output.
