-- Stream Player — segmented video with buffer-based adaptive bitrate.

namespace Graphics.SP

inductive PlayerState where | idle | buffering | playing | paused | ended
  deriving Repr, BEq

structure Segment where
  index : Nat
  durationMs : Nat
  sizesByBitrate : List (Nat × Nat) := []
  deriving Repr

def Segment.sizeFor (s : Segment) (bitrate : Nat) : Option Nat :=
  (s.sizesByBitrate.find? (fun p => p.1 == bitrate)).map Prod.snd

structure Stream where
  bitratesBps : List Nat
  segments : List Segment
  safetyFactor : Float := 0.85
  targetBufferMs : Nat := 6000
  deriving Repr

def Stream.totalDurationMs (s : Stream) : Nat :=
  s.segments.foldl (init := 0) (fun acc seg => acc + seg.durationMs)

def Stream.pickBitrate (s : Stream) (bufferMs bandwidthBps : Nat)
    (segment : Segment) : Nat :=
  let effectiveBw := max 1.0 (bandwidthBps.toFloat * s.safetyFactor)
  let rawBudget := max bufferMs segment.durationMs
  let budgetMs := min rawBudget (s.targetBufferMs * 2)
  let chosen := (s.bitratesBps.head?).getD 0
  s.bitratesBps.foldl (init := chosen) fun acc bitrate =>
    match segment.sizeFor bitrate with
    | none => acc
    | some bytes =>
      let downloadMs := bytes.toFloat * 8.0 / effectiveBw * 1000.0
      if downloadMs ≤ budgetMs.toFloat then bitrate else acc

inductive EventKind where
  | stateChanged | segmentLoaded | bitrateSwitched | rebuffer | seeked
  deriving Repr, BEq

structure PlayerEvent where
  kind : EventKind
  detail : String := ""
  deriving Repr

structure Player where
  state : PlayerState := .idle
  currentSegment : Nat := 0
  bufferMs : Nat := 0
  playedMs : Nat := 0
  bandwidthBps : Nat := 1000000
  currentBitrate : Nat := 0
  events : List PlayerEvent := []
  stream : Option Stream := none
  deriving Repr

def Player.setState (p : Player) (state : PlayerState) : Player :=
  if p.state == state then p
  else { p with state, events := p.events ++ [{ kind := .stateChanged,
    detail := s!"{repr p.state} -> {repr state}" }] }

def Player.attach (p : Player) (stream : Stream) : Player :=
  let p' := { p with stream := some stream,
                      currentBitrate := (stream.bitratesBps.head?).getD 0 }
  p'.setState .buffering

def Player.setBandwidth (p : Player) (bps : Nat) : Player :=
  { p with bandwidthBps := bps }

def Player.loadNextSegment (p : Player) : (Bool × Player) :=
  match p.stream with
  | none => (false, p)
  | some stream =>
    if p.currentSegment ≥ stream.segments.length then (false, p)
    else
      let seg := stream.segments.get! p.currentSegment
      let newBitrate := stream.pickBitrate p.bufferMs p.bandwidthBps seg
      let p1 :=
        if newBitrate == p.currentBitrate then p
        else { p with currentBitrate := newBitrate,
                       events := p.events ++ [{ kind := .bitrateSwitched,
                         detail := s!"{p.currentBitrate} -> {newBitrate}" }] }
      let p2 := { p1 with bufferMs := p1.bufferMs + seg.durationMs,
                          currentSegment := p1.currentSegment + 1 }
      let p3 := { p2 with events := p2.events ++ [{ kind := .segmentLoaded,
                            detail := s!"segment {seg.index} @ {newBitrate}bps" }] }
      let p4 :=
        if p3.state == .buffering ∧ p3.bufferMs ≥ stream.targetBufferMs then
          p3.setState .playing
        else p3
      (true, p4)

def Player.tick (p : Player) (ms : Nat) : Player :=
  if p.state ≠ .playing then p
  else match p.stream with
  | none => p
  | some stream =>
    let drained := min ms p.bufferMs
    let p1 := { p with bufferMs := p.bufferMs - drained,
                        playedMs := p.playedMs + drained }
    let atEnd := p1.currentSegment ≥ stream.segments.length
    if p1.bufferMs == 0 ∧ atEnd then p1.setState .ended
    else if p1.bufferMs == 0 then
      let p2 := { p1 with events := p1.events ++ [{ kind := .rebuffer,
                            detail := s!"buffer empty at {p1.playedMs}ms" }] }
      p2.setState .buffering
    else p1

end Graphics.SP
