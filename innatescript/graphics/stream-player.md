# G117 — Stream Video from Online

> The Rosetta Stone's fourth **Graphics project**. Models the **adaptive-bitrate streaming (ABR)** kernel every HLS/DASH/MSS client ships: a stream has multiple bitrate renditions; each segment is downloaded one at a time; the ABR algorithm picks each segment's quality based on **buffer health** and **observed bandwidth**. The distinctive move: ABR is a **pure function** over `(buffer_ms, bandwidth_bps, segment_bytes_per_bitrate)` → chosen bitrate. Playback is a **five-state FSM** (Idle → Buffering → Playing → Paused → Ended) driven by buffer level and segment availability.

```yaml
id: G117
title: Stream Video from Online
category: graphics
requires: [G065-progress-bar, G082-cms, G109-tv-show-tracker, G116-grayscale-converter]
provides: [adaptive-bitrate-selection, segmented-playback, playback-fsm, event-log]
```

## Insight: ABR Is a Pure Function

Every HLS/DASH client's quality-selection logic reduces to: *given a buffer level (ms), a bandwidth estimate (bps), and a segment of known size (bytes) at each candidate bitrate, pick the highest bitrate whose estimated download time fits inside the buffer.* Stated that way, it's a pure function.

```
effective_bw = bandwidth_bps * safety_factor    // usually 0.85
budget_ms    = max(buffer_ms, segment_duration_ms)  // never less than one segment
for each bitrate ascending:
    download_ms = bytes * 8000 / effective_bw
    if download_ms <= budget_ms: chosen = bitrate
```

The safety factor hedges against bandwidth estimate error. The buffer-vs-segment-duration max ensures the startup case (empty buffer) still considers all bitrates instead of locking at the lowest.

First Rosetta Stone project where **a pure function over measurable inputs drives a UX-critical choice**. G086's frecency ranking was similar — scalar inputs to scalar output — but G117's output selects from a set (bitrate list), and the stakes are ongoing (every segment re-runs the decision).

## Insight: Buffer Budget Has Two Floors

Naive "download must fit in buffer" breaks at startup (buffer = 0 means no bitrate fits). The fix: the budget floor is **one segment duration** — download must fit in "the time this segment will play for once we have it". Also applies mid-playback if the buffer dips below one segment.

Combined with a cap (`min(raw_budget, target_buffer_ms * 2)`), the ABR is bounded from both ends — never picks absurdly low in startup, never picks absurdly high when buffer is already deep.

First Rosetta Stone project where **a single numeric input is clamped by both a floor and a ceiling** with domain-specific meaning. G107's budget variance used sentinels; G117 uses real clamps tied to segment timing.

## Insight: Playback Is a Five-State FSM

Idle → (attach stream) → Buffering → (enough buffer) → Playing → (pause) → Paused → (play) → Playing → (buffer drained, more segments) → Buffering → ... → Ended.

Transitions are **driven by two things**: the buffer level crossing thresholds, and segment availability relative to the stream's length. No other inputs matter. Pause/play are user-triggered; every other transition is computed from state.

First Rosetta Stone project with **a five-state FSM driven by numeric thresholds + list boundaries**. G073 had a telnet protocol FSM (command-driven); G117's FSM is level-driven (continuous variables crossing lines).

## Insight: Events Are the Audit Log

Every state transition, every bitrate switch, every rebuffer, every seek gets a `PlayerEvent`. The event list is the player's ledger — tests can assert "a rebuffer happened"; production monitoring can aggregate rebuffer rate per session; UI can animate transitions from observed events.

Same pattern as G094's append-only logs and G098's copy events — side effects captured as values, not discarded.

First Rosetta Stone project where **an FSM emits an event per transition** as a first-class output. Enables test assertions about *behaviour*, not just *final state*.

## Insight: Multi-Bitrate Means a Per-Segment Size Map

A stream has N bitrates; each segment comes in N sizes. `Segment.sizes_by_bitrate: {bitrate → bytes}` captures the manifest. The ABR picks from this map per segment; different segments can have different sizes at the same bitrate (VBR encoding — variable bitrate with per-chunk variation).

First Rosetta Stone project where **the same logical unit has multiple physical representations** selected at query time. G112 had dialect-specific DDL; G117 has bitrate-specific segment bytes. Same shape: one abstract thing, many concrete forms, picked at use.

## Insight: Seek Clears Buffer

Seeking to a different segment invalidates the current buffer — the bytes in flight are for the wrong playhead. The seek resets `buffer_ms = 0`, updates `played_ms` to the cumulative duration of segments before the target, and transitions to Buffering.

Every real player does this. Skipping this step creates "ghost playback" where old buffer content plays for a moment after seek.

First Rosetta Stone project where **a user action invalidates an internal cache** with visible consequences. G098's bulk renames invalidated paths; G117's seek invalidates the buffer.

## Choreographic Case: Vault Video Library

```innate
(@vault-video-library){
  @stream <- @hls/parse-manifest{url: @video.manifest-url}
  @player <- @sp/new{}
  @sp/attach{player: @player, stream: @stream}

  @on-network-tick (@new-bandwidth){
    @sp/set-bandwidth{player: @player, bandwidth-bps: @new-bandwidth}
    @when (@should-load-more-segments){
      @sp/load-next-segment{player: @player}
    }
  }

  @on-playback-tick (@elapsed-ms){
    @sp/tick{player: @player, ms: @elapsed-ms}
    @ui/render-playhead{played: @player.played-ms,
                         total: @stream.total-duration-ms,
                         state: @player.state,
                         bitrate: @player.current-bitrate}
  }

  @on-user-scrubs (@segment-idx){
    @sp/seek-to-segment{player: @player, segment-idx: @segment-idx}
  }
}
```

The vault's video-player shell is a thin wrapper: network ticks feed bandwidth, playback ticks drain buffer, user scrub triggers seek. ABR and state transitions happen inside the player; the UI watches state.

## Structures

```innate
(defenum player-state IDLE | BUFFERING | PLAYING | PAUSED | ENDED)

(defstruct segment
  index              : Int
  duration-ms        : Int
  sizes-by-bitrate   : {Int -> Int})

(defstruct stream
  bitrates-bps       : [Int]
  segments           : [Segment]
  safety-factor      : Float
  target-buffer-ms   : Int)

(defenum event-kind
  STATE_CHANGED | SEGMENT_LOADED | BITRATE_SWITCHED | REBUFFER | SEEKED)

(defstruct player-event
  kind   : EventKind
  detail : String)

(defstruct player
  state              : PlayerState
  current-segment    : Int
  buffer-ms          : Int
  played-ms          : Int
  bandwidth-bps      : Int
  current-bitrate    : Int
  events             : [PlayerEvent]
  stream             : Stream?)
```

## Resolver Natives

```innate
@sp/new{}                                            -> Player
@sp/attach{player, stream}                           -> Unit
@sp/set-bandwidth{player, bandwidth-bps}             -> Unit
@sp/pick-bitrate{stream, buffer-ms, bandwidth-bps, segment} -> Int
@sp/load-next-segment{player}                        -> Bool
@sp/tick{player, ms}                                 -> Unit
@sp/pause{player}                                    -> Unit
@sp/play{player}                                     -> Unit
@sp/seek-to-segment{player, segment-idx}             -> Bool
```

## Demo

```innate
(@demo){
  @stream <- {bitrates-bps: [500_000, 1_000_000, 2_000_000, 4_000_000],
              segments: [...5 segments × 4 sizes...],
              safety-factor: 0.85,
              target-buffer-ms: 6000}

  @p <- @sp/new{}
  @sp/attach{player: @p, stream: @stream}
  @sp/set-bandwidth{player: @p, bandwidth-bps: 10_000_000}

  (repeat 3 @sp/load-next-segment{player: @p})
  @p.state          ;; -> PLAYING (buffer reached target)
  @p.current-bitrate ;; -> 4_000_000 (highest that fits)

  @sp/tick{player: @p, ms: 6000}
  @p.state          ;; -> BUFFERING (buffer drained, more segments available)
}
```

## Where

ABR MUST be a pure function — same inputs must yield the same bitrate, or A/B tests of ABR logic become impossible. Buffer budget MUST have a one-segment-duration floor — startup and low-buffer cases need a sane fallback so all bitrates are reachable. Safety factor MUST apply to bandwidth — real-world bandwidth estimates have error; running the math at exactly measured bandwidth leaves no headroom. Playback state MUST be a closed enum — implicit states (like a boolean `is_playing`) make invalid combinations representable. Seek MUST reset buffer AND transition to Buffering — stale buffer content plays wrong content for the new position. Events MUST be emitted per transition — tests and telemetry need behavioural assertions, not just final state. End-of-stream MUST be distinguished from mid-stream empty buffer — one transitions to Ended, the other to Buffering; conflating them breaks "playback completed" UX.
