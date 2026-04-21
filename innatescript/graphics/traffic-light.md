# G125 — Traffic Light Application

> The Rosetta Stone's twelfth **Graphics project**. Models the **intersection controller** every traffic cabinet runs: a coordinated super-state FSM (NSGreen → NSYellow → AllRed → EWGreen → EWYellow → AllRed → NSGreen → ...), emergency preemption that saves and restores the phase, adaptive green extension when queued vehicles are detected, and a pedestrian walk sub-phase gated to AllRed. The distinctive move: **each direction's light color is derived from a single coordinated phase**, so impossible combinations (both directions green simultaneously) are unrepresentable — the phase is the single source of truth, and red/yellow/green per direction are read-only projections.

```yaml
id: G125
title: Traffic Light Application
category: graphics
requires: [G073-telnet-server, G117-stream-player, G118-mp3-player]
provides: [coordinated-super-state, preemption-with-state-preservation, adaptive-timing, sub-phase-gating]
```

## Insight: Coordinated Super-State Makes Invalid Combinations Unrepresentable

Naively, you'd model each direction's light as an independent `{Red, Yellow, Green}`. But that lets a bug set both to Green — a crash. The coordinated approach: one `Phase` value (six variants including the emergency state) drives both directions. `ns_light()` and `ew_light()` are pure functions of `Phase`. No invalid state is representable.

First Rosetta Stone project where **a multi-axis state is represented as one coordinated super-state** with per-axis projections, not as independent axes. G117's player FSM was single-axis; G125 has NS and EW sub-axes that must not contradict.

## Insight: Emergency Preemption with State Preservation

`begin_emergency()` saves `(phase, phase_elapsed_ms, cycle_side)`, forces the super-state to `EmergencyAllRed`, freezes the phase clock. `end_emergency()` restores the saved tuple — the intersection resumes exactly where it left off. Same pattern as G121's retry: store the identity, not the derived position.

`EmergencyAllRed` is a distinct variant, not a flag — it participates in the `Phase` enum. That means `ns_light()` / `ew_light()` handle it naturally (both red), and `tick()` knows to return early without advancing.

First Rosetta Stone project where **preemption preserves resumable state across an override**. G121's retry stored bytes_done; G125 stores phase + elapsed.

## Insight: Adaptive Green Extension With Queue-Driven Cap

When `phase_elapsed_ms` reaches `phase_duration_ms` on a green phase, check the queue for that direction. If `queue > 0` and total extension is under the cap, extend `phase_duration_ms` by `queue * 2000ms` (capped at `max_green_extension_ms`).

The check happens **once per phase** — not per tick — so the extension is deterministic: same queue at end-of-nominal-green, same extension. No oscillation as vehicles arrive during extension.

First Rosetta Stone project where **a phase duration is recomputed once at end-of-nominal**, not continuously. G117's ABR was per-segment; G125 is per-phase-end.

## Insight: Pedestrian Walk Sub-Phase Gated to AllRed

A pedestrian request is *queued* with `ped_requested: bool`. The walk phase **only activates when the super-state transitions to AllRed** — this is safe, since no cars are moving. Once activated, walk runs its own internal FSM: Inactive → Walk (for `ped_walk_ms`) → Flashing (for `ped_flashing_ms`) → Inactive.

Pedestrian time is tracked separately from phase time — the walk may outlast the AllRed phase (continuing into the next green) or finish early. Tests assert both cases.

First Rosetta Stone project with **a secondary FSM gated by a specific primary super-state**. G117/G118 had single FSMs; G125 has a ped-FSM coupled to the super-FSM only at one transition.

## Insight: Phase Durations Are Config, Not Hardcoded

```
Config {
    ns_green_ms, ns_yellow_ms,
    ew_green_ms, ew_yellow_ms,
    all_red_ms,
    max_green_extension_ms,
    ped_walk_ms, ped_flashing_ms,
}
```

All timings in one struct. Tuning an intersection (e.g., longer green for a busy arterial) is a config change, not code. Same-config-same-behavior is the determinism guarantee.

First Rosetta Stone project where **every timing constant is surfaced in a single config struct**. G117 had safety factor + target buffer; G125 has eight independent timings.

## Insight: Overflow Carry Across Multiple Phase Transitions

A `tick(ms)` can exceed the current phase's remaining duration. After transitioning, the overflow carries into the new phase. If the new phase is also shorter than the overflow, transition again. The tick loop continues until overflow is consumed or emergency state is reached.

```
tick(1500) while phase = NSYellow (200ms remaining):
    phase_elapsed = 200+1500 → transition → AllRed (overflow 1500)
    AllRed dur = 100 → transition → EWGreen (overflow 1400)
    EWGreen dur = 1000 → transition → EWYellow (overflow 400)
    ...
```

First Rosetta Stone project where **a single tick can traverse multiple FSM transitions**. G117's player advanced one transition per tick; G125's intersection may traverse several if ticks are coarse.

## Insight: Pedestrian Activation Mid-Tick Gets Remaining Overflow Only

When the walk phase activates *during* the tick processing (because a phase transition triggered it), the pedestrian clock must only advance by `overflow_after_transition`, not the full `ms`. Otherwise a ped would consume 1200ms of walk time when walking only activated at the 1000ms mark of a 1200ms tick.

The fix: track `ped_just_started_at_remaining: Option<u64>` during the transition loop; use that value if set, else the full `ms`.

First Rosetta Stone project with **tick-bookkeeping that correctly attributes time to a sub-FSM only from its activation moment**.

## Choreographic Case: Vault Intersection Simulator

```innate
(@vault-intersection-sim){
  @cfg <- {ns-green-ms: 20000, ns-yellow-ms: 3000,
            ew-green-ms: 20000, ew-yellow-ms: 3000,
            all-red-ms: 2000, max-green-extension-ms: 10000,
            ped-walk-ms: 8000, ped-flashing-ms: 5000}
  @light <- @tl/new-intersection{config: @cfg}

  @on-clock-tick (@ms){
    @tl/tick{intersection: @light, ms: @ms}
    @ui/render-lights{ns: @light.ns-light, ew: @light.ew-light,
                        ped: @light.ped-signal}
  }

  @on-ns-sensor-detect (@count){
    @tl/set-ns-queue{intersection: @light, count: @count}
  }

  @on-ped-button{
    @tl/request-ped-crossing{intersection: @light}
  }

  @on-emergency-vehicle{
    @tl/begin-emergency{intersection: @light}
    @ui/alert{message: "Emergency vehicle approaching"}
  }

  @on-emergency-cleared{
    @tl/end-emergency{intersection: @light}
  }
}
```

The vault's intersection shell wires sensor events to queue counters and the emergency channel to the preemption FSM; everything else is derived.

## Structures

```innate
(defenum phase
  NS_GREEN | NS_YELLOW | ALL_RED | EW_GREEN | EW_YELLOW | EMERGENCY_ALL_RED)
(defenum light-color RED | YELLOW | GREEN)
(defenum ped-signal DONT_WALK | WALK | FLASHING)
(defenum direction NS | EW)

(defstruct config
  ns-green-ms               : Int
  ns-yellow-ms              : Int
  ew-green-ms               : Int
  ew-yellow-ms              : Int
  all-red-ms                : Int
  max-green-extension-ms    : Int
  ped-walk-ms               : Int
  ped-flashing-ms           : Int)

(defstruct intersection
  phase                 : Phase
  phase-elapsed-ms      : Int
  phase-duration-ms     : Int
  cycle-side            : Direction
  config                : Config
  ns-queue              : Int
  ew-queue              : Int
  ped-requested         : Bool
  ped-state             : PedState
  saved-phase           : (Phase, Int, Direction)?
  total-elapsed-ms      : Int
  events                : [IntersectionEvent])
```

## Resolver Natives

```innate
@tl/new-intersection{config}                        -> Intersection
@tl/tick{intersection, ms}                          -> Unit
@tl/set-ns-queue{intersection, count}               -> Unit
@tl/set-ew-queue{intersection, count}               -> Unit
@tl/request-ped-crossing{intersection}              -> Unit
@tl/begin-emergency{intersection}                   -> Unit
@tl/end-emergency{intersection}                     -> Unit
@tl/ns-light{intersection}                          -> LightColor
@tl/ew-light{intersection}                          -> LightColor
@tl/ped-signal{intersection}                        -> PedSignal
```

## Demo

```innate
(@demo){
  @cfg <- {ns-green-ms: 1000, ns-yellow-ms: 200,
            ew-green-ms: 1000, ew-yellow-ms: 200,
            all-red-ms: 100, max-green-extension-ms: 500,
            ped-walk-ms: 500, ped-flashing-ms: 300}
  @i <- @tl/new-intersection{config: @cfg}
  @i.phase            ;; -> NS_GREEN
  @i.ns-light         ;; -> GREEN
  @i.ew-light         ;; -> RED

  @tl/tick{intersection: @i, ms: 1000}
  @i.phase            ;; -> NS_YELLOW (NSGreen drained)
  @tl/tick{intersection: @i, ms: 200}
  @i.phase            ;; -> ALL_RED
  @tl/tick{intersection: @i, ms: 100}
  @i.phase            ;; -> EW_GREEN (cycle-side flipped)

  ;; Adaptive: 3 vehicles queued at NSGreen → extend by 500 ms (capped)
  @j <- @tl/new-intersection{config: @cfg}
  @tl/set-ns-queue{intersection: @j, count: 3}
  @tl/tick{intersection: @j, ms: 1000}
  @j.phase-duration-ms  ;; -> 1500

  ;; Emergency: preserves and restores state
  @k <- @tl/new-intersection{config: @cfg}
  @tl/tick{intersection: @k, ms: 300}
  @tl/begin-emergency{intersection: @k}
  @k.phase              ;; -> EMERGENCY_ALL_RED
  @tl/tick{intersection: @k, ms: 10000}
  @k.phase              ;; -> EMERGENCY_ALL_RED (frozen)
  @tl/end-emergency{intersection: @k}
  @k.phase              ;; -> NS_GREEN (resumed)
  @k.phase-elapsed-ms   ;; -> 300 (preserved)

  ;; Pedestrian request: gated to AllRed
  @l <- @tl/new-intersection{config: @cfg}
  @tl/request-ped-crossing{intersection: @l}
  @l.ped-signal         ;; -> DONT_WALK (waiting)
  @tl/tick{intersection: @l, ms: 1200}  ;; NSGreen → NSYellow → AllRed; walk starts
  @l.ped-signal         ;; -> WALK
  @tl/tick{intersection: @l, ms: 500}
  @l.ped-signal         ;; -> FLASHING
  @tl/tick{intersection: @l, ms: 300}
  @l.ped-signal         ;; -> DONT_WALK
}
```

## Where

Phase MUST be a closed ADT — independent per-direction colors allow impossible combinations (both green = crash). Light colors MUST be projections of phase — two sources of truth for direction color guarantees drift. Emergency MUST save full phase state (phase + elapsed + cycle_side) — restoring only the phase loses timing and the cycle direction. Green extension MUST be considered once at end-of-nominal — per-tick re-evaluation oscillates. Extension MUST be capped — an always-queued arterial would starve cross traffic indefinitely. Pedestrian walk MUST gate to AllRed — activating walk during green is a safety violation. Pedestrian timer MUST only advance by time-since-activation on the first tick after a mid-tick activation — full tick-ms double-counts. The tick loop MUST handle multiple transitions per call — coarse ticks (e.g., simulation 1 tick = 1 minute) skip multiple phases at once. Config struct MUST collect all timings — scattered constants prevent per-intersection tuning.
