# G128 — Screen Saver

> The Rosetta Stone's fifteenth **Graphics project**. Models the **idle-detect + animated-screen-saver kernel** every OS ships: track the last user input, transition through a fade-dim phase when idle too long, run an animated pattern, then auto-lock. The distinctive move: **time-parameterized 2D physics** drives the animation — each line endpoint has a velocity; `tick(ms)` integrates position and reflects off canvas edges. No real-time jitter; integer-scaled for cross-language byte-identity.

```yaml
id: G128
title: Screen Saver
category: graphics
requires: [G117-stream-player, G118-mp3-player, G122-wallpaper-manager]
provides: [idle-fsm, 2d-physics-sim, dim-opacity-ramp, activity-reset]
```

## Insight: Six-State Idle FSM

`Active → Dimming → Saver → Locked → Waking → Active`. All transitions are time-triggered except `Waking`, which is input-triggered (activity + explicit unlock). `Locked` is a terminal-ish state: only `unlock()` escapes it; mere activity doesn't. This mirrors every real OS: a quick mouse wiggle wakes a dimmed screen but doesn't unlock a password prompt.

First Rosetta Stone project where **state transitions are mostly time-triggered, with one explicit user-triggered transition** (unlock). G121's download FSM was also time-triggered except for retry reports; G128 mixes time + input triggers.

## Insight: Physics Is Integer-Scaled for Byte-Identity

Positions and velocities are stored as sub-pixel units × 100:
- `x = 9500` means 95 pixels (precision to 0.01 px).
- `vx = 10_000` means 100 sub-pixel-per-ms × 100 = 1 pixel per ms.

Integration: `x += vx * ms / 100`. Reflection: `x = 2 * bound - x; vx = -vx`. All integer arithmetic; byte-identical across languages.

First Rosetta Stone project with **2D physics simulation** — a time-integrated particle system with edge collisions. G115's radial layout was static positioning; G128 has moving particles.

## Insight: Reflection Preserves Energy

Elastic reflection off a wall: `new_pos = 2 * bound - old_pos; new_vel = -old_vel`. Speed is preserved; direction flips along the bounced axis. This keeps animations visually continuous — lines don't get stuck at walls or lose energy over time.

First Rosetta Stone project where **an invariant (energy conservation) is preserved across a simulated event**. G125 had extension caps; G128 has physical symmetry.

## Insight: Dim Opacity Is a Pure Function of State Time

During `Dimming`, the UI needs an opacity value for blending. Rather than store it, compute it: `opacity = min(since, dim_ms) * 255 / dim_ms` where `since = elapsed - state_entered`. Same inputs, same output. No drift.

First Rosetta Stone project where **a visual-only scalar is computed on demand**, not stored. G125's cycle_side was stored state; G128's dim opacity is derived.

## Insight: Activity Resets But Doesn't Unlock

`record_activity()` updates `last_activity_ms` and:
- In `Active` → idle timer resets; no state change.
- In `Dimming` / `Saver` → transitions to `Waking`.
- In `Locked` → does nothing (activity alone doesn't unlock).

The semantic separation matters: every keystroke or mouse move shouldn't bypass auth. Locked is a security state; only `unlock()` escapes.

First Rosetta Stone project with **a state where the same event has different effects depending on context** — and the "no effect in context X" is an intentional design, not an omission.

## Insight: Physics Only Runs During Saver

`tick()` dispatches on state. Physics integration only happens in `Saver`. In `Active`, `Dimming`, `Locked`, `Waking`, the simulation freezes. This saves computation (no point animating when the screen is blank or auth is pending) and keeps the animation deterministic across pause/resume.

First Rosetta Stone project where **a subsystem only runs during specific FSM states** — gated activation, not just gated output.

## Insight: State-Entered Timestamp Drives Phase Timing

`state_entered_ms` is updated on every transition. `since_state = elapsed - state_entered` tells how long we've been in the current state. Used by Dimming (compare to `dim_ms`), Saver (compare to `auto_lock_ms`), Waking (compare to `wake_ms`). One scalar, used three ways.

First Rosetta Stone project where **a single bookkeeping scalar is reused across multiple phase checks**.

## Choreographic Case: Vault Idle Guardian

```innate
(@vault-idle-guardian){
  @cfg <- {idle-ms: 60000, dim-ms: 3000, auto-lock-ms: 30000, wake-ms: 500,
            canvas-width: 1920, canvas-height: 1080}
  @saver <- @ss/new-saver{config: @cfg}

  ;; Seed lines with random positions/velocities
  (for @i in 0..5{
    @ss/add-line{saver: @saver, line: {
      p1: {x: (rand 0 192000), y: (rand 0 108000),
           vx: (rand -10000 10000), vy: (rand -10000 10000)},
      p2: {x: (rand 0 192000), y: (rand 0 108000),
           vx: (rand -10000 10000), vy: (rand -10000 10000)},
      color: {r: 0, g: 255, b: 255}
    }}
  })

  @on-clock-tick (@ms){
    @ss/tick{saver: @saver, ms: @ms}
    (if (== @saver.state "saver") (@ui/render-saver @saver))
    (if (== @saver.state "dimming")
        (@ui/fade-to-black @saver.dim-opacity))
  }

  @on-input-event{
    @ss/record-activity{saver: @saver}
  }

  @on-password-entered (@ok){
    (if @ok (@ss/unlock @saver))
  }
}
```

## Structures

```innate
(defenum saver-state ACTIVE | DIMMING | SAVER | LOCKED | WAKING)

(defstruct config
  idle-ms          : Int
  dim-ms           : Int
  auto-lock-ms     : Int
  wake-ms          : Int
  canvas-width     : Int
  canvas-height    : Int)

(defstruct endpoint
  x   : Int      ;; sub-pixel × 100
  y   : Int
  vx  : Int      ;; sub-pixel/ms × 100
  vy  : Int)

(defstruct bouncing-line
  p1    : Endpoint
  p2    : Endpoint
  color : Rgb)

(defstruct screen-saver
  config              : Config
  state               : SaverState
  lines               : [BouncingLine]
  elapsed-ms          : Int
  last-activity-ms    : Int
  state-entered-ms    : Int
  events              : [SaverEvent])
```

## Resolver Natives

```innate
@ss/new-saver{config}                       -> ScreenSaver
@ss/add-line{saver, line}                   -> Unit
@ss/tick{saver, ms}                         -> Unit
@ss/record-activity{saver}                  -> Unit
@ss/unlock{saver}                           -> Unit
@ss/dim-opacity{saver}                      -> Int    ;; 0..255
```

## Demo

```innate
(@demo){
  @cfg <- {idle-ms: 1000, dim-ms: 200, auto-lock-ms: 500, wake-ms: 100,
            canvas-width: 100, canvas-height: 100}
  @s <- @ss/new-saver{config: @cfg}
  @ss/add-line{saver: @s, line: {
    p1: {x: 9500, y: 5000, vx: 10000, vy: 0},
    p2: {x: 5000, y: 5000, vx: 0, vy: 0},
    color: {r: 255, g: 255, b: 255}
  }}

  @s.state                               ;; -> ACTIVE
  @ss/tick{saver: @s, ms: 500}
  @s.state                               ;; -> ACTIVE (idle = 500ms < 1000ms)
  @ss/tick{saver: @s, ms: 600}
  @s.state                               ;; -> DIMMING
  @ss/dim-opacity{saver: @s}             ;; -> 0 (just entered)
  @ss/tick{saver: @s, ms: 100}
  @ss/dim-opacity{saver: @s}             ;; -> 127 (half of dim_ms=200)
  @ss/tick{saver: @s, ms: 100}
  @s.state                               ;; -> SAVER

  ;; Physics: line at x=9500 with vx=10000 (1 px/ms) moves right 10 px in 10ms
  ;; hits edge at 10000, reflects to x=9500 with vx=-10000
  @ss/tick{saver: @s, ms: 10}
  @s.lines[0].p1.x                       ;; -> 9500
  @s.lines[0].p1.vx                      ;; -> -10000

  @ss/tick{saver: @s, ms: 600}
  @s.state                               ;; -> LOCKED (past auto_lock_ms)

  @ss/record-activity{saver: @s}
  @s.state                               ;; -> LOCKED (activity doesn't unlock)

  @ss/unlock{saver: @s}
  @s.state                               ;; -> WAKING
  @ss/tick{saver: @s, ms: 150}
  @s.state                               ;; -> ACTIVE (past wake_ms=100)
}
```

## Where

Locked MUST NOT wake on activity — a screen lock is a security boundary; activity-wakes would defeat its purpose. The transition from Locked to Waking MUST be via an explicit `unlock()` call — implicit activation on any event breaks the boundary. Physics integration MUST only run in Saver state — animating during Dimming wastes CPU on a fading screen; animating during Active wastes CPU on a visible desktop. Integer-scaled positions/velocities MUST be the norm — floats diverge across languages, defeating Rosetta Stone's parity contract. Reflection MUST preserve speed (flip sign only) — inelastic decay is a separate concern (friction), not baseline physics. Dim opacity MUST be computed, not stored — a stored value drifts with tick noise; a computed value is the truth. State-entered timestamp MUST be updated on every transition — stale values produce wrong durations for subsequent time checks. Events MUST log every transition — post-hoc analysis of "how did we get locked?" needs the trail.
