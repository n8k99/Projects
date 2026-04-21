-- Wallpaper Manager — multi-monitor rotation + resolution best-fit + recency.

namespace Graphics.Wallpaper

structure Resolution where
  width : Nat
  height : Nat
  deriving Repr, BEq

def Resolution.aspectRatio (r : Resolution) : Float :=
  r.width.toFloat / r.height.toFloat

def Resolution.pixelCount (r : Resolution) : Nat := r.width * r.height

structure Monitor where
  id : Nat
  resolution : Resolution
  deriving Repr

structure WallpaperVariant where
  resolution : Resolution
  path : String
  deriving Repr

structure Wallpaper where
  id : Nat
  name : String
  tags : List String := []
  variants : List WallpaperVariant := []
  deriving Repr

def resolutionFitScore (monitor variant : Resolution) : Int :=
  if monitor == variant then 1000
  else
    let mAr := monitor.aspectRatio
    let vAr := variant.aspectRatio
    let arDiff := (mAr - vAr).abs.min 0.5
    let arScore := ((0.5 - arDiff) / 0.5 * 500.0).toInt64.toInt
    let mPx := monitor.pixelCount.toFloat
    let vPx := variant.pixelCount.toFloat
    let ratio := vPx / mPx
    let pxScore : Int :=
      if ratio ≥ 1.0 then
        let lg := (Float.log2 ratio).max 1.0 |>.min 2.0 |>.max 1.0
        (300.0 / lg).toInt64.toInt
      else
        (((ratio - 0.5).max 0.0) * 600.0).toInt64.toInt
    arScore + pxScore

def bestVariantForMonitor (w : Wallpaper) (monitor : Resolution)
    : Option WallpaperVariant :=
  match w.variants with
  | [] => none
  | v :: rest =>
    let scored := rest.foldl (init := (v, resolutionFitScore monitor v.resolution))
      fun (bestV, bestScore) cur =>
        let s := resolutionFitScore monitor cur.resolution
        if s > bestScore then (cur, s) else (bestV, bestScore)
    some scored.1

inductive Schedule where
  | interval (ms : Nat)
  | fixedList (times : List Nat)
  | tagWindow (tag : String) (startMs endMs innerMs : Nat)
  deriving Repr

inductive EventKind where
  | rotated | skippedNoEligible
  deriving Repr, BEq

structure ManagerEvent where
  kind : EventKind
  monitorId : Nat
  atMs : Nat
  wallpaperId : Option Nat := none
  deriving Repr

structure MonitorState where
  currentWallpaperId : Option Nat := none
  lastRotationMs : Nat := 0
  history : List Nat := []
  deriving Repr

def lcgMul : UInt64 := 6364136223846793005
def lcgInc : UInt64 := 1442695040888963407

structure Manager where
  schedule : Schedule
  recencyWindow : Nat
  lcgState : UInt64
  monitors : List Monitor := []
  wallpapers : List Wallpaper := []
  assignments : List (Nat × MonitorState) := []
  elapsedMs : Nat := 0
  events : List ManagerEvent := []
  deriving Repr

def Manager.new (schedule : Schedule) (recency : Nat) (seed : UInt64) : Manager :=
  { schedule, recencyWindow := recency, lcgState := seed + 1 }

def Manager.addMonitor (m : Manager) (mon : Monitor) : Manager :=
  { m with
    monitors := m.monitors ++ [mon],
    assignments := m.assignments ++ [(mon.id, {})] }

def Manager.addWallpaper (m : Manager) (w : Wallpaper) : Manager :=
  { m with wallpapers := m.wallpapers ++ [w] }

def Manager.stateFor (m : Manager) (monitorId : Nat) : Option MonitorState :=
  (m.assignments.find? (fun p => p.1 == monitorId)).map Prod.snd

def updateAssignments (assigns : List (Nat × MonitorState)) (monId : Nat)
    (f : MonitorState → MonitorState) : List (Nat × MonitorState) :=
  assigns.map fun (id, s) => if id == monId then (id, f s) else (id, s)

def Manager.lcgNext (m : Manager) : Manager × UInt64 :=
  let newState := m.lcgState * lcgMul + lcgInc
  ({ m with lcgState := newState }, newState)

def matchesTag (w : Wallpaper) (tag : Option String) : Bool :=
  match tag with
  | none => true
  | some t => w.tags.contains t

def Manager.pickNextForMonitor (m : Manager) (monitorId : Nat)
    (tagFilter : Option String) : Manager × Option Nat :=
  match m.stateFor monitorId with
  | none => (m, none)
  | some state =>
    let recent := (state.history.reverse).take m.recencyWindow
    let eligible := m.wallpapers.filter (fun w =>
      !(recent.contains w.id) ∧ matchesTag w tagFilter)
    let candidates :=
      if eligible.isEmpty then m.wallpapers.filter (fun w => matchesTag w tagFilter)
      else eligible
    match candidates with
    | [] => (m, none)
    | _ =>
      let (m', r) := m.lcgNext
      let idx := r.toNat % candidates.length
      (m', some (candidates.get! idx).id)

def Manager.rotateMonitor (m : Manager) (monitorId : Nat)
    (tagFilter : Option String) : Manager :=
  let (m1, pick?) := m.pickNextForMonitor monitorId tagFilter
  match pick? with
  | none =>
    { m1 with events := m1.events ++ [
      { kind := .skippedNoEligible, monitorId, atMs := m1.elapsedMs }]}
  | some pick =>
    let atMs := m1.elapsedMs
    let newAssigns := updateAssignments m1.assignments monitorId fun s =>
      { s with
        currentWallpaperId := some pick,
        lastRotationMs := atMs,
        history := s.history ++ [pick] }
    { m1 with
      assignments := newAssigns,
      events := m1.events ++ [
        { kind := .rotated, monitorId, atMs, wallpaperId := some pick }]}

def Manager.shouldRotate (m : Manager) (monitorId : Nat) : Bool :=
  match m.stateFor monitorId with
  | none => false
  | some state =>
    match m.schedule with
    | .interval ms =>
      state.currentWallpaperId.isNone ∨ m.elapsedMs - state.lastRotationMs ≥ ms
    | .fixedList times =>
      times.any (fun t => state.lastRotationMs < t ∧ t ≤ m.elapsedMs)
    | .tagWindow _ _ _ innerMs =>
      state.currentWallpaperId.isNone ∨ m.elapsedMs - state.lastRotationMs ≥ innerMs

def Manager.activeTag (m : Manager) : Option String :=
  match m.schedule with
  | .tagWindow tag startMs endMs _ =>
    let dayMs := 24 * 60 * 60 * 1000
    let inDay := m.elapsedMs % dayMs
    if startMs ≤ inDay ∧ inDay < endMs then some tag else none
  | _ => none

def Manager.tick (m : Manager) (ms : Nat) : Manager :=
  let m1 := { m with elapsedMs := m.elapsedMs + ms }
  m1.monitors.foldl (init := m1) fun acc mon =>
    if acc.shouldRotate mon.id then
      acc.rotateMonitor mon.id acc.activeTag
    else acc

end Graphics.Wallpaper
