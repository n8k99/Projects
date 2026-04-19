# G065 — Progress Bar for Downloads

> The Rosetta Stone's first **concurrent producer-consumer** — one thread writes, another reads, and a synchronisation primitive ensures no field is seen half-updated. The first project where "who computes" and "who displays" are deliberately distinct actors with a contract between them.

```yaml
id: G065
title: Progress Bar for Downloads
category: threading
requires: [G062-vending-machine]
provides: [shared-atomic-state, producer-consumer, cooperative-cancellation, fsm-across-threads, coalesced-render]
```

## Insight: The Producer and the Consumer Are Distinct Actors

Every prior Classes project had a single actor that both *recorded* state changes and *surfaced* them to the caller. The flower shop prints its own receipt; the vending machine announces its own state. G065 is the first project where the recorder (the worker, incrementing a byte counter) and the observer (the display, reading the counter) are **different threads** with a contract between them.

This separation is what makes concurrency necessary. If the same actor did both, there would be nothing to synchronise — a single thread's sequence of operations is already ordered. Two threads mean two timelines, and merging them requires an explicit primitive: an atomic counter, a lock, a channel, a ref. Every concurrency pattern in the noosphere — agent A streaming to agent B, the IPC daemon handling many clients, the choreography runner polling for progress on spawned tasks — has the same shape as G065.

## Insight: Atomics Are the Minimum Synchronisation

A counter that is *only* incremented by one thread and *only* read (not modified) by another needs no lock — an atomic add (producer) and atomic load (consumer) is sufficient. The Rust/Go implementations use this: `AtomicU64::fetch_add` and `AtomicU64::load`. The Python and Common Lisp implementations take a mutex because their idiomatic concurrency primitive is a lock, but even there the critical section contains only a single addition — the lock is effectively operating as an atomic.

**Choose the lightest primitive that works.** A mutex costs more than an atomic. A channel costs more than a mutex. An RwLock is overkill for single-writer state. Every concurrency primitive has a cost; the design question is which one is cheap enough for the throughput you need. G065 presents the answer-the-question-of-cost reasoning on the smallest possible case, and the answer is "atomic counter" — the minimum.

## Insight: Completion Is a State, Not a Flag

A boolean "done" flag is tempting but insufficient. The tracker needs to distinguish at least four states: running, completed successfully, cancelled by user, failed with an error. This is a small **FSM observed from another thread** — the same primitive as G062's vending machine, but now with memory-ordering implications.

The terminal states are sticky: once Completed, a later Cancel is a no-op. The implementation uses `compare_exchange` / `CompareAndSwap` to enforce "only transition from Running; if the status is already terminal, the attempt is silently ignored." This is the first Rosetta Stone pattern where **state transitions are conditional on current state**, at the CPU instruction level rather than via a higher-level lock.

Memory ordering matters. The worker writes `bytes_done`, then writes `status = Completed`. The observer reads `status`, sees Completed, and then reads `bytes_done`. If these reads and writes aren't ordered, the observer could see `status == Completed && bytes_done < total` — a corrupt view. Rust's `Ordering::SeqCst` on the status transitions enforces the ordering; Go's atomics default to sequential consistency. This is the concurrency specialist's permanent vigilance: every multi-field read is a place where interleavings can betray you unless you order the operations explicitly.

## Insight: Cancellation Is Cooperative, Not Preemptive

The controller does not "kill" the worker thread. It sets a flag (`status = Cancelled`), and the worker checks that flag between chunks. If the worker never checks, it never notices, and cancellation is effectively ignored.

This is a design choice with profound implications. **Preemptive cancellation is unsafe in almost every language** — killing a thread mid-operation can leave locks held, files open, reference counts wrong, buffers half-written. Every production concurrency library (Go contexts, Rust `tokio::select!`, Java interruption, Python's `threading.Event`) uses cooperative cancellation because the alternatives are nightmares.

Cooperative cancellation imposes a contract on the worker: **check frequently**. The worker is not a black box that runs to completion; it is a sequence of small operations with cancellation checkpoints between them. G065's worker loop is one chunk per iteration; between chunks, it checks. This is the first Rosetta Stone project where the granularity of the worker's loop is *itself a design parameter* — coarser grain means cheaper work but slower response to cancellation; finer grain means faster response but more overhead.

The noosphere's choreography runner uses the same pattern: between choreography steps, the runner checks whether the parent choreography has been cancelled. Steps are checkpoints. An agent that does a 10-minute computation with no checkpoints is uncancellable.

## Insight: Output Throughput Decouples from Input Throughput

A 1 GB download in 1 KB chunks is a million increments. The human eye distinguishes perhaps 30 frames per second; anything more is wasted work. The display thread **polls on its own schedule**, not on the producer's schedule — every 50 ms in G065's Python demo, regardless of whether one chunk or ten thousand chunks arrived in between.

This is the first Rosetta Stone pattern where **the consumer does not react to every producer event**. Every prior project had a 1:1 event-to-response mapping (one purchase → one dispensed product, one recipe scale → one scaled output). G065 introduces deliberate downsampling: the producer writes fast, the consumer reads slow, and the synchronisation primitive guarantees that each read sees a consistent snapshot.

Coalescing is the general pattern. Log aggregation, metrics collection, the vault's editor-update debounce, the quickshell bar's 10s bluetooth poll — all instances of "the consumer chooses its own cadence, ignoring the producer's rate." G065 is the minimal teaching example.

## Insight: Time Becomes a First-Class Input

Every prior Rosetta Stone project computed outputs as pure functions of inputs. G065 is the first project where **wall-clock time is part of the computed output**: `bytes_per_second = bytes_done / (now - start_time)`, `eta_seconds = (remaining_bytes) / bytes_per_second`. These values depend on a clock, not just on the data.

This is the edge where pure-functional code ends. Time-dependent output means repeated calls with the same inputs return different values — a violation of referential transparency. Test setups get harder: to assert "ETA should be 10 seconds," you must either inject a fake clock or accept non-determinism. G065 lets `elapsed_seconds` be real time because the tests that matter (the ETA test) only assert `eta > 0`, not a specific value — the first instance of **accepting imprecision in tests because the system under test is inherently non-deterministic**.

Noosphere analogues: the daily note's "schedule" block reads the current clock; the agent-dispatch routing includes response-time SLAs computed against elapsed wall time; cache expiration depends on `now - cached_at`. Any real system that does not treat time as an input will fail as soon as it deals with durations.

## Choreographic Case: Long-Running Agent Task with Live Progress

```innate
(@long-fetch){
  @url <- @params/url
  @dest <- @params/dest
  @total <- @http/content-length{url: @url}
  @tracker <- @progress/new{total: @total}

  @spawn (@worker){
    @bytes-remaining <- @total
    @for chunk in @http/stream-chunks{url: @url} {
      where { not-cancelled: not @tracker/is-cancelled }
      @file/append{path: @dest, bytes: chunk}
      @progress/advance{tracker: @tracker, delta: chunk.size}
    }
    @progress/complete{tracker: @tracker}
  }

  @spawn (@display){
    @every 100.ms until @tracker.status != "running" {
      @render{bar: @progress/snapshot{tracker: @tracker}}
    }
  }

  @on-user-cancel {
    @progress/cancel{tracker: @tracker}    ;; worker sees it on next chunk
  }
}
```

The choreography reads as prose because the primitive exists: one worker, one display, one cancellation flag, all communicating through a shared tracker. This is the core composition every long-running agent task in the noosphere will use.

## Structures

```innate
(defstruct progress-tracker
  bytes-done   : Int              ;; atomic
  total-bytes  : Int
  status       : "running" | "completed" | "cancelled" | "failed"
  start-time   : Timestamp)

(defstruct progress-snapshot
  bytes-done       : Int
  total-bytes      : Int
  status           : Status
  elapsed-seconds  : Float)
```

## Resolver Natives

```innate
@progress/new{total}                 -> Tracker
@progress/advance{tracker, delta}    -> Unit       ;; atomic add
@progress/cancel{tracker}            -> Unit       ;; transition only from running
@progress/complete{tracker}          -> Unit
@progress/fail{tracker}              -> Unit
@progress/is-cancelled{tracker}      -> Bool
@progress/snapshot{tracker}          -> Snapshot

@snapshot/percent{s}                 -> Float
@snapshot/bytes-per-second{s}        -> Float
@snapshot/eta-seconds{s}             -> Float?
@snapshot/render-bar{s, width}       -> String
```

## Demo

```innate
(@demo){
  @tr <- @progress/new{total: 1_000_000}
  @spawn {
    @for i in 0..100 {
      break-if @progress/is-cancelled{tracker: @tr}
      @progress/advance{tracker: @tr, delta: 10_000}
      @sleep 5.ms
    }
    @progress/complete{tracker: @tr}
  }
  @every 50.ms while @progress/snapshot{@tr}.status == "running" {
    @print @snapshot/render-bar{s: @progress/snapshot{@tr}, width: 40}
  }
}
```

## Where

Counter updates MUST use an atomic / compare-exchange primitive appropriate to the language — a plain `+=` on a shared field is a data race and a correctness bug, not merely a performance one. Status transitions MUST be conditional on the current state (only `Running → terminal`); terminal states MUST be sticky. Cancellation MUST be cooperative — the worker MUST check `is_cancelled()` at least once per chunk, and chunk size IS a design parameter that trades throughput against cancellation latency. Snapshot reads MUST capture the entire state in one observation; interleaved per-field reads that span an update interval are a visible bug, even if no test catches them. Rendering MUST NOT block the worker — if the display thread falls behind, the worker keeps working and the display will catch up on its next poll.
