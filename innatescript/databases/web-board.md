# G113 — Web Board / Forum

> The Rosetta Stone's thirteenth **Databases project** — and the one that **closes the category**. Introduces the **parent-pointer tree**: posts are stored flat (each with `parent_id: Option<u64>`) and the reply tree is derived on read via DFS. Adds **vote tallies** (score = upvotes − downvotes, can go negative) and the classic **Reddit-style hot score** (`sign(s) * log10(|s|) − age_days * 0.2`) so new content surfaces against old. Every forum, Reddit-style threaded comment system, and email-reply tree uses this exact storage pattern.

```yaml
id: G113
title: Web Board / Forum
category: databases
requires: [G086-quick-launcher, G089-transaction-averages, G102-remote-sql-tool, G111-erd-creator]
provides: [parent-pointer-tree, derived-reply-tree, vote-score, hot-score-ranking, multi-order-thread-sort]
```

## Insight: Parent-Pointer Storage, Derived Tree on Read

A post doesn't store its children — it stores *which post it's replying to*, via `parent_id`. Top-level posts have `parent_id = None`. This is the SQL-native way to represent trees: a single column that references `posts.id`.

The tree shape is **derived on read**: to render a thread, collect all posts with that `thread_id`, DFS-walk from roots, yield `(post, depth)` pairs. No tree structure is stored; only the parent pointers.

Every SQL forum, every email threading system (`In-Reply-To` header), every Reddit comment system uses this layout. It's the unique representation that's both compact and queryable.

First Rosetta Stone project where **a tree structure is implicit in flat data** and rebuilt per query. G087's filesystem had trees as explicit nested nodes; G111's ERD was a graph but stored the adjacency directly. G113's tree lives in the parent pointers — the flat-list representation is the source of truth.

## Insight: DFS With Depth Tracking Is the Tree Walk

`thread_tree(thread_id)` returns `[(post, depth)]` in DFS order. The UI uses `depth` for indentation (2 * depth spaces, or indent lines, or CSS margins).

The DFS is recursive: for each top-level post, push `(post, 0)`, then recurse into its children with depth+1. Pre-order traversal, stable within each level via `created_ms` sort.

First Rosetta Stone project where **tree traversal exposes depth as first-class output**. G087 had recursive tree operations but didn't expose depth; G091 had layout depth implicitly via page offsets. G113 makes depth a value the caller gets — rendering becomes trivial.

## Insight: Score Can Go Negative

Upvotes and downvotes are separate counts. Score is `upvotes − downvotes` — signed. A post with 3 upvotes and 7 downvotes has score −4. The UI may hide deeply downvoted posts, but the model stores the raw counts.

This is the **two-counter pattern** — tracking the positive and negative separately lets the system compute not just score but confidence (high-traffic posts have more data than low-traffic ones, even at the same score).

First Rosetta Stone project where **a derived quantity can be negative** as a deliberate feature. G089 had signed variance (overspent = positive), G107 had variance, but both were monotone. G113's score flips sign based on community reaction.

## Insight: Hot Score Balances Score and Recency

The Reddit-style hot formula: `sign(score) * log10(max(1, |score|)) − age_days * 0.2`.

Breakdown:
* `log10` compresses the score range — a post with 10,000 upvotes isn't 100× a post with 100; it's 2× (log₁₀(10000) vs log₁₀(100) = 4 vs 2).
* `sign(score)` preserves direction — heavily downvoted posts sink.
* Age penalty (`0.2 * days`) makes recency matter — a week-old post loses 1.4 points regardless of score.

The constants (log base 10, weight 0.2) are tunable; the *shape* is universal — log-compress score, subtract linear age. Every "hot" ranking on the internet uses some variant.

First Rosetta Stone project where **log-compression of a metric is a ranking primitive**. G086 used log frecency; G113 uses log score. Both prevent extreme values from dominating and make "order of magnitude" the semantic unit.

## Insight: Pinned Threads Ignore Sort Order

Sorting by NEW, TOP, or HOT gives the user's preferred view — but **pinned** threads always come first. Moderators use pinning to elevate announcements, rules, or megathreads.

G113 implements this as a primary sort key: `(is_pinned ? 0 : 1, sort_metric)`. Pinned always sorts first; within pinned and within non-pinned, the selected metric decides.

First Rosetta Stone project with **a primary sort key that's independent of the user's chosen sort**. G088's multi-key sort composed user-specified keys; G113's pinned-first is a hardcoded override.

## Insight: Parent Must Be Validated

When adding a post, the system checks: does `thread_id` exist? If `parent_id` is set, does the parent post exist? Is the parent in the *same thread*? Without these checks, forum data drifts into nonsense — replies to nothing, cross-thread replies, orphaned trees.

G113's validation runs at insert time, returning errors before the post is stored. This is the database-like constraint pattern — foreign-key integrity enforced by the application rather than the backend store (since G102's in-memory engine doesn't enforce FKs itself).

First Rosetta Stone project where **hierarchical integrity is enforced at write time**. G102 was the equivalent for flat data; G113 extends to parent-child integrity.

## Insight: Closing Databases — Thirteen Patterns

G101 opened with the parser. Every subsequent project built on that vocabulary:

| # | Project | New Pattern |
|---|---------|-------------|
| G101 | SQL Query Analyzer | Tokenise → parse → analyse pipeline |
| G102 | Remote SQL Tool | In-memory engine + transactions |
| G103 | Card Collector | Multiset + trade evaluation |
| G104 | Report Generator | Kind-tagged rows + grouped subtotals |
| G105 | Database Backup Script Maker | Code gen + shell-safe quoting |
| G106 | Event Scheduler | Half-open intervals + recurrence |
| G107 | Budget Tracker | Envelopes + scale-invariant alerts |
| G108 | Address Book | Canonical IDs + union-find dedup |
| G109 | TV Show Tracker | Library/watchlist split + next-up |
| G110 | Travel Planner | Chain-of-edges validation |
| G111 | ERD Creator | Typed graph + topo sort + DOT |
| G112 | Database Translation | Abstract types + dialect rendering |
| G113 | Web Board | Parent-pointer trees + hot-score |

Thirteen vocabulary items every data-heavy app reaches for. Together with Files (G085–G100), they're the backbone of every CRUD-and-query system.

## Choreographic Case: Vault Discussion Board

```innate
(@vault-discussion-board){
  @board <- @wb/load{path: "discussions/board.json"}

  @on-user-visits-front-page {
    @threads <- @wb/list-threads{board: @board, sort: "hot", now-ms: @now}
    @ui/render-thread-list{threads: @threads}
  }

  @on-user-opens-thread (@thread-id){
    @tree <- @wb/thread-tree{board: @board, thread-id: @thread-id}
    @ui/render-tree{tree: @tree}    ;; depth → indent
  }

  @on-user-replies (@parent-id @body){
    @wb/add-post{board: @board, post: {
      thread-id: @parent-post.thread-id,
      parent-id: @parent-id,
      author: @user, body: @body,
      created-ms: @now, upvotes: 0, downvotes: 0
    }}
  }
}
```

Front page lists threads sorted by hot score; opening a thread renders the DFS tree; replies append posts with parent pointers. The board, threads, and tree are all pure data; the UI is a renderer.

## Structures

```innate
(defstruct thread-entry
  id          : Int
  title       : String
  author      : String
  created-ms  : Int
  pinned      : Bool
  locked      : Bool)

(defstruct post
  id          : Int
  thread-id   : Int
  parent-id   : Int?        ;; None = top-level
  author      : String
  body        : String
  created-ms  : Int
  upvotes     : Int
  downvotes   : Int)

(defstruct post-with-depth
  post  : Post
  depth : Int)

(defenum thread-sort NEW | TOP | HOT)

(defstruct board
  threads : {Int -> ThreadEntry}
  posts   : {Int -> Post})
```

## Resolver Natives

```innate
@wb/new{}                                        -> Board
@wb/add-thread{board, thread}                    -> Unit
@wb/add-post{board, post}                        -> Unit | Error
@wb/post-score{post}                              -> Int
@wb/hot-score{post, now-ms}                       -> Float
@wb/thread-tree{board, thread-id}                -> [PostWithDepth]
@wb/top-level-posts{board, thread-id}            -> [Post]
@wb/children-of{board, post-id}                  -> [Post]
@wb/list-threads{board, sort, now-ms}            -> [ThreadEntry]
@wb/post-count{board, thread-id}                 -> Int
```

## Demo

```innate
(@demo){
  @b <- @wb/new{}
  @wb/add-thread{board: @b, thread: {id: 1, title: "Welcome", pinned: true, created-ms: 1_000_000}}
  @wb/add-post{board: @b, post: {id: 10, thread-id: 1, parent-id: null,
                                   author: "alice", body: "hi", created-ms: 1_000_000,
                                   upvotes: 10, downvotes: 1}}
  @wb/add-post{board: @b, post: {id: 11, thread-id: 1, parent-id: 10,
                                   author: "bob", body: "hi alice", created-ms: 1_100_000,
                                   upvotes: 5, downvotes: 0}}
  @wb/thread-tree{board: @b, thread-id: 1}
  ;; -> [(post #10, depth 0), (post #11, depth 1), ...]
}
```

## Where

Parent pointers MUST be validated at write time — `thread_id` must exist; `parent_id` must exist and be in the same thread. Score MUST be `upvotes − downvotes` (signed) — the two-counter pattern preserves confidence data that collapsing to one would lose. Hot score MUST use log-compressed magnitude — linear scores let 10,000-upvote posts dominate forever. Age penalty MUST be linear in days — exponential decay works in some systems but linear is simpler and predictable. Pinned threads MUST ignore sort — moderators' choice overrides user preference. Tree traversal MUST expose depth — UIs need it for indentation, and computing depth after the fact requires re-walking. Top-level posts MUST have `parent_id = None`, NOT `parent_id = 0` or self-reference — `None` is the unambiguous root marker.

## Closing the Databases Category

**Databases 13/13. 113/130 complete.** The category journeyed from query parsing (G101) through execution (G102), multi-entity relationships (G103, G108, G109), reporting (G104), schema migration (G111, G112), chained data (G106, G110), envelope budgeting (G107), backup codegen (G105), to threaded forums (G113). Thirteen patterns that appear in every data-driven application.

Remaining: **Graphics (G114–G130) — 17 projects** to close the Rosetta Stone milestone.
