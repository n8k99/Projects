-- Screen Saver — idle FSM + bouncing-line physics + dim ramp + activity reset.

namespace Graphics.Saver

inductive SaverState where
  | active | dimming | saver | locked | waking
  deriving Repr, BEq

structure Config where
  idleMs : Nat := 60000
  dimMs : Nat := 3000
  autoLockMs : Nat := 30000
  wakeMs : Nat := 500
  canvasWidth : Nat := 1920
  canvasHeight : Nat := 1080
  deriving Repr

structure Endpoint where
  x : Int
  y : Int
  vx : Int
  vy : Int
  deriving Repr

structure SSRgb where
  r : Nat
  g : Nat
  b : Nat
  deriving Repr

structure BouncingLine where
  p1 : Endpoint
  p2 : Endpoint
  color : SSRgb
  deriving Repr

inductive EventKind where
  | stateChanged | activityReset | lineBounced
  deriving Repr, BEq

structure SaverEvent where
  kind : EventKind
  atMs : Nat
  detail : String := ""
  deriving Repr

structure ScreenSaver where
  config : Config
  state : SaverState := .active
  lines : List BouncingLine := []
  elapsedMs : Nat := 0
  lastActivityMs : Nat := 0
  stateEnteredMs : Nat := 0
  events : List SaverEvent := []
  deriving Repr

def ScreenSaver.new (cfg : Config) : ScreenSaver := { config := cfg }

def ScreenSaver.addLine (s : ScreenSaver) (line : BouncingLine) : ScreenSaver :=
  { s with lines := s.lines ++ [line] }

def ScreenSaver.dimOpacity (s : ScreenSaver) : Nat :=
  if s.state != .dimming then 0
  else
    let since := if s.elapsedMs > s.stateEnteredMs then s.elapsedMs - s.stateEnteredMs else 0
    if s.config.dimMs == 0 then 255
    else (min since s.config.dimMs) * 255 / s.config.dimMs

def transitionTo (s : ScreenSaver) (newState : SaverState) : ScreenSaver :=
  if s.state == newState then s
  else { s with
    state := newState,
    stateEnteredMs := s.elapsedMs,
    events := s.events ++ [{
      kind := .stateChanged, atMs := s.elapsedMs,
      detail := s!"{repr s.state} -> {repr newState}"
    }]
  }

def ScreenSaver.recordActivity (s : ScreenSaver) : ScreenSaver :=
  let s1 := { s with
    lastActivityMs := s.elapsedMs,
    events := s.events ++ [{ kind := .activityReset, atMs := s.elapsedMs }]
  }
  if s1.state == .dimming ∨ s1.state == .saver then transitionTo s1 .waking
  else s1

def ScreenSaver.unlock (s : ScreenSaver) : ScreenSaver :=
  if s.state == .locked then
    let s1 := transitionTo s .waking
    { s1 with lastActivityMs := s1.elapsedMs }
  else s

def reflectEndpoint (events : List SaverEvent) (ep : Endpoint) (idx : Nat)
    (ms cw ch : Int) (atMs : Nat) : Endpoint × List SaverEvent :=
  let newX := ep.x + ep.vx * ms / 100
  let newY := ep.y + ep.vy * ms / 100
  let ep1 := { ep with x := newX, y := newY }
  let (ep2, evs2) :=
    if ep1.x < 0 then
      ({ ep1 with x := -ep1.x, vx := -ep1.vx },
       events ++ [{ kind := .lineBounced, atMs,
                     detail := s!"line {idx} axis x" }])
    else if ep1.x > cw then
      ({ ep1 with x := 2*cw - ep1.x, vx := -ep1.vx },
       events ++ [{ kind := .lineBounced, atMs,
                     detail := s!"line {idx} axis x" }])
    else (ep1, events)
  let (ep3, evs3) :=
    if ep2.y < 0 then
      ({ ep2 with y := -ep2.y, vy := -ep2.vy },
       evs2 ++ [{ kind := .lineBounced, atMs,
                  detail := s!"line {idx} axis y" }])
    else if ep2.y > ch then
      ({ ep2 with y := 2*ch - ep2.y, vy := -ep2.vy },
       evs2 ++ [{ kind := .lineBounced, atMs,
                  detail := s!"line {idx} axis y" }])
    else (ep2, evs2)
  (ep3, evs3)

def integratePhysics (s : ScreenSaver) (ms : Nat) : ScreenSaver :=
  let cw := Int.ofNat (s.config.canvasWidth * 100)
  let ch := Int.ofNat (s.config.canvasHeight * 100)
  let msI := Int.ofNat ms
  let (newLines, newEvents) :=
    s.lines.zipIdx.foldl (init := ([], s.events)) fun (accLines, accEvents) ⟨line, idx⟩ =>
      let (p1', ev1) := reflectEndpoint accEvents line.p1 idx msI cw ch s.elapsedMs
      let (p2', ev2) := reflectEndpoint ev1 line.p2 idx msI cw ch s.elapsedMs
      (accLines ++ [{ line with p1 := p1', p2 := p2' }], ev2)
  { s with lines := newLines, events := newEvents }

def ScreenSaver.tick (s : ScreenSaver) (ms : Nat) : ScreenSaver :=
  let s1 := { s with elapsedMs := s.elapsedMs + ms }
  let sinceState := s1.elapsedMs - s1.stateEnteredMs
  match s1.state with
  | .active =>
    let idle := if s1.elapsedMs > s1.lastActivityMs
                then s1.elapsedMs - s1.lastActivityMs else 0
    if idle ≥ s1.config.idleMs then transitionTo s1 .dimming else s1
  | .dimming =>
    if sinceState ≥ s1.config.dimMs then transitionTo s1 .saver else s1
  | .saver =>
    let s2 := if sinceState ≥ s1.config.autoLockMs
              then transitionTo s1 .locked else s1
    integratePhysics s2 ms
  | .locked => s1
  | .waking =>
    if sinceState ≥ s1.config.wakeMs then
      let s2 := transitionTo s1 .active
      { s2 with lastActivityMs := s2.elapsedMs }
    else s1

end Graphics.Saver
