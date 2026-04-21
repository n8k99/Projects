# G122 — Wallpaper Manager

> The Rosetta Stone's ninth **Graphics project**. Models the **multi-monitor wallpaper rotation** every desktop compositor ships: a collection of wallpapers (each with multiple resolution variants), a set of monitors, an independent rotation state per monitor, a schedule policy (interval / fixed-list / tag-window), a resolution best-fit score picking the right variant per monitor, and a recency-weighted selection avoiding the last-N shown. The distinctive move: **the manager holds a mapping from monitor to wallpaper**, not a single cursor. N monitors rotate in parallel, each with its own history and schedule alignment.

```yaml
id: G122
title: Wallpaper Manager
category: graphics
requires: [G096-rpg-stats, G116-grayscale-converter, G118-mp3-player]
provides: [multi-slot-assignment, resolution-fit-score, recency-weighted-pick, schedule-policies]
```

## Insight: Multi-Slot Assignment Instead of a Single Cursor

G118's player had one `queue_position`. G122 has N monitor slots, each with its own `current_wallpaper_id`, `last_rotation_ms`, and `history`. A rotation on monitor 1 doesn't affect monitor 2. This is the general pattern whenever "the selected item" is ambient, not modal — multiple selections coexist, each bound to a context.

First Rosetta Stone project where **selection is a map from context to item**, not a single global pointer. G116 had one active image; G122 has N active images, one per monitor.

## Insight: Resolution Best-Fit Is a Scoring Function

`resolution_fit_score(monitor, variant)` is a pure function returning `i64`. Three contributions:
1. **Exact match** → 1000 (short-circuit).
2. **Aspect-ratio match** → up to 500 based on AR delta (0 delta = 500, 0.5 delta = 0).
3. **Pixel-count proximity** → up to 300, with asymmetry: upscaling penalized more than downscaling (an image twice as wide as the monitor scales down gracefully; half as wide does not).

The asymmetry matters: a 1080p monitor with both 720p and 4K variants prefers the 4K (slight quality loss from downscale) over the 720p (visible upscale artifacts).

First Rosetta Stone project where **a matching score has directional asymmetry**. G086's frecency was symmetric in its factors; G122's fit score penalizes "below" differently than "above".

## Insight: Recency-Weighted Pick with Graceful Fallback

`recency_window: int` — the last N picks are excluded from selection. If N exceeds the eligible pool (after tag filtering), fall back to the full pool. Without fallback, the system deadlocks when recency > pool size.

Pure determinism: same seed + same history → same pick. The LCG advances once per call (whether eligible or fallback), so the sequence is reproducible across test runs.

First Rosetta Stone project where **a recency filter has an explicit fallback when the filter would exclude everything**. G097's flashcard spacing didn't need it (always at least one card ready); G122 needs it because tag filtering can shrink the pool arbitrarily.

## Insight: Schedule Is a Policy Value, Not a Timer

```
Schedule::{Interval(ms), FixedList(times_ms), TagWindow(tag, start_ms, end_ms, inner_ms)}
```

- **Interval** — rotate every `ms` milliseconds per monitor.
- **FixedList** — rotate at specific ms timestamps (like cron at specific times of day).
- **TagWindow** — within a window (mapped to time-of-day), constrain picks to a tag; outside the window, fall back to unconstrained. Rotate every `inner_ms` within the window.

`should_rotate(monitor)` is a pure function of schedule + monitor_state + elapsed. No timers; no sleeps. `tick(ms)` advances elapsed and checks each monitor.

First Rosetta Stone project where **scheduling has multiple policy shapes** (frequency-based vs timestamp-based vs window+tag-based) unified under one abstract interface. G121 had a single retry policy; G122 has three schedule shapes with distinct state.

## Insight: Time-of-Day Mapping via Modulo

For `TagWindow`, "9am-6pm" is `(9 * 3600_000, 18 * 3600_000)`. `elapsed_ms % (24 * 3600_000)` maps elapsed to a time-of-day slot. This lets the manager simulate days of rotation with a single clock advancing monotonically.

First Rosetta Stone project to **map a monotonic clock to a circular-time-of-day slot via modulo**. G111's topo sort was DAG-ordered; G122's time has both monotonic (elapsed) and cyclic (time-of-day) readings.

## Insight: Rotation Is At-Most-Once Per Tick Per Monitor

If a `FixedList` schedule has three times in a single tick, the manager rotates **once**, not three times. Cumulative wallpaper "catch-up" flashing through missed rotations would be worse UX than the single end-state rotation. Test asserts this explicitly.

First Rosetta Stone project where **multiple missed scheduled events collapse to a single action per tick**. G121's downloader also rotated "at most once per tick" but without the explicit collapse assertion.

## Choreographic Case: Vault Workspace Theming

```innate
(@vault-workspace-theming){
  @monitors <- @wayland/monitors{}
  @wallpapers <- @vault/wallpapers{}
  @mgr <- @wm/new-manager{
    schedule: {kind: "tag-window", tag: "focus",
                start-ms: 9h, end-ms: 17h, inner-ms: 30m},
    recency-window: 3,
    seed: 42
  }

  (for @m in @monitors @wm/add-monitor{manager: @mgr, monitor: @m})
  (for @w in @wallpapers @wm/add-wallpaper{manager: @mgr, wallpaper: @w})

  @on-clock-tick (@elapsed-ms){
    @wm/tick{manager: @mgr, ms: @elapsed-ms}
    (for @m in @monitors{
      @variant <- @wm/current-variant-for-monitor{manager: @mgr, monitor-id: @m.id}
      (if @variant
          @wayland/set-wallpaper{monitor: @m, path: @variant.path}
          @ui/log{level: "warn", msg: "no variant for monitor ${@m.id}"})
    })
  }
}
```

The vault's wallpaper daemon is a thin wrapper: tick advances the schedule, each monitor gets its current best-fit variant pushed to the compositor. No additional logic needed beyond the manager.

## Structures

```innate
(defstruct resolution
  width  : Int
  height : Int)

(defstruct monitor
  id         : Int
  resolution : Resolution)

(defstruct wallpaper-variant
  resolution : Resolution
  path       : String)

(defstruct wallpaper
  id       : Int
  name     : String
  tags     : [String]
  variants : [WallpaperVariant])

(defenum schedule-kind INTERVAL | FIXED_LIST | TAG_WINDOW)

(defstruct schedule
  kind              : ScheduleKind
  interval-ms       : Int
  rotation-times-ms : [Int]
  tag               : String
  start-ms          : Int
  end-ms            : Int
  inner-ms          : Int)

(defstruct monitor-state
  current-wallpaper-id : Int?
  last-rotation-ms     : Int
  history              : [Int])

(defstruct manager
  schedule        : Schedule
  recency-window  : Int
  lcg-state       : Int
  monitors        : [Monitor]
  wallpapers      : [Wallpaper]
  assignments     : {Int -> MonitorState}
  elapsed-ms      : Int
  events          : [ManagerEvent])
```

## Resolver Natives

```innate
@wm/new-manager{schedule, recency-window, seed}         -> Manager
@wm/add-monitor{manager, monitor}                        -> Unit
@wm/add-wallpaper{manager, wallpaper}                    -> Unit
@wm/tick{manager, ms}                                    -> Unit
@wm/rotate-monitor{manager, monitor-id, tag-filter?}     -> Bool
@wm/current-variant-for-monitor{manager, monitor-id}     -> WallpaperVariant?
@wm/resolution-fit-score{monitor-res, variant-res}       -> Int
@wm/best-variant-for-monitor{wallpaper, monitor-res}     -> WallpaperVariant?
```

## Demo

```innate
(@demo){
  @mgr <- @wm/new-manager{schedule: {kind: "interval", interval-ms: 1000},
                            recency-window: 2, seed: 42}

  @wm/add-monitor{manager: @mgr, monitor: {id: 1, resolution: {w: 1920, h: 1080}}}
  @wm/add-monitor{manager: @mgr, monitor: {id: 2, resolution: {w: 2560, h: 1440}}}

  (for @id in [10, 11, 12, 13, 14]{
    @wm/add-wallpaper{manager: @mgr,
                       wallpaper: {id: @id, variants: [{res: {w: 1920, h: 1080}},
                                                         {res: {w: 2560, h: 1440}}]}}
  })

  @wm/tick{manager: @mgr, ms: 1}
  @mgr.assignments[1]   ;; -> {current-wallpaper-id: 13, ...}
  @mgr.assignments[2]   ;; -> {current-wallpaper-id: 13, ...}

  @wm/tick{manager: @mgr, ms: 1100}
  @mgr.assignments[1]   ;; -> {current-wallpaper-id: 10, ...} (13 in recency)
  @mgr.assignments[2]   ;; -> {current-wallpaper-id: 14, ...} (13 in recency)

  @wm/resolution-fit-score{monitor: {w: 1920, h: 1080},
                             variant: {w: 1920, h: 1080}}   ;; -> 1000
  @wm/resolution-fit-score{monitor: {w: 1920, h: 1080},
                             variant: {w: 1280, h: 720}}    ;; -> 500 (same AR, downscale)
  @wm/resolution-fit-score{monitor: {w: 1920, h: 1080},
                             variant: {w: 2000, h: 1500}}   ;; -> 355 (AR mismatch)
}
```

## Where

Assignment MUST be a monitor-indexed map, not a single cursor — N monitors have N independent rotation states. Resolution fit score MUST weight aspect-ratio above pixel count — a 16:9 variant on a 16:9 monitor looks right even if pixel counts differ; a 4:3 variant on 16:9 looks wrong at any size. Pixel proximity MUST be asymmetric — upscaling reveals artifacts; downscaling does not. Recency filter MUST have a fallback — a filter that excludes everything deadlocks the manager; fall back to the full pool. Schedule MUST be a closed ADT — three shapes cover the real cases (interval, fixed-list, tag-window); a fourth would need explicit addition. `tick(ms)` MUST rotate at most once per monitor per tick — multiple missed events collapse; flashing through catch-up rotations is worse UX than one accurate end-state rotation. Time-of-day windows MUST wrap via `elapsed_ms % 24h` — the manager has no calendar, just an elapsed clock; circular time is derived. Determinism MUST hold — same seed + same history + same policy → same pick; tests depend on it and "resume last shuffle" UX depends on it.
