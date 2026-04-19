# G068 — Bulk Thumbnail Creator

> The Rosetta Stone's first project where **the result is a shared index built by concurrent workers** — and where content-addressing makes parallel-safe deduplication trivial. Closes the Threading category by composing G066's fan-out worker pool with G056's content-addressed identity.

```yaml
id: G068
title: Bulk Thumbnail Creator
category: threading
requires: [G056-image-gallery, G066-download-manager]
provides: [shared-index-output, content-addressed-dedup, reservation-pattern, cpu-bound-parallelism, aggregation-as-result]
```

## Insight: The Result Is an Aggregation, Not a List of Independent Outcomes

G066 produced a list of `DownloadResult`s — each result was independent; putting them in a list was bookkeeping. G068 produces a **shared index** that the workers cooperatively build: `content_hash → thumbnail`, `source_id → content_hash`. The result is not a collection of isolated outputs; it is a single aggregated artifact shaped by every worker's contribution.

This distinction is load-bearing. Many real parallel pipelines produce an aggregate rather than a list: building a search index, computing a dependency graph, assembling a code-map, synthesising embeddings into a vector store. The workers don't compute independent answers — they each contribute a piece of a final whole. G068 makes the pattern concrete at minimal scale.

## Insight: Content-Addressing Makes Dedup Trivial and Parallel-Safe

If two source files have identical bytes, they should produce one thumbnail, not two. The standard naive approach (check-if-exists, generate, store) has a race condition under concurrency — two workers can both see "not present," both generate, and one overwrites the other. Wasted work at best, torn writes at worst.

**Content-addressing dissolves the race.** The primary key of the thumbnail index is the content hash itself. Two workers with identical content compute the same hash, which collides in the index under a single lock — and *only one worker succeeds in claiming the slot*. The other worker observes the collision and records the mapping without regenerating. The expensive thumbnailer runs exactly once per unique content, regardless of how many sources present that content or how many workers race to process them.

This is the pattern behind every content-addressed system in production: Nix / Guix package caches (hash of build inputs → build output), Docker image layers (hash of layer content → stored layer), Git object storage (hash of object → object), IPFS (hash → data), the vault's future blob-backed image store. **G056 introduced content-addressed identity; G068 shows why it's the right choice under concurrency**.

## Insight: The Reservation Pattern Decouples Lock Hold Time from Work Time

The naive check-and-insert must hold the lock while generating the thumbnail — otherwise two workers race and both generate. But holding the lock during an expensive computation *serialises all workers*, destroying the point of the worker pool.

The **reservation pattern** solves this:

1. Acquire the index lock.
2. If the hash is already present (with any value, even a placeholder), release and record dedup.
3. Otherwise, insert a *placeholder* value (empty string, None, whatever signals "in progress") and release.
4. Run the expensive thumbnailer **without holding any lock**.
5. Reacquire the lock briefly to install the real value.

Concurrent workers that arrive between steps 3 and 5 see the placeholder as "present" and treat it as dedup — semantically correct, because someone else *is* generating this thumbnail. The lock is held only for two tiny windows; the expensive work runs in parallel.

This is the pattern behind lazy memoisation under concurrency (Go's `sync.Once`, Rust's `OnceLock`, Python's `functools.cache` in multi-thread programs), content-addressed cache lookups, and any "compute once, share many" scenario. G068 presents the primitive explicitly; production systems hide it inside a library.

## Insight: CPU-Bound Parallelism Has Different Ergonomics from I/O-Bound

G066 was I/O-bound (waiting on network). G068 is CPU-bound (computing pixels). The difference matters:

- **Optimal worker count.** For I/O-bound work, many workers (dozens, sometimes hundreds) win because most of each worker's time is spent waiting, not running. For CPU-bound work, workers beyond the CPU count produce no speedup and may actually slow things down via context-switching. G068's best worker count is usually `N = num_CPUs`.
- **Lock contention is a bigger deal.** I/O-bound workers spend most of their time outside the lock. CPU-bound workers are constantly ready to run; anything that serialises them (oversized critical sections, contentious shared state) directly caps throughput.
- **Python's GIL matters.** Python threads do not provide CPU-bound parallel speedup for pure-Python code — the GIL serialises bytecode execution on a single core. The dedup pattern still works correctly; the speedup does not materialise unless the thumbnailer releases the GIL (most image libraries do). Real Python CPU-bound parallelism requires `multiprocessing`. G068's Python implementation documents this and gets correctness without making performance claims.

## Insight: Aggregation State Requires Two Orthogonal Locks

G066 had one shared structure (the queue, effectively; the results list was append-only with trivial contention). G068 has two:

- **Queue lock** (hot: every pop acquires it).
- **Index lock** (hot: every claim and install acquires it).

These are orthogonal — operations on the queue don't affect the index and vice versa — so two separate mutexes are correct and preferred over one giant mutex. Contention on each is independent; a worker waiting on the queue lock doesn't block another worker installing into the index.

This is the first Rosetta Stone project where **partitioning locks** is visibly the right choice. In production, the index lock itself is often *further* partitioned (sharded by hash prefix, or replaced with a lock-free concurrent map) to handle thousands of concurrent workers without contention collapse. G068 uses one global index lock because the demo scale doesn't need more — but the seam is there.

## Insight: Closing Threading — All Four Primitives Are Now Present

Threading's four projects introduced, in order:
- G065: **Shared counter + FSM** (one producer, one consumer, atomic primitives).
- G066: **Shared queue + fan-out** (one producer, N consumers, pull scheduling).
- G067: **Shared broadcast** (N producers, N consumers, total ordering via monotonic counter).
- G068: **Shared index** (N producers contributing to an aggregated output).

Together these cover the four fundamental concurrency patterns. Any real multi-threaded program is assembled from these primitives: a thread pool (G066) with per-worker progress tracking (G065), broadcasting completions to subscribers (G067), and contributing results to a shared index (G068). The noosphere's agent-dispatch layer will use all four.

The category is short because the **primitives are few**. Every other concurrency pattern in existence is a composition of these with timeouts, back-pressure, load shedding, or ordering constraints added on top.

## Choreographic Case: Parallel Vault Thumbnail Generation

```innate
(@generate-vault-thumbnails){
  @images <- @vault/images/all
  @creator <- @thumbnails/new{worker_count: 8}

  @for img in @images {
    @creator/enqueue{req: {source_id: @img.path, content: @fs/read{@img.path}}}
  }

  @index <- @creator/run-with{
    hasher: @content/sha256,
    thumbnailer: @image/resize-to{dims: [200, 200]}
  }

  @print "${@index.generated} thumbnails generated, ${@index.deduplicated} dedup'd"
  @for (source-id, hash) in @index.mappings {
    @vault/write-mapping{source: @source-id, thumbnail: @index.thumbnails[@hash]}
  }
}
```

The choreography reads prose-like because the domain matches: enqueue every image, eight workers hash and generate in parallel, duplicates are detected and mapped without regeneration, the final index has one entry per unique content and a source→hash mapping preserving the original source list.

## Structures

```innate
(defstruct thumbnail-request
  source-id : String
  content   : Bytes)

(defstruct thumbnail-index-snapshot
  thumbnails    : {Int -> String}       ;; content-hash -> thumbnail-token
  mappings      : {String -> Int}       ;; source-id -> content-hash
  generated     : Int                   ;; thumbnails actually computed
  deduplicated  : Int)                  ;; sources that hit the cache

(defstruct bulk-thumbnail-creator
  worker-count  : Int
  queue         : [ThumbnailRequest]
  index         : ThumbnailIndexState)
```

## Resolver Natives

```innate
@thumbnails/new{worker_count}                          -> Creator
@creator/enqueue{creator, req}                         -> Unit
@creator/run-with{creator, hasher, thumbnailer}        -> Snapshot    ;; blocks
```

## Demo

```innate
(@demo){
  @c <- @thumbnails/new{worker_count: 4}
  @for kind in 0..10 {
    @for copy in 0..5 {
      @c/enqueue{req: {source_id: "src_${kind}_${copy}",
                       content: bytes("content kind ${kind}")}}
    }
  }
  @snap <- @c/run-with{hasher: @fake/hash, thumbnailer: @fake/resize}
  ;; -> unique: 10, mappings: 50, generated: 10, deduplicated: 40
}
```

## Where

The thumbnailer MUST run exactly once per unique content hash, regardless of how many sources present that content or how many workers race on it — the reservation-pattern claim-and-release-before-work is what guarantees this, not hope. The claim MUST be atomic (check + insert-placeholder under one lock acquisition); a check-then-insert in two lock acquisitions has a visible race and is a bug. The placeholder MUST be overwritten when the thumbnailer returns — leaving the empty string in place produces a silently-corrupted index. The source_id → hash mapping MUST be recorded for every request including dedup'd ones — the mapping is how downstream queries find the thumbnail for a given source. CPU-bound thumbnailers MUST release any GIL / interpreter-lock when possible; otherwise worker_count > 1 produces no speedup and is misleading to callers. The two locks (queue and index) MUST be separate — one giant lock serialises workers that should run in parallel.
