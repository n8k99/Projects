-- Mp3 Player — playlist navigation with shuffle/repeat + position tracking.

namespace Graphics.Mp3

structure Track where
  id : Nat
  title : String
  artist : String := ""
  durationMs : Nat
  deriving Repr

structure Library where
  tracks : List Track := []
  deriving Repr

def Library.add (l : Library) (t : Track) : Library := { l with tracks := l.tracks ++ [t] }
def Library.get (l : Library) (id : Nat) : Option Track := l.tracks.find? (fun t => t.id == id)

structure Playlist where
  id : Nat
  title : String
  trackIds : List Nat := []
  deriving Repr

inductive RepeatMode where | off | one | all
  deriving Repr, BEq

inductive PlayerState where | idle | playing | paused | stopped
  deriving Repr, BEq

inductive EventKind where
  | stateChanged | trackChanged | repeatChanged | shuffleChanged
  deriving Repr, BEq

structure PlayEvent where
  kind : EventKind
  detail : String := ""
  deriving Repr

structure Player where
  playlist : Option Playlist := none
  queueOrder : List Nat := []
  queuePosition : Nat := 0
  positionMs : Nat := 0
  state : PlayerState := .idle
  repeat' : RepeatMode := .off
  shuffle : Bool := false
  shuffleSeed : UInt64 := 42
  history : List Nat := []
  events : List PlayEvent := []
  deriving Repr

def Player.currentTrackId (p : Player) : Option Nat :=
  p.queueOrder.get? p.queuePosition

def Player.setState (p : Player) (s : PlayerState) : Player :=
  if p.state == s then p
  else { p with state := s, events := p.events ++ [{ kind := .stateChanged,
    detail := s!"{repr p.state} -> {repr s}" }] }

def Player.emitTrackChanged (p : Player) : Player :=
  match p.currentTrackId with
  | none => p
  | some id => { p with events := p.events ++ [{ kind := .trackChanged,
      detail := s!"track {id}" }] }

-- Fisher-Yates shuffle via LCG (same constants as G096).
partial def swapAt (xs : List Nat) (i j : Nat) : List Nat :=
  let arr := xs.toArray
  (arr.swap! i j).toList

def lcgStep (state : UInt64) : UInt64 :=
  state * 6364136223846793005 + 1442695040888963407

def fisherYates (xs : List Nat) (seed : UInt64) : List Nat := Id.run do
  let mut arr := xs.toArray
  let n := arr.size
  let mut state := seed + 1
  let mut i := n - 1
  while i > 0 do
    state := lcgStep state
    let r : UInt32 := (state >>> 33).toUInt32
    let j : Nat := r.toNat % (i + 1)
    arr := arr.swap! i j
    i := i - 1
  pure arr.toList

def Player.applyShuffle (p : Player) : Player :=
  match p.playlist with
  | none => p
  | some pl =>
    let shuffled := fisherYates pl.trackIds p.shuffleSeed
    match p.currentTrackId with
    | none => { p with queueOrder := shuffled }
    | some curId =>
      match shuffled.idxOf? curId with
      | none => { p with queueOrder := shuffled }
      | some pos =>
        let arr := shuffled.toArray
        let rotated := (arr.swap! 0 pos).toList
        { p with queueOrder := rotated, queuePosition := 0 }

def Player.unshuffle (p : Player) : Player :=
  match p.playlist with
  | none => p
  | some pl =>
    let curId := p.currentTrackId
    let order := pl.trackIds
    match curId with
    | none => { p with queueOrder := order }
    | some id =>
      match order.idxOf? id with
      | none => { p with queueOrder := order }
      | some pos => { p with queueOrder := order, queuePosition := pos }

def Player.loadPlaylist (p : Player) (pl : Playlist) : Player :=
  let p1 := { p with playlist := some pl, queueOrder := pl.trackIds,
                     queuePosition := 0, positionMs := 0 }
  let p2 := if p1.shuffle then p1.applyShuffle else p1
  p2.setState .stopped

def Player.play (p : Player) : Player :=
  if p.queueOrder.isEmpty then p else p.setState .playing

def Player.pause (p : Player) : Player :=
  if p.state == .playing then p.setState .paused else p

def Player.stop (p : Player) : Player :=
  { p.setState .stopped with positionMs := 0 }

-- advance/repeat/end encoded as a result struct
structure NextAction where
  kind : String  -- "advance" | "repeat" | "end"
  pos : Nat := 0

def Player.computeNext (p : Player) : NextAction :=
  let n := p.queueOrder.length
  match p.repeat' with
  | .one => { kind := "repeat" }
  | .all => { kind := "advance", pos := (p.queuePosition + 1) % n }
  | .off =>
    if p.queuePosition + 1 < n then { kind := "advance", pos := p.queuePosition + 1 }
    else { kind := "end" }

def Player.tick (p : Player) (ms : Nat) (lib : Library) : Player :=
  if p.state != .playing ∨ p.queueOrder.isEmpty then p
  else match p.currentTrackId with
  | none => p
  | some curId =>
    match lib.get curId with
    | none => p
    | some tr =>
      let remaining := tr.durationMs - p.positionMs
      if ms ≥ remaining then
        let p1 := { p with history := p.history ++ [curId], positionMs := 0 }
        let action := p1.computeNext
        match action.kind with
        | "advance" => { p1 with queuePosition := action.pos }.emitTrackChanged
        | "repeat" => p1.emitTrackChanged
        | _ => p1.setState .stopped
      else { p with positionMs := p.positionMs + ms }

def Player.next (p : Player) (_lib : Library) : (Bool × Player) :=
  match p.currentTrackId with
  | none => (false, p)
  | some curId =>
    let p1 := { p with history := p.history ++ [curId], positionMs := 0 }
    let action := p1.computeNext
    match action.kind with
    | "advance" => (true, { p1 with queuePosition := action.pos }.emitTrackChanged)
    | "repeat" => (true, p1.emitTrackChanged)
    | _ => (false, p1.setState .stopped)

def Player.prev (p : Player) : (Bool × Player) :=
  if p.positionMs > 3000 then (true, { p with positionMs := 0 })
  else match p.history.reverse with
  | [] => (false, { p with positionMs := 0 })
  | prevId :: rest =>
    let newHistory := rest.reverse
    match p.queueOrder.idxOf? prevId with
    | none => (false, { p with positionMs := 0 })
    | some pos =>
      let p1 := { p with history := newHistory, queuePosition := pos, positionMs := 0 }
      (true, p1.emitTrackChanged)

def Player.setRepeat (p : Player) (mode : RepeatMode) : Player :=
  if p.repeat' == mode then p
  else { p with repeat' := mode, events := p.events ++ [{ kind := .repeatChanged,
    detail := s!"{repr mode}" }] }

def Player.setShuffle (p : Player) (on : Bool) : Player :=
  if p.shuffle == on then p
  else
    let p1 := { p with shuffle := on, events := p.events ++ [{ kind := .shuffleChanged,
      detail := if on then "on" else "off" }] }
    if on then p1.applyShuffle else p1.unshuffle

end Graphics.Mp3
