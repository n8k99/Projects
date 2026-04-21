-- Traffic Light — coordinated super-state FSM + preemption + adaptive timing.

namespace Graphics.TrafficLight

inductive Phase where
  | nsGreen | nsYellow | allRed | ewGreen | ewYellow | emergencyAllRed
  deriving Repr, BEq

inductive LightColor where | red | yellow | green
  deriving Repr, BEq

inductive PedSignal where | dontWalk | walk | flashing
  deriving Repr, BEq

inductive Direction where | ns | ew
  deriving Repr, BEq

def Phase.nsLight : Phase → LightColor
  | .nsGreen => .green
  | .nsYellow => .yellow
  | _ => .red

def Phase.ewLight : Phase → LightColor
  | .ewGreen => .green
  | .ewYellow => .yellow
  | _ => .red

structure Config where
  nsGreenMs : Nat := 20000
  nsYellowMs : Nat := 3000
  ewGreenMs : Nat := 20000
  ewYellowMs : Nat := 3000
  allRedMs : Nat := 2000
  maxGreenExtensionMs : Nat := 10000
  pedWalkMs : Nat := 8000
  pedFlashingMs : Nat := 5000
  deriving Repr

inductive EventKind where
  | phaseChanged | greenExtended | pedRequested | pedWalkStarted
  | pedWalkEnded | emergencyStarted | emergencyEnded
  deriving Repr, BEq

structure IntersectionEvent where
  kind : EventKind
  atMs : Nat
  detail : String := ""
  deriving Repr

inductive PedState where | inactive | walk | flashing
  deriving Repr, BEq

structure Intersection where
  phase : Phase := .nsGreen
  phaseElapsedMs : Nat := 0
  phaseDurationMs : Nat := 0
  cycleSide : Direction := .ew
  config : Config
  nsQueue : Nat := 0
  ewQueue : Nat := 0
  pedRequested : Bool := false
  pedState : PedState := .inactive
  pedElapsedMs : Nat := 0
  savedPhase : Option (Phase × Nat × Direction) := none
  totalElapsedMs : Nat := 0
  events : List IntersectionEvent := []
  deriving Repr

def durationFor (cfg : Config) : Phase → Nat
  | .nsGreen => cfg.nsGreenMs
  | .nsYellow => cfg.nsYellowMs
  | .ewGreen => cfg.ewGreenMs
  | .ewYellow => cfg.ewYellowMs
  | .allRed => cfg.allRedMs
  | .emergencyAllRed => 10 ^ 18

def Intersection.new (cfg : Config) : Intersection :=
  { config := cfg, phaseDurationMs := durationFor cfg .nsGreen }

def Intersection.nsLight (i : Intersection) : LightColor := i.phase.nsLight
def Intersection.ewLight (i : Intersection) : LightColor := i.phase.ewLight

def Intersection.pedSignal (i : Intersection) : PedSignal :=
  match i.pedState with
  | .inactive => .dontWalk
  | .walk => .walk
  | .flashing => .flashing

def Intersection.setNsQueue (i : Intersection) (n : Nat) : Intersection := { i with nsQueue := n }
def Intersection.setEwQueue (i : Intersection) (n : Nat) : Intersection := { i with ewQueue := n }

def Intersection.requestPedCrossing (i : Intersection) : Intersection :=
  if i.pedState != .inactive then i
  else if i.pedRequested then i
  else { i with
    pedRequested := true,
    events := i.events ++ [{ kind := .pedRequested, atMs := i.totalElapsedMs }]
  }

def Intersection.beginEmergency (i : Intersection) : Intersection :=
  if i.phase == .emergencyAllRed then i
  else
    let at := i.totalElapsedMs
    let from := i.phase
    { i with
      savedPhase := some (i.phase, i.phaseElapsedMs, i.cycleSide),
      phase := .emergencyAllRed,
      phaseElapsedMs := 0,
      phaseDurationMs := 10 ^ 18,
      pedState := .inactive,
      events := i.events ++ [
        { kind := .emergencyStarted, atMs := at },
        { kind := .phaseChanged, atMs := at,
          detail := s!"{repr from} -> emergencyAllRed" }
      ]
    }

def Intersection.endEmergency (i : Intersection) : Intersection :=
  if i.phase != .emergencyAllRed then i
  else
    let at := i.totalElapsedMs
    let from := i.phase
    match i.savedPhase with
    | some (p, elapsed, side) =>
      { i with
        phase := p, phaseElapsedMs := elapsed,
        phaseDurationMs := durationFor i.config p,
        cycleSide := side, savedPhase := none,
        events := i.events ++ [
          { kind := .emergencyEnded, atMs := at },
          { kind := .phaseChanged, atMs := at,
            detail := s!"{repr from} -> {repr p}" }
        ]
      }
    | none =>
      { i with
        phase := .allRed, phaseElapsedMs := 0,
        phaseDurationMs := i.config.allRedMs,
        events := i.events ++ [
          { kind := .emergencyEnded, atMs := at },
          { kind := .phaseChanged, atMs := at,
            detail := s!"{repr from} -> allRed" }
        ]
      }

def considerExtension (i : Intersection) : Intersection :=
  let direction : Option Direction :=
    match i.phase with
    | .nsGreen => some .ns
    | .ewGreen => some .ew
    | _ => none
  match direction with
  | none => i
  | some dir =>
    let queue := match dir with | .ns => i.nsQueue | .ew => i.ewQueue
    if queue == 0 then i
    else
      let base := match dir with | .ns => i.config.nsGreenMs | .ew => i.config.ewGreenMs
      let already := if i.phaseDurationMs > base then i.phaseDurationMs - base else 0
      if already ≥ i.config.maxGreenExtensionMs then i
      else
        let remaining := i.config.maxGreenExtensionMs - already
        let grant := min (queue * 2000) remaining
        if grant == 0 then i
        else
          let dirStr := match dir with | .ns => "ns" | .ew => "ew"
          { i with
            phaseDurationMs := i.phaseDurationMs + grant,
            events := i.events ++ [{
              kind := .greenExtended, atMs := i.totalElapsedMs,
              detail := s!"{dirStr} +{grant}ms"
            }]
          }

def nextPhase (i : Intersection) : Phase × Direction :=
  match i.phase with
  | .nsGreen => (.nsYellow, i.cycleSide)
  | .nsYellow => (.allRed, .ew)
  | .allRed =>
    match i.cycleSide with
    | .ew => (.ewGreen, .ns)
    | .ns => (.nsGreen, .ew)
  | .ewGreen => (.ewYellow, i.cycleSide)
  | .ewYellow => (.allRed, .ns)
  | .emergencyAllRed => (.emergencyAllRed, i.cycleSide)

def transitionPhase (i : Intersection) : Intersection × Bool :=
  let (newPhase, newSide) := nextPhase i
  let from := i.phase
  let i1 := { i with
    phase := newPhase, cycleSide := newSide,
    phaseElapsedMs := 0,
    phaseDurationMs := durationFor i.config newPhase,
    events := i.events ++ [{
      kind := .phaseChanged, atMs := i.totalElapsedMs,
      detail := s!"{repr from} -> {repr newPhase}"
    }]
  }
  if newPhase == .allRed ∧ i1.pedRequested then
    ({ i1 with
       pedRequested := false,
       pedState := .walk,
       pedElapsedMs := 0,
       events := i1.events ++ [{
         kind := .pedWalkStarted, atMs := i1.totalElapsedMs
       }]
    }, true)
  else (i1, false)

def advancePedestrian (i : Intersection) (ms : Nat) : Intersection :=
  if i.pedState == .inactive then i
  else
    let newElapsed := i.pedElapsedMs + ms
    match i.pedState with
    | .walk =>
      if newElapsed ≥ i.config.pedWalkMs then
        { i with pedState := .flashing, pedElapsedMs := 0 }
      else { i with pedElapsedMs := newElapsed }
    | .flashing =>
      if newElapsed ≥ i.config.pedFlashingMs then
        { i with
          pedState := .inactive, pedElapsedMs := 0,
          events := i.events ++ [{ kind := .pedWalkEnded, atMs := i.totalElapsedMs }]
        }
      else { i with pedElapsedMs := newElapsed }
    | _ => i

partial def tickLoop (i : Intersection) (pedJustStartedRemaining : Option Nat)
    : Intersection × Option Nat :=
  if i.phaseElapsedMs ≥ i.phaseDurationMs ∧ i.phase != .emergencyAllRed then
    let overflow := i.phaseElapsedMs - i.phaseDurationMs
    let (i1, started) := transitionPhase i
    let newElapsed := min overflow i1.phaseDurationMs
    let i2 := { i1 with phaseElapsedMs := newElapsed }
    let pedRem := if started then some overflow else pedJustStartedRemaining
    tickLoop i2 pedRem
  else (i, pedJustStartedRemaining)

def Intersection.tick (i : Intersection) (ms : Nat) : Intersection :=
  let i0 := { i with totalElapsedMs := i.totalElapsedMs + ms }
  if i0.phase == .emergencyAllRed then i0
  else
    let i1 := { i0 with phaseElapsedMs := i0.phaseElapsedMs + ms }
    let i2 := if (i1.phase == .nsGreen ∨ i1.phase == .ewGreen)
                ∧ i1.phaseElapsedMs ≥ i1.phaseDurationMs
              then considerExtension i1 else i1
    let (i3, pedStarted?) := tickLoop i2 none
    let pedAdvance :=
      if i3.pedState == .inactive then 0
      else match pedStarted? with
           | some r => r
           | none => ms
    advancePedestrian i3 pedAdvance

end Graphics.TrafficLight
