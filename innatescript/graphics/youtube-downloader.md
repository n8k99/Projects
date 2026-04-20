# G121 — YouTube Downloader

> The Rosetta Stone's eighth **Graphics project**. Models the **concurrency-limited download manager** every multi-stream downloader ships (youtube-dl, yt-dlp, Aria2): a set of pending downloads, a max-in-flight limit, shared bandwidth, per-download resumable progress, and retry with exponential backoff. The distinctive move: **no threads**. The whole manager is driven by `tick(ms)`, bandwidth splits equally across active downloads each tick, and retries are data (a ready-at-ms deadline compared against elapsed). Resumable state (bytes-done) **persists across failures** — a download that fails midway keeps its progress and resumes from that offset.

```yaml
id: G121
title: YouTube Downloader
category: graphics
requires: [G098-file-copier, G117-stream-player, G120-cd-burner]
provides: [concurrency-limit, resumable-state, exponential-backoff, format-selection]
```

## Insight: Concurrency Is a Slot Count, Not Threads

`max_concurrent: int` is the only parallelism primitive. The manager holds a `pending_queue` of download IDs; `tick(ms)` pulls from the queue into active slots up to the limit, drains bytes proportionally from each active slot, and completes downloads when `bytes_done >= bytes_total`. A free slot is filled immediately when one opens (via completion or failure).

No threads, no async, no mutex. A tick is atomic: all slots move by the same simulated millisecond. Tests are deterministic — same inputs, same output.

First Rosetta Stone project where **concurrency is simulated via tick-driven slot accounting**. G117's stream player had one active stream; G121 has N streams sharing one bandwidth budget.

## Insight: Bandwidth Splits Equally Across Active

`per_stream_bps = bandwidth_bps / len(active)`. Each tick, every active download advances by `(per_stream_bps * ms) / 8000` bytes. When the set of active downloads changes (one completes, one fails, one starts), the per-stream rate changes for the next tick.

This models the TCP fair-sharing assumption — when N flows share a link, each gets approximately `bandwidth / N`. It's not perfectly accurate (real TCP has flow-control variance), but for a downloader's UX it's exactly right.

First Rosetta Stone project where **a shared resource is proportionally divided among active consumers each tick**. G107's budget was whole-value allocation; G121 has continuous proportional division.

## Insight: Resumable State Persists Across Failures

`bytes_done` lives on the Download, not the attempt. When a download fails, its state transitions to `Retrying { ready_at_ms }` but `bytes_done` stays put. When the retry promotes back to `InFlight`, progress resumes from the last byte received. This is the in-memory model of HTTP Range requests (`Range: bytes=X-`).

Tests verify this explicitly — a failure at 500 KB into a 10 MB download retries from 500 KB, not from 0.

First Rosetta Stone project where **partial progress is first-class state, persistent across control-flow failures**. G097's flashcard spacing had per-card state; G121 has per-transfer byte offsets that survive retry transitions.

## Insight: Backoff Is a Pure Function Over Attempt

```
backoff_for_attempt(n, policy) = policy.initial_ms * policy.multiplier^(n-1)
```

Exponential backoff with integer multiplier (typically 2). Same attempt count, same policy → same backoff. No timer; `retry_scheduled` records `ready_at_ms = elapsed + backoff`, and `tick` promotes `Retrying` to `Pending` once `elapsed >= ready_at_ms`. The ready check is a pure comparison.

First Rosetta Stone project where **retry is a scheduled state transition driven by tick comparison**, not a timer or `sleep`. G094's journaled logs had time but no retries; G121 introduces time-conditional transitions.

## Insight: Format Selection Is a Policy, Not a Pick

`pick_format(formats, policy) -> Format?` where `policy ∈ {MaxQuality, Lowest, BestUnder(cap)}`. The user picks *the policy*; the system picks *the format*. Swapping policies re-runs the pick without the user knowing which specific format was chosen last time.

`BestUnder(cap)` can return `None` if no format fits — callers must handle the empty case explicitly. `MaxQuality` on an empty list also returns `None`.

Same shape as G117's ABR bitrate pick (pure function over measured inputs), but at a different level: ABR picks per segment; format picks once per download.

First Rosetta Stone project where **user intent is captured as a policy value**, not a concrete selection. G086's frecency was a ranking function; G121's format policy is a ranking *over a constrained subset*.

## Insight: FSM Is Five States with One Self-Loop

`Pending → InFlight → Retrying → Pending (loop) → Succeeded | Failed`. Failure goes to Retrying (if attempts left) or Failed (terminal). Retrying loops back to Pending once the deadline passes. The FSM is larger than G118's (4 states) because retries introduce a bounded self-loop.

`attempts` is tracked; `Failed` is entered when `attempts >= max_attempts`. Tests assert both the happy-path traversal and the abandoned-after-N-retries terminal state.

First Rosetta Stone project with **a self-looping FSM bounded by an attempt counter**. G107's budget had bounded overruns; G121 has bounded retry cycles.

## Choreographic Case: Vault Offline Archive

```innate
(@vault-offline-archive){
  @videos <- @vault/saved-videos{tag: "to-archive"}
  @mgr <- @ytd/new-manager{max-concurrent: 3, bandwidth-bps: 50_000_000,
                             retry-policy: {max-attempts: 3,
                                            initial-ms: 1000,
                                            multiplier: 2}}

  (for @v in @videos{
    @format <- @ytd/pick-format{
      formats: @v.formats,
      policy: {kind: "best-under", cap-bytes: 500_000_000}
    }
    (if @format @ytd/enqueue{manager: @mgr, video-id: @v.id, format: @format}
                @ui/log{level: "warn", msg: "no format under 500MB for ${@v.title}"})
  })

  @on-network-tick (@elapsed-ms){
    @ytd/tick{manager: @mgr, ms: @elapsed-ms}
    @ui/render-progress{downloads: @mgr.downloads,
                         active: @mgr.active-count,
                         pending: @mgr.pending-queue.length}
  }

  @on-network-error (@download-id){
    @ytd/report-failure{manager: @mgr, id: @download-id}
  }
}
```

The vault's archiver shell is a thin wrapper: enqueue all videos at a fixed quality cap, tick to drive progress, report failures from the network layer. The manager handles backoff, resumption, and slot filling.

## Structures

```innate
(defstruct video-format
  id            : String
  resolution    : Int
  codec         : String
  container     : String
  size-bytes    : Int
  bitrate-bps   : Int)

(defenum format-policy-kind MAX_QUALITY | LOWEST | BEST_UNDER)

(defstruct format-policy
  kind       : FormatPolicyKind
  cap-bytes  : Int)

(defstruct retry-policy
  max-attempts : Int
  initial-ms   : Int
  multiplier   : Int)

(defenum download-state-kind
  PENDING | IN_FLIGHT | RETRYING | SUCCEEDED | FAILED)

(defstruct download-state
  kind          : DownloadStateKind
  ready-at-ms   : Int)

(defstruct download
  id           : Int
  video-id     : String
  format       : VideoFormat
  bytes-done   : Int
  bytes-total  : Int
  attempts     : Int
  state        : DownloadState)

(defstruct manager
  max-concurrent   : Int
  bandwidth-bps    : Int
  retry-policy     : RetryPolicy
  downloads        : [Download]
  pending-queue    : [Int]
  elapsed-ms       : Int
  events           : [ManagerEvent])
```

## Resolver Natives

```innate
@ytd/pick-format{formats, policy}                      -> VideoFormat?
@ytd/retry-policy-backoff{policy, attempt}             -> Int
@ytd/new-manager{max-concurrent, bandwidth-bps, retry-policy} -> Manager
@ytd/enqueue{manager, video-id, format}                -> Int (download id)
@ytd/tick{manager, ms}                                  -> Unit
@ytd/report-failure{manager, id}                        -> Unit
@ytd/download{manager, id}                              -> Download?
@ytd/active-count{manager}                              -> Int
```

## Demo

```innate
(@demo){
  @formats <- [
    {id: "a", resolution: 480,  size-bytes: 50_000_000,  bitrate-bps: 500_000},
    {id: "b", resolution: 720,  size-bytes: 100_000_000, bitrate-bps: 1_000_000},
    {id: "c", resolution: 1080, size-bytes: 200_000_000, bitrate-bps: 2_000_000}
  ]

  @ytd/pick-format{formats: @formats, policy: {kind: "max-quality"}}   ;; -> c
  @ytd/pick-format{formats: @formats, policy: {kind: "lowest"}}        ;; -> a
  @ytd/pick-format{formats: @formats,
                    policy: {kind: "best-under", cap-bytes: 120_000_000}}
  ;; -> b  (720p fits under 120MB; 1080p at 200MB doesn't)

  @mgr <- @ytd/new-manager{max-concurrent: 2, bandwidth-bps: 8_000_000,
                             retry-policy: {max-attempts: 3,
                                            initial-ms: 1000,
                                            multiplier: 2}}
  @ytd/enqueue{manager: @mgr, video-id: "v1", format: @formats[1]}
  @ytd/enqueue{manager: @mgr, video-id: "v2", format: @formats[0]}
  @ytd/enqueue{manager: @mgr, video-id: "v3", format: @formats[2]}

  @ytd/tick{manager: @mgr, ms: 1000}
  @mgr.active-count   ;; -> 2  (slots 1 and 2 active; 3 still pending)
  ;; with 8 Mbps / 2 streams = 4 Mbps = 500 KB/s each, each advances 500,000 bytes

  @ytd/report-failure{manager: @mgr, id: 1}
  @mgr.download[1].state   ;; -> {kind: RETRYING, ready-at-ms: 2000}
  @mgr.download[1].attempts ;; -> 1
  @mgr.download[1].bytes-done ;; -> 500_000 (preserved across failure!)

  @ytd/retry-policy-backoff{policy: @mgr.retry-policy, attempt: 2}  ;; -> 2000
  @ytd/retry-policy-backoff{policy: @mgr.retry-policy, attempt: 3}  ;; -> 4000
}
```

## Where

Concurrency MUST be a slot count, not threads — thread-based concurrency makes tests non-deterministic and state changes non-atomic. Bandwidth MUST divide equally across active — models TCP fair-sharing; any other split requires explaining why this stream is favored over that one. Bytes-done MUST persist across failures — users expect resume-from-offset; starting over on every retry wastes bandwidth and time. Retry MUST be a scheduled state transition — comparing `elapsed >= ready_at_ms` on each tick is deterministic; a `sleep`-based retry is not. Backoff MUST be a pure function — same attempt, same policy, same ms; any randomness (jitter) must be a separate parameter. Format selection MUST be policy-based — user picks the *policy*, system picks the *format*; this lets policies be saved and applied to new videos. The FSM's Failed state MUST be terminal — abandoning after `max_attempts` is the only way to prevent infinite retry loops. Pending queue order MUST be FIFO — fair to enqueue order; priority ordering is a separate feature.
