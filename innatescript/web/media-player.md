# G078 — Media Player Widget

> The Rosetta Stone's first **three-state FSM with playlist modes**. Playback is Stopped ↔ Playing ↔ Paused; transport actions (`play`, `pause`, `stop`, `next`, `previous`) are **context-dependent** — what they do depends on the current state. First project where the tick-function is itself the engine: calling `tick(delta_ms)` advances playback, emits boundary events, and performs automatic transitions (track end → next track → playlist end → stop).

```yaml
id: G078
title: Media Player Widget
category: web
requires: [G062-vending-machine, G065-progress-bar, G077-password-safe]
provides: [three-state-playback-fsm, context-dependent-transport, tick-driven-playback, playlist-modes, event-emitting-engine]
```

## Insight: Three-State Playback Is the Minimum for Pausable Playback

G062's vending machine had two states (Idle, Accepting). G073's telnet had three (Unauthenticated, Authenticated, Disconnected — with Disconnected terminal). G077's password safe had two (Locked, Unlocked). G078 has three **non-terminal** states: Stopped, Playing, Paused — and every state can transition to every other state, so the FSM has a full 3×3 transition table.

The minimum for pausable playback is three states because:
- **Stopped** = nothing loaded-and-ready-to-resume. No current track, no position.
- **Playing** = time advances when `tick` fires.
- **Paused** = loaded and positioned, but `tick` does nothing. Resume is cheap.

Skip Paused and you get "Stopped" semantics every time the user lifts their finger off Play — everything has to re-enter from track 0. Merge Stopped into Paused and you lose the distinction between "nothing happening, start from the beginning" and "pick up where I left off." The three-state model is not over-engineered; it's the point where the abstraction fits user intuition.

First Rosetta Stone project where a three-state FSM is **load-bearing**, not incidental. The noosphere's choreography engine will likely use this exact pattern: not-started (stopped), in-progress (playing), paused-for-user-input (paused) — mapping `tick` to "process one step" and `pause` to "wait for user."

## Insight: Transport Actions Are Context-Dependent

`play()` has **different behaviour in each state**:
- **Stopped**: start track 0, emit TrackStarted(0) + StateChanged(Playing).
- **Paused**: resume from current position, emit StateChanged(Playing) only.
- **Playing**: no-op.

This context-dependency is the hallmark of a proper FSM. The same API call produces different effects; the caller doesn't need to query the state first. The handler encapsulates the policy.

Compare with a naive design that has separate `start()` / `resume()` / `continue()` functions — the caller must check state before calling. G078's `play()` absorbs the dispatch: "I want to play, do whatever that means right now." The FSM provides the context.

First Rosetta Stone project with **context-dependent operations on a running state machine**. G062 had state-gated operations (refused if wrong state). G078 has state-interpreted operations (works in every state, does the right thing).

## Insight: The Tick Function Is the Engine

`tick(delta_ms)` is the heart of the player. It advances `position_ms`, checks if the current track has ended, fires track-boundary events, advances to the next track (consulting repeat/shuffle), and loops until `delta_ms` is consumed or the playlist runs out.

A 10-second `tick` on a playlist of 1-second tracks produces 10 boundary events and ends at track N. A 500-ms `tick` on a 10-second current track produces no events and advances position. A `tick` on a paused or stopped player is a no-op.

This is the **game-loop pattern**: the world has a `tick` function that takes a time delta and advances everything. Every simulation, every physics engine, every game, every scheduled-task executor has this shape. The caller decides when to tick; the engine decides what ticking means.

First Rosetta Stone project with **tick-driven state advancement**. G062's vending machine had instantaneous transitions; G065 progress bar had time as elapsed-since-start. G078 is the first where time is **consumed** by a function that turns "time passed" into "state changed and events fired."

## Insight: Playlist Modes (Repeat + Shuffle) Are Orthogonal

Four combinations: repeat=(None|One|All) × shuffle=(off|on). Every combination is a valid play mode:
- repeat=None + shuffle=off: standard linear playback, stop at end.
- repeat=One + shuffle=off: loop current track forever.
- repeat=All + shuffle=off: loop entire playlist linearly.
- repeat=One + shuffle=on: loop the (randomly-selected) current track.
- repeat=All + shuffle=on: shuffle the playlist on each lap, play through, repeat with fresh shuffle.

The implementation: `next_track()` consults both modes. Shuffle changes *which* track is next; repeat changes *whether to stop or continue* at the end. They don't interact; they're independent axes.

First Rosetta Stone project with **orthogonal mode dimensions** on the same operation. Previous projects had single-mode operations (G073 login is always login). G078 shows that mode axes compose: the product of all combinations is the set of supported behaviours.

## Insight: Events Fire on Boundaries, Not Every Tick

Every time a track ends, `TrackEnded(id)` fires. Every time a new track starts (after a track ends, after `next_track()`, after `play()` from Stopped), `TrackStarted(id)` fires. When the playlist ends (repeat=None, past last track), `PlaylistEnded` fires. When the state changes, `StateChanged(new)` fires.

**Events are boundary signals**, not continuous streams. A UI subscribed to events updates on track changes; it doesn't update on every position-tick. This separates "things worth reacting to" from "things worth polling." The caller polls `position_ms` for the scrubber; the caller subscribes to events for "now playing" display.

First Rosetta Stone project with **event stream as a supplement to state query**. G065 had state queries only; G066 had outcomes returned from the run. G078 has both: state queries (`position_ms`, `current_track`) for continuous display, and events for boundary-triggered reactions.

## Insight: Repeat-One Uses Insertion Position, Not Playlist Position

When `repeat = One` and a track ends, we restart the **current index**, not "track id 0" or "first-in-shuffle." This matters because:
- If shuffle is on and current is shuffle-index 3 (playlist-index 7), repeat-one restarts shuffle-index 3, playing playlist-track 7 again.
- If shuffle were off, same behaviour: current playlist-index restarts.

This is the interaction between the two modes. The implementation is simple — "current_index" is the pointer, and what-it-points-at depends on whether shuffle is on — but the logic has to be consistent. Test: with shuffle on and repeat=One, the same (shuffled-first) track plays forever, not "track ID 0."

First Rosetta Stone case where **a pointer abstraction works the same way under different modes**. Index into shuffle_order vs index into playlist — the arithmetic is the same, the dereference differs.

## Choreographic Case: Vault-Scoped Background Music Channel

```innate
(@music-channel){
  @playlist <- @vault/find-notes{tag: "track"}.map(@render-as-track)
  @player <- @media/new-player
  @media/load-playlist{player: @player, tracks: @playlist}
  @media/set-repeat{player: @player, mode: "all"}
  @media/set-shuffle{player: @player, on: true}
  @media/play{player: @player}

  @every 100ms {
    @events <- @media/tick{player: @player, delta_ms: 100}
    @for ev in @events {
      when (@ev.kind == "track_started"){
        @ui/update-now-playing{track_id: @ev.track_id}
        @vault/log{msg: "now playing", track: @ev.track_id}
      }
    }
  }

  @on-skip-command {
    @media/next-track{player: @player}
  }
}
```

Background music as a choreography: the player advances by tick, events drive UI updates and logging, user commands go through the transport API. Production music apps look exactly like this once you peel back the UI and audio-engine layers.

## Structures

```innate
(defenum state Stopped | Playing | Paused)
(defenum repeat None | One | All)

(defstruct track
  id          : Int
  title       : String
  artist      : String
  duration-ms : Int)

(defstruct event
  kind     : "track_started" | "track_ended" | "playlist_ended" | "state_changed"
  track-id : Int?
  state    : State?)

(defstruct player
  playlist      : [Track]
  state         : State
  current-idx   : Int?
  position-ms   : Int
  volume        : Float            ;; clamped to [0, 1]
  repeat        : Repeat
  shuffle       : Bool
  shuffle-order : [Int])            ;; permutation when shuffle on
```

## Resolver Natives

```innate
@media/new-player                                    -> Player
@media/load-playlist{player, tracks}                 -> Unit
@media/play{player}                                  -> [Event]   ;; context-dependent
@media/pause{player}                                 -> [Event]
@media/stop{player}                                  -> [Event]
@media/next-track{player}                            -> [Event]
@media/previous-track{player}                        -> [Event]
@media/seek-ms{player, pos}                          -> Bool
@media/set-volume{player, v}                         -> Unit      ;; clamps to [0,1]
@media/set-repeat{player, mode}                      -> Unit
@media/set-shuffle{player, on}                       -> Unit
@media/tick{player, delta_ms}                        -> [Event]   ;; the engine
@media/state{player}                                 -> State
@media/position-ms{player}                           -> Int
@media/current-track{player}                         -> Track?
```

## Demo

```innate
(@demo){
  @p <- @media/new-player
  @media/load-playlist{player: @p,
                        tracks: [
                          {id: 1, title: "Intro",   duration_ms: 30000},
                          {id: 2, title: "Track A", duration_ms: 180000},
                          {id: 3, title: "Outro",   duration_ms: 60000}]}
  @media/play{player: @p}                       ;; -> TrackStarted(1), StateChanged(Playing)
  @media/tick{player: @p, delta_ms: 240000}     ;; -> TrackEnded(1), TrackStarted(2), TrackEnded(2), TrackStarted(3)
  @media/current-track{player: @p}              ;; -> {id: 3, title: "Outro"}
  @media/position-ms{player: @p}                ;; -> 30000 (30s into Outro)

  @media/set-repeat{player: @p, mode: "one"}
  @media/tick{player: @p, delta_ms: 600000}     ;; -> track 3 loops ~10× (60s each)
  @media/current-track{player: @p}              ;; -> still track 3

  @media/stop{player: @p}                       ;; -> StateChanged(Stopped)
  @media/state{player: @p}                      ;; -> Stopped
}
```

## Where

`play()` MUST be context-dependent — from Stopped it starts, from Paused it resumes, from Playing it is a no-op; forcing callers to check state before calling is an anti-pattern. `tick()` MUST NOT advance time when the player is not in Playing state — a paused player with an active tick caller must stay at the same position. Tick MUST emit TrackEnded before TrackStarted on a boundary crossing — consumers expect "this just ended" before "that just started." `previous()` at track-0 MUST restart the current track at position 0, not go to the end of the previous playlist (which doesn't exist); this matches every real music player's behaviour. Repeat=One MUST restart the current track, not advance and then return — the transition should be observable as position=0 without any track change. Shuffle toggling MUST preserve the currently-playing track identity — flipping shuffle on should not jump to a different track mid-play; the player re-indexes to find the same track in the new order.
