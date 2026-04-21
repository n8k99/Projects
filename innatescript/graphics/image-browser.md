# G124 — Image Browser

> The Rosetta Stone's eleventh **Graphics project**. Models the **visual file browser** every OS ships (macOS Finder gallery view / Windows Explorer / nautilus): a 2D grid of thumbnails over a directory tree, with a cursor that moves Left/Right/Up/Down, a filter+sort query, an LRU thumbnail cache (so scrolling a thousand-image folder doesn't re-decode every thumbnail on every tick), and a breadcrumb navigation stack. The distinctive move: **LRU eviction replaces FIFO eviction** — G123's ring buffer dropped the oldest by insertion order; G124's cache drops the one untouched longest, so a thumbnail you recently viewed stays warm even if it was loaded hours ago.

```yaml
id: G124
title: Image Browser
category: graphics
requires: [G092-bulk-renamer, G116-grayscale-converter, G123-screen-capture]
provides: [grid-2d-cursor, lru-cache, composable-query, breadcrumb-navigation]
```

## Insight: 2D Grid Cursor Over a 1D Index

The cursor is a single `selected: u32` indexing into a flat list. Grid semantics are derived: `row = selected / cols`, `col = selected % cols`. Moves translate to arithmetic:

- **Left** → `selected - 1` (clamped at 0).
- **Right** → `selected + 1` (clamped at `total - 1`).
- **Up** → `(row - 1) * cols + col` (noop at row 0).
- **Down** → `(row + 1) * cols + col` **if that index exists**; else if next row is partial and has any items, clamp to last entry.

The partial-last-row clamp matters: a 3-column grid with 7 items has rows of size 3, 3, 1. Pressing Down from `col=2, row=1` would compute `idx=8` which is out of bounds; the clamp sends the cursor to `idx=6` (the single last-row entry).

First Rosetta Stone project with **2D grid navigation derived from a 1D index**. G113's forum had tree navigation (parent/child); G124 has grid navigation (row/col) — both derived from flat storage.

## Insight: LRU Cache Replaces FIFO Ring

G123's history was a fixed-size FIFO ring — evict by **insertion order**. G124's thumbnail cache is LRU — evict by **access order**. The difference:

```
FIFO: insert A, B, C, D (cap=3) → keep {B, C, D}, drop A.
LRU:  insert A, B, C. access A. insert D → keep {A, C, D}, drop B.
```

Why LRU matters for a browser: if the user scrolls past A, B, C then back to A, A should be warm. With FIFO, A would evict first. With LRU, accessing A refreshes its position, so B (the actual least-used) evicts instead.

`get(k)` moves `k` to the "most recently used" end. `put(k, v)` on a full cache evicts the least-recent key and returns it (so callers can log or animate the eviction).

First Rosetta Stone project with **recency-based (not insertion-based) eviction**. G123 proved it was an FSM; G124 proves there are two evict-orderings and the right choice depends on access pattern.

## Insight: Query Is a Data Value, Not a Method Chain

```
Query { filters: [NameContains("jpg"), TagHas("nature")],
        sort_by: Size, order: Desc }
```

- **Filters** — a list of predicates AND-ed together. Empty list = match all.
- **Sort** — one `SortBy` + `Order`. No secondary sort (multiple sorts require multiple passes or a composite sort key).
- **Immutable** — applying a query to a list returns a filtered + sorted result; the original is untouched.

Compositional: building a UI filter panel translates directly to `Vec<FilterPredicate>`. Saving a query as a "smart folder" is literally serializing the struct.

First Rosetta Stone project where **filter composition is AND-ed predicates, not a fluent builder**. G119's transform pipeline was an ordered list of *operations*; G124's filters are an unordered set of *predicates* (AND is commutative).

## Insight: Breadcrumb Is a Stack, Path Is a Fold

`breadcrumbs: Vec<String>`. `navigate_into(name)` pushes; `navigate_up()` pops; `current_path()` is `"/" + breadcrumbs.join("/")`. The path isn't stored — it's derived from the stack on demand.

Benefits:
- **Undo-navigate** is pop.
- **Truncate to ancestor** is `breadcrumbs.truncate(n)`.
- **Full path** is a pure fold: `breadcrumbs.iter().fold("/", |acc, s| acc + "/" + s)`.
- **Root-is-empty** is the natural semantic: `breadcrumbs.is_empty() ⟺ at_root`.

First Rosetta Stone project where **hierarchical navigation is a stack with derived paths**. G113's forum had parent pointers; G124 has a traversal stack (the *path taken*, not the *tree structure*).

## Insight: Thumbnail Load Is Lazy with Cache Fallthrough

`get_or_load_thumbnail(id, producer)` — if cached, return hit. Else call `producer()`, cache the result, return (result, miss=true). The miss flag lets tests verify the producer is called exactly once per cache miss.

This is the *memoize* pattern applied to I/O-bound production. Same shape works for any expensive lookup (disk reads, network fetches, heavy computations).

First Rosetta Stone project with **lazy memoization tagged with hit/miss for observability**. G086's frecency ranker was pure; G124's thumbnail loader has side effects but hides them behind the cache.

## Insight: Filter Then Sort, Not Sort Then Filter

`apply_query` filters first, then sorts. Filtering a smaller list is cheaper than sorting then discarding; sorting N then filtering to k is O(N log N), but filtering to k then sorting is O(N + k log k). For big image directories (N = 10_000), sort-after matters.

First Rosetta Stone project where **operation order within a query matters for performance** in a way worth preserving in the implementation.

## Choreographic Case: Vault Image Explorer

```innate
(@vault-image-explorer){
  @br <- @ibrowser/new{grid-cols: 4, thumbnail-cache-capacity: 100}
  @ibrowser/set-entries{browser: @br, entries: @vault/images{}}

  @on-key-left  (@ibrowser/move{browser: @br, direction: "left"})
  @on-key-right (@ibrowser/move{browser: @br, direction: "right"})
  @on-key-up    (@ibrowser/move{browser: @br, direction: "up"})
  @on-key-down  (@ibrowser/move{browser: @br, direction: "down"})

  @on-tag-filter-changed (@tag){
    @ibrowser/set-query{browser: @br,
                         query: {filters: [{kind: "tag-has", s: @tag}],
                                  sort-by: "created", order: "desc"}}
  }

  @on-render-tick{
    (for @entry in @br.visible-entries{
      @thumb <- @ibrowser/get-or-load-thumbnail{
        browser: @br, id: @entry.id,
        producer: (fn () @image/thumbnail{entry: @entry, size: 128})
      }
      @ui/draw-thumbnail{row: @entry.row, col: @entry.col, data: @thumb}
    })
    @ui/highlight-selected{row: @br.grid.row, col: @br.grid.col}
  }

  @on-enter{
    @selected <- @ibrowser/selected-entry{browser: @br}
    @ui/open-viewer{image: @selected}
  }
}
```

## Structures

```innate
(defstruct image-entry
  id          : Int
  name        : String
  path        : String
  width       : Int
  height      : Int
  size-bytes  : Int
  tags        : [String]
  created-ms  : Int)

(defenum filter-kind
  NAME_CONTAINS | TAG_HAS | SIZE_UNDER | SIZE_OVER
  | WIDTH_AT_LEAST | CREATED_AFTER)

(defstruct filter-predicate
  kind : FilterKind
  s    : String
  n    : Int)

(defenum sort-by NAME | SIZE | CREATED | WIDTH)
(defenum order ASC | DESC)

(defstruct query
  filters  : [FilterPredicate]
  sort-by  : SortBy
  order    : Order)

(defstruct grid
  cols     : Int
  total    : Int
  selected : Int)

(defenum grid-move LEFT | RIGHT | UP | DOWN | HOME | END)

(defstruct lru-cache
  capacity : Int
  entries  : [(Int, Bytes)])   ;; head = LRU, tail = MRU

(defstruct browser
  grid-cols         : Int
  breadcrumbs       : [String]
  all-entries       : [ImageEntry]
  query             : Query
  grid              : Grid
  thumbnail-cache   : LruCache)
```

## Resolver Natives

```innate
@ibrowser/new{grid-cols, thumbnail-cache-capacity}    -> Browser
@ibrowser/set-entries{browser, entries}               -> Unit
@ibrowser/set-query{browser, query}                   -> Unit
@ibrowser/filtered-entries{browser}                   -> [ImageEntry]
@ibrowser/selected-entry{browser}                     -> ImageEntry?
@ibrowser/move{browser, direction}                    -> Unit
@ibrowser/navigate-into{browser, subdir}              -> Unit
@ibrowser/navigate-up{browser}                        -> Bool
@ibrowser/current-path{browser}                       -> String
@ibrowser/get-or-load-thumbnail{browser, id, producer} -> (Bytes, Bool)
@apply-query{entries, query}                          -> [ImageEntry]
```

## Demo

```innate
(@demo){
  @br <- @ibrowser/new{grid-cols: 3, thumbnail-cache-capacity: 3}
  @ibrowser/set-entries{browser: @br, entries: [
    {id: 1, name: "alpha.jpg", size-bytes: 200000, tags: ["nature"], ...},
    {id: 2, name: "beta.jpg", size-bytes: 50000, tags: ["abstract"], ...},
    {id: 3, name: "gamma.png", size-bytes: 150000, tags: ["nature"], ...},
    {id: 4, name: "delta.jpg", size-bytes: 500000, tags: ["nature"], ...},
    {id: 5, name: "epsilon.png", size-bytes: 100000, tags: ["abstract"], ...}
  ]}
  @br.grid.total   ;; -> 5

  @ibrowser/set-query{browser: @br, query: {
    filters: [{kind: "name-contains", s: ".jpg"},
               {kind: "tag-has", s: "nature"}],
    sort-by: "size", order: "asc"
  }}
  ;; filtered: 2 entries (alpha.jpg 200K, delta.jpg 500K); sorted asc by size
  @ibrowser/filtered-entries{browser: @br}
    ;; -> [alpha.jpg, delta.jpg]

  @ibrowser/selected-entry{browser: @br}   ;; -> alpha.jpg
  @ibrowser/move{browser: @br, direction: "right"}
  @ibrowser/selected-entry{browser: @br}   ;; -> delta.jpg
  @ibrowser/move{browser: @br, direction: "right"}
  @ibrowser/selected-entry{browser: @br}   ;; -> delta.jpg (at end, no-op)

  @ibrowser/navigate-into{browser: @br, subdir: "photos"}
  @ibrowser/navigate-into{browser: @br, subdir: "2026"}
  @ibrowser/current-path{browser: @br}     ;; -> "/photos/2026"
  @ibrowser/navigate-up{browser: @br}
  @ibrowser/current-path{browser: @br}     ;; -> "/photos"

  ;; LRU cache with capacity 2
  @lru <- @lru-cache/new{capacity: 2}
  @lru-cache/put{cache: @lru, key: 1, val: "a"}
  @lru-cache/put{cache: @lru, key: 2, val: "b"}
  @lru-cache/get{cache: @lru, key: 1}   ;; refreshes 1; 2 is now LRU
  @lru-cache/put{cache: @lru, key: 3, val: "c"}
  ;; -> evicts 2 (LRU), not 1
}
```

## Where

Grid cursor MUST be derived from 1D index — storing row+col separately makes bounds checks non-local; `selected / cols`, `selected % cols` keeps the invariant trivial. Down-arrow MUST clamp on partial last row — pressing Down should land *somewhere*, not disappear off-grid. LRU MUST refresh on `get` — forgetting to move on access turns LRU into FIFO. Filter composition MUST be AND across the list — OR requires a separate composite operator (or serializing as a single `OneOf` predicate). Filter-then-sort MUST be the order — sort-then-filter is O(N log N + N); filter-then-sort is O(N + k log k) and the difference matters at N=10_000. Breadcrumb MUST be a stack with derived paths — storing the full path string and splitting on `/` fails on filenames with embedded slashes. Thumbnail cache hit/miss MUST be surfaced — tests need it; UI may animate loading differently; telemetry may track cache hit rate. `put` on full cache MUST return the evicted key — callers need to decide whether to persist or discard the evicted entry's associated resources.
