# G118 — Mp3 Player

> The Rosetta Stone's fifth **Graphics project**. Models the **playlist navigation kernel** every music app ships: a library holds tracks, a playlist holds an ordered list of track IDs, a player holds a **queue order** (shuffle-aware), a **queue position**, and a **position-within-track**. Repeat modes (Off/One/All) alter next-track logic. Shuffle is a **deterministic permutation** driven by a seeded LCG (same constants as G096). A history stack enables "prev track" with a 3-second grace rule: if you're >3s into a track, prev restarts it; otherwise it pops the history.

```yaml
id: G118
title: Mp3 Player
category: graphics
requires: [G096-rpg-stats, G107-budget-tracker, G117-stream-player]
provides: [playlist-navigation, deterministic-shuffle, playback-fsm, position-tracking, history-stack]
```

## Insight: Playlist ≠ Queue Order

A playlist is an ordered list of track IDs — the **authorial intent**. The queue order is what plays next — the **runtime state**. Shuffle splits them: `queue_order` becomes a permutation of the playlist, and `queue_position` indexes into `queue_order`, not the playlist. Unshuffle rebuilds `queue_order` from the playlist and locates the current track's position in that original order.

First Rosetta Stone project where **authorial order and playback order are separate data structures** with explicit reconciliation. G109's TV tracker had seasons→episodes order; G118 adds a *permutation* layer between the spec and the pointer.

## Insight: Shuffle Is a Deterministic Permutation

Same seed → same shuffle. This matters for tests (assert exact order) and for "resume where I left off" UX (shuffle session is stable across app restarts). Fisher-Yates with the G096 LCG constants (multiplier `6364136223846793005`, increment `1442695040888963407`, shift `33`) gives a balanced permutation with no bias toward any original position.

After shuffling, the **current track gets swapped to position 0** so playback continues from the current track rather than jumping to a random one.

First Rosetta Stone project where **a seeded pseudo-random algorithm is used for UX determinism**, not just for statistical modeling. G096 used it for stat generation; G118 uses it for playback order — the seed is the shuffle session identity.

## Insight: Repeat Modes Change Next-Track Logic, Not State

Repeat is a mode (Off/One/All), not a state. `compute_next` branches on it:
- `One` → stay at same position (re-emit track-changed).
- `All` → advance with wrap: `(pos + 1) % n`.
- `Off` → advance if possible, else transition to Stopped.

The same `tick` and `next` code paths use `compute_next`; the mode is the only branch. This keeps the four-state FSM (Idle/Playing/Paused/Stopped) stable.

First Rosetta Stone project where **a mode flag modulates transition logic without adding states**. G117's FSM was pure; G118's FSM is narrower (four states) but has mode-parameterized transitions.

## Insight: Prev Has a 3-Second Rule

Every music app does this: pressing prev within the first ~3 seconds of a track goes to the *previous* track; pressing prev after 3s restarts the *current* track. It's the universal "I meant to go back, no I meant to restart" disambiguator.

```
if position_ms > 3000: restart current
elif history: pop history, jump to that track's position in queue_order
else: restart current (or noop if nothing plays)
```

The history stack is the only reason prev works across shuffled reorderings: it stores track IDs, not positions, so we re-derive the position via `queue_order.index(prev_id)`.

First Rosetta Stone project where **a single button has position-dependent semantics**. G117's scrubber was one action; G118's prev is two actions behind one button, disambiguated by state.

## Insight: Position-in-Track Is a Second Cursor

The player tracks *two* cursors: `queue_position` (which track) and `position_ms` (how far into that track). `tick` drains the second; when it crosses the track's duration, the first advances (via `compute_next`) and the second resets to 0. Both are writable for seek-like operations; both are observable for UI.

First Rosetta Stone project with **two orthogonal cursors advanced by the same tick**. G117 had a single `played_ms` + `current_segment` where segments were fixed-width; G118 has variable-duration tracks, so both cursors are genuinely independent.

## Insight: History Stack Survives Shuffle Toggle

Toggling shuffle rewrites `queue_order` but leaves the history stack alone. Prev after a shuffle-toggle still works: the history holds *track IDs*, so we re-derive positions in the new order. This composes naturally — the data model has one source of truth (track IDs) and one derived lookup (position in current order).

First Rosetta Stone project where **a user action reshapes an index while an orthogonal state (history) remains valid**. The lesson: store identities, derive positions.

## Choreographic Case: Vault Music Library

```innate
(@vault-music-library){
  @library <- @mp3/new-library{}
  @playlist <- @mp3/playlist-from-tags{tag: "chill"}
  @player <- @mp3/new-player{}
  @mp3/load-playlist{player: @player, playlist: @playlist}
  @mp3/play{player: @player}

  @on-playback-tick (@elapsed-ms){
    @mp3/tick{player: @player, ms: @elapsed-ms, library: @library}
    @ui/render-now-playing{track-id: @player.current-track-id,
                            state: @player.state,
                            position-ms: @player.position-ms}
  }

  @on-user-presses-prev{
    @mp3/prev{player: @player, library: @library}
  }

  @on-user-toggles-shuffle{
    @mp3/set-shuffle{player: @player, on: !@player.shuffle}
  }

  @on-user-cycles-repeat{
    @mp3/set-repeat{player: @player, mode: @next-mode{current: @player.repeat}}
  }
}
```

The vault's player shell is a thin wrapper: a tick drains position; a prev click dispatches to the 3s rule; shuffle/repeat toggles modulate next-track logic. The library is a resolver target for track metadata (title, artist, duration).

## Structures

```innate
(defenum repeat-mode OFF | ONE | ALL)
(defenum player-state IDLE | PLAYING | PAUSED | STOPPED)

(defstruct track
  id           : Int
  title        : String
  artist       : String
  duration-ms  : Int)

(defstruct library
  tracks : {Int -> Track})

(defstruct playlist
  id         : Int
  title      : String
  track-ids  : [Int])

(defenum event-kind
  STATE_CHANGED | TRACK_CHANGED | REPEAT_CHANGED | SHUFFLE_CHANGED)

(defstruct play-event
  kind   : EventKind
  detail : String)

(defstruct player
  playlist        : Playlist?
  queue-order     : [Int]
  queue-position  : Int
  position-ms     : Int
  state           : PlayerState
  repeat          : RepeatMode
  shuffle         : Bool
  shuffle-seed    : Int
  history         : [Int]
  events          : [PlayEvent])
```

## Resolver Natives

```innate
@mp3/new-library{}                                -> Library
@mp3/library-add{library, track}                  -> Unit
@mp3/library-get{library, id}                     -> Track?
@mp3/new-player{}                                 -> Player
@mp3/load-playlist{player, playlist}              -> Unit
@mp3/play{player}                                 -> Unit
@mp3/pause{player}                                -> Unit
@mp3/stop{player}                                 -> Unit
@mp3/tick{player, ms, library}                    -> Unit
@mp3/next{player, library}                        -> Bool
@mp3/prev{player, library}                        -> Bool
@mp3/set-repeat{player, mode}                     -> Unit
@mp3/set-shuffle{player, on}                      -> Unit
@mp3/current-track-id{player}                     -> Int?
```

## Demo

```innate
(@demo){
  @lib <- @mp3/new-library{}
  @tracks <- [{id: 1, title: "Alpha", duration-ms: 120000},
              {id: 2, title: "Beta",  duration-ms: 180000},
              {id: 3, title: "Gamma", duration-ms: 150000},
              {id: 4, title: "Delta", duration-ms: 200000}]
  (for @t in @tracks @mp3/library-add{library: @lib, track: @t})

  @p <- @mp3/new-player{}
  @mp3/load-playlist{player: @p,
                      playlist: {id: 1, title: "Mix", track-ids: [1, 2, 3, 4]}}
  @mp3/play{player: @p}

  @mp3/current-track-id{player: @p}  ;; -> 1
  @mp3/tick{player: @p, ms: 125000, library: @lib}
  @mp3/current-track-id{player: @p}  ;; -> 2 (advanced after 120000ms)

  @mp3/set-repeat{player: @p, mode: ALL}
  (@advance-to-last @p)
  @mp3/tick{player: @p, ms: 210000, library: @lib}
  @mp3/current-track-id{player: @p}  ;; -> 1 (wrapped via repeat-all)

  @mp3/set-shuffle{player: @p, on: true}
  @p.queue-order  ;; -> [1, 4, 3, 2] (deterministic by seed=42; cur=1 held at pos 0)
}
```

## Where

Playlist and queue order MUST be separate — authorial intent is not the same as playback order, and shuffle is the canonical case where they diverge. Shuffle MUST be seeded — deterministic behavior is required for tests and "resume last shuffle" UX. Current track MUST survive a shuffle — swap it to position 0 after the permutation so playback doesn't jump. Repeat MUST be a mode, not a state — it modulates transitions but the player FSM is still just Idle/Playing/Paused/Stopped. Prev MUST implement the 3-second rule — universal UX convention; violating it feels broken. History MUST store track IDs (not positions) — positions are invalidated by shuffle; IDs are stable. Position-in-track MUST be independent of queue position — tick advances both but at different rates and with different reset conditions. Events MUST be emitted per state, track, repeat, and shuffle change — behavioural assertions need more than final state.
