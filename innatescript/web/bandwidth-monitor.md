# G075 — Bandwidth Monitor

> The Rosetta Stone's first project with **time series as primary data structure**. Raw observations are monotonic counters; the useful signal is the *derivative* — bytes per second over a rolling window. Average rate smooths noise; peak rate preserves spikes. Counter wraparound is a first-class concern handled by skipping unmeasurable intervals rather than reporting nonsense negative rates.

```yaml
id: G075
title: Bandwidth Monitor
category: web
requires: [G034-atomic-time, G065-progress-bar]
provides: [time-series-buffer, counter-to-rate, rolling-window-averages, peak-tracking, wraparound-handling]
```

## Insight: The Useful Signal Is the Derivative, Not the Counter

A network interface tells you one thing: a monotonically-increasing byte counter. That counter, on its own, is useless — "the machine has sent 47,385,219 bytes since it booted" is not information anyone acts on. The **signal** is bytes per second *right now*, *on average over the last minute*, *at peak over the last five minutes*.

G075's design recognises this. The stored data is the counter stream; the derived data is every useful rate. Every rate query walks the counter stream, picks two samples spanning the window, divides byte-delta by time-delta. The counter is the source of truth; the rates are projections.

First Rosetta Stone project where **the stored form and the useful form are different**. Every prior project had queries that returned values stored directly (current balance, user count, tag list). G075 stores counters but never returns counters — only derived rates. The transformation happens on every query, not on write.

Parallels everywhere: CPU percent is `(cpu_busy_delta / wall_delta)`; disk I/O is `(bytes_delta / wall_delta)`; request throughput is `(count_delta / wall_delta)`. Every metrics system — Prometheus, StatsD, the vault's future observability layer — has this shape: store monotonic counters, derive rates on query.

## Insight: Sampling Cadence Trades Precision for Overhead

G075 doesn't sample — the caller passes in samples. But the choice of when to sample is the central performance question for any real monitor.

- **Too infrequent**: misses bursts. A 1-second-average over a 100-ms spike sees `spike_bytes / 1000ms` — the spike looks 10× smaller than it was.
- **Too frequent**: wastes CPU measuring things that don't change. Sampling every millisecond on a GB/s link generates 1M samples/s of overhead to detect a rate that changes perhaps 10 times/s.

Real monitors choose around 1 Hz for general-purpose network stats and higher frequencies for debugging bursts. G075 presents the primitive; the sampling schedule is the caller's choice.

First Rosetta Stone project where **the observation frequency is itself a design parameter**, separate from the observation primitive. The noosphere's agent-metric collection will face this: sample choreography-step counters on a cadence that catches interesting behaviour without drowning in data.

## Insight: Average Smooths, Peak Preserves

A network link running steady at 1 MB/s with a 100 ms 10 MB/s spike has:
- Average rate over 1 s ≈ (10 MB spike + 9× 0.1MB baseline) / 1 s ≈ 1.9 MB/s (almost invisible)
- Peak rate over 1 s = 10 MB/s (exactly the spike)

Average and peak are **different statistics on the same data**, and both matter. A saturated-link-spike problem is visible in peak but hidden in average. A slow-sustained-growth problem is visible in average but not peak. Production monitors report both.

First Rosetta Stone project where **two different summary statistics answer different questions about the same data**. Same philosophical shape as G063 Josephus (survivor vs elimination-order), G067 Chat (log vs inbox), G074 Whiteboard (retained structure vs rendered output). Pick the algorithm to match the question; store the raw data so multiple algorithms can be applied.

## Insight: Counter Wraparound Is an Adversary, Not an Error

Real 32-bit byte counters wrap at 4 GB. On a 100 Mbit link, that's about every 5.5 minutes. A naive rate calculation `(bytes_now - bytes_before)` goes *negative* across the wrap and the rate becomes implausible.

G075 treats wraparound as a **gap in observation**, not an error. If the new count is less than the old count, that pair is skipped — the interval's contribution to total bytes and to rate calculations is zero, not negative. The rate "resumes" with the next pair of consecutive increasing counts.

Alternatives exist (e.g., assume one wraparound happened and compute `(new + 2^32 - old)`), and they're fine if you actually know the counter width. G075's approach — skip what you can't measure — is safe without that knowledge.

First Rosetta Stone project where **the adversarial-data model is explicit**. Previous projects trusted their inputs; G075 treats the input as potentially misbehaved (because it is). Every real metrics system does this: Prometheus has explicit `reset` detection; statsd treats negative deltas as counter resets; the Linux `/proc/net/dev` format documents that counters can reset.

## Insight: Ring Buffer Is the Right Shape

Time-series storage that keeps only a recent window is a ring buffer. G075 uses a fixed-capacity deque that evicts the oldest when full. Queries walk the buffer; they don't paginate, aggregate externally, or need a database. Bounded memory, bounded query cost.

First Rosetta Stone project where **bounded storage** is an explicit design property. Previous projects assumed collections grow unboundedly (Family Tree, Chat Room's log, WYSIWYG's runs); G075 caps the buffer because time-series data is "most recent N" semantics, not "everything ever."

Same pattern in systems: the kernel's event ring buffer for perf; syslog rotation; Prometheus's TSDB chunks; the vault's activity log with rolling archive. Every observability layer caps its buffer because unbounded observation buffers eat disks and memory without bound.

## Choreographic Case: Per-Agent Throughput Monitoring

```innate
(@monitor-agents){
  @monitors <- @agents/all.map(fun(a){
    {agent: @a, monitor: @bw/new-monitor{capacity: 300}}
  })

  @every 1.s {
    @for m in @monitors {
      @count <- @agent/message-count{agent: @m.agent}
      @bw/record{monitor: @m.monitor, timestamp: @now, bytes_total: @count}

      @avg <- @bw/average-rate{monitor: @m.monitor, window_ms: 60_000}
      @peak <- @bw/peak-rate{monitor: @m.monitor, window_ms: 300_000}

      where { above_alert: @peak > @m.agent.peak_threshold }
      @alert/dispatch{agent: @m.agent, avg: @avg, peak: @peak}
    }
  }
}
```

The choreography composes directly on G075: per-agent monitors, sampled on a cadence, averaged for health, peaked for alerts. Standard observability shape with nothing bespoke.

## Structures

```innate
(defstruct sample
  timestamp-ms  : Int
  bytes-total   : Int)           ;; monotonic counter, may wrap

(defstruct monitor
  samples       : [Sample]       ;; ring buffer, oldest first
  capacity      : Int)
```

## Resolver Natives

```innate
@bw/new-monitor{capacity}                         -> Monitor
@bw/record{monitor, timestamp, bytes_total}       -> Unit
@bw/sample-count{monitor}                         -> Int
@bw/total-bytes{monitor}                          -> Int      ;; sum of deltas, wraparound skipped
@bw/instantaneous-rate{monitor}                   -> Float    ;; B/s between last two samples
@bw/average-rate{monitor, window_ms}              -> Float    ;; B/s over window
@bw/peak-rate{monitor, window_ms}                 -> Float    ;; max B/s between any pair in window
```

## Demo

```innate
(@demo){
  @m <- @bw/new-monitor{capacity: 20}
  @total <- 0
  @for i in 0..15 {
    @delta <- (if @i == 7 then 50_000 else 1_000)
    @total <- @total + @delta
    @bw/record{monitor: @m, timestamp: @i * 1000, bytes_total: @total}
  }
  @bw/total-bytes{monitor: @m}                         ;; -> 63_000
  @bw/instantaneous-rate{monitor: @m}                  ;; -> 1_000.0  (baseline after spike)
  @bw/average-rate{monitor: @m, window_ms: 10_000}     ;; -> 5_900.0  (smoothed over window)
  @bw/peak-rate{monitor: @m, window_ms: 10_000}        ;; -> 50_000.0 (the actual spike)
}
```

## Where

The monitor MUST require capacity ≥ 2 at construction — one sample cannot measure a rate. Record MUST evict the oldest sample when over capacity — the ring buffer's bounded memory is a correctness property, not an optimisation. Wraparound (a sample with smaller bytes_total than its predecessor) MUST be skipped in every derived calculation — reporting a negative rate is a bug, not a feature. The total-bytes query MUST NOT include wraparound intervals. Rate calculations with zero time delta MUST return zero — dividing by zero is undefined, returning infinity is worse than returning zero because downstream code rarely checks for infinity. Peak rate is the max of instantaneous rates between consecutive pairs starting within the window — it is NOT the max of any-two-points rates, which would be an undefined quantity at unbounded look-back. Average rate uses the FIRST sample in the window and the LAST sample — not a sum of intermediate rates, which would be more expensive and equivalent.
