-- Media Player Widget — playback FSM + playlist with repeat/shuffle modes.
-- Pure-functional: every operation returns a new Player.

namespace Web.MP

inductive State where
  | stopped | playing | paused
  deriving Repr, DecidableEq, BEq

inductive Repeat where
  | none | one | all
  deriving Repr, DecidableEq, BEq

structure Track where
  id : Nat
  title : String
  artist : String
  durationMs : Nat
  deriving Repr

inductive Event where
  | trackStarted (id : Nat)
  | trackEnded (id : Nat)
  | playlistEnded
  | stateChanged (s : State)
  deriving Repr

structure Player where
  playlist : List Track := []
  state : State := .stopped
  currentIdx : Option Nat := none
  positionMs : Nat := 0
  volume : Float := 1.0
  repeat : Repeat := .none
  shuffle : Bool := false
  deriving Repr

def Player.empty : Player := {}

def Player.loadPlaylist (p : Player) (tracks : List Track) : Player :=
  { p with
    playlist := tracks,
    state := .stopped,
    currentIdx := none,
    positionMs := 0 }

def Player.currentTrack (p : Player) : Option Track :=
  match p.currentIdx with
  | none => none
  | some i => p.playlist.get? i

def Player.setVolume (p : Player) (v : Float) : Player :=
  { p with volume := max 0.0 (min 1.0 v) }

def Player.setRepeat (p : Player) (r : Repeat) : Player := { p with repeat := r }
def Player.setShuffle (p : Player) (on : Bool) : Player := { p with shuffle := on }

def Player.play (p : Player) : Player × List Event :=
  match p.state with
  | .playing => (p, [])
  | .paused => ({ p with state := .playing }, [.stateChanged .playing])
  | .stopped =>
    if p.playlist.isEmpty then (p, [])
    else
      let p' : Player := { p with currentIdx := some 0, positionMs := 0, state := .playing }
      let events :=
        match p'.currentTrack with
        | some t => [.trackStarted t.id, .stateChanged .playing]
        | none => [.stateChanged .playing]
      (p', events)

def Player.pause (p : Player) : Player × List Event :=
  if p.state == .playing then
    ({ p with state := .paused }, [.stateChanged .paused])
  else (p, [])

def Player.stop (p : Player) : Player × List Event :=
  if p.state == .stopped then (p, [])
  else
    ({ p with state := .stopped, currentIdx := none, positionMs := 0 },
     [.stateChanged .stopped])

def Player.nextTrack (p : Player) : Player × List Event :=
  match p.currentIdx with
  | none => p.play
  | some cur =>
    if p.repeat == .one then
      let p' := { p with positionMs := 0 }
      match p'.currentTrack with
      | some t => (p', [.trackStarted t.id])
      | none => (p', [])
    else
      let nxt := cur + 1
      if nxt < p.playlist.length then
        let p' : Player := { p with currentIdx := some nxt, positionMs := 0 }
        match p'.currentTrack with
        | some t => (p', [.trackStarted t.id])
        | none => (p', [])
      else if p.repeat == .all then
        let p' : Player := { p with currentIdx := some 0, positionMs := 0 }
        match p'.currentTrack with
        | some t => (p', [.trackStarted t.id])
        | none => (p', [])
      else
        let p' : Player := { p with state := .stopped, currentIdx := none, positionMs := 0 }
        (p', [.playlistEnded, .stateChanged .stopped])

def Player.previousTrack (p : Player) : Player × List Event :=
  match p.currentIdx with
  | none => (p, [])
  | some 0 =>
    let p' := { p with positionMs := 0 }
    match p'.currentTrack with
    | some t => (p', [.trackStarted t.id])
    | none => (p', [])
  | some (n + 1) =>
    let p' : Player := { p with currentIdx := some n, positionMs := 0 }
    match p'.currentTrack with
    | some t => (p', [.trackStarted t.id])
    | none => (p', [])

def Player.seekMs (p : Player) (pos : Nat) : Player × Bool :=
  match p.currentTrack with
  | none => (p, false)
  | some t => if pos > t.durationMs then (p, false)
              else ({ p with positionMs := pos }, true)

/-- Advance playback by delta ms. Events emitted on track boundaries. -/
partial def Player.tick (p : Player) (deltaMs : Nat) : Player × List Event :=
  if p.state ≠ .playing then (p, [])
  else
    let rec loop (p : Player) (remaining : Nat) (acc : List Event)
        : Player × List Event :=
      if remaining == 0 then (p, acc.reverse)
      else
        match p.currentTrack with
        | none => (p, acc.reverse)
        | some cur =>
          let timeLeft :=
            if cur.durationMs ≥ p.positionMs then cur.durationMs - p.positionMs else 0
          if remaining < timeLeft then
            ({ p with positionMs := p.positionMs + remaining }, acc.reverse)
          else
            let p' := { p with positionMs := cur.durationMs }
            let acc' := .trackEnded cur.id :: acc
            let (p'', evs) := p'.nextTrack
            let acc'' := evs.reverse ++ acc'
            if p''.state == .stopped then (p'', acc''.reverse)
            else loop p'' (remaining - timeLeft) acc''
    loop p deltaMs []

end Web.MP
