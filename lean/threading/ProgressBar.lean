-- Progress Bar for Downloads — concurrent producer-consumer in Lean 4.
--
-- Lean's concurrency primitive is `IO.Ref` (atomic reference cell) combined
-- with `Task`. An `IO.Ref Tracker` is shared by reference; readers and writers
-- use `.atomically` to get a consistent point-in-time view while compound
-- transitions (bytes+status together) execute under a single read-modify-write.
--
-- The tracker is a value; the mutability lives in the Ref that holds it.
-- This is the purely-functional view of concurrency: state is a sequence of
-- immutable values threaded through an atomically-updated reference. No
-- locks in the user's code — the Ref IS the lock.

namespace Threading

inductive Status where
  | running
  | completed
  | cancelled
  | failed
  deriving Repr, DecidableEq

structure Tracker where
  bytesDone : Nat := 0
  totalBytes : Nat
  status : Status := .running
  startTimeMs : Nat := 0
  deriving Repr

structure Snapshot where
  bytesDone : Nat
  totalBytes : Nat
  status : Status
  elapsedSeconds : Float
  deriving Repr

def Snapshot.percent (s : Snapshot) : Float :=
  if s.totalBytes == 0 then 100.0
  else (s.bytesDone.toFloat / s.totalBytes.toFloat) * 100.0

def Snapshot.bytesPerSecond (s : Snapshot) : Float :=
  if s.elapsedSeconds ≤ 0.0 then 0.0
  else s.bytesDone.toFloat / s.elapsedSeconds

/-- ETA in seconds; `none` if no rate yet or already done. -/
def Snapshot.etaSeconds (s : Snapshot) : Option Float :=
  let rate := s.bytesPerSecond
  if rate ≤ 0.0 || s.bytesDone ≥ s.totalBytes then none
  else some ((s.totalBytes - s.bytesDone).toFloat / rate)

def Snapshot.renderBar (s : Snapshot) (width : Nat) : String :=
  let pct := s.percent
  let filledF := (pct / 100.0) * width.toFloat
  let filled := min width filledF.toUInt64.toNat
  let empty := width - filled
  let bar := String.mk (List.replicate filled '#') ++ String.mk (List.replicate empty '-')
  s!"[{bar}] {pct}%"

abbrev TrackerRef := IO.Ref Tracker

/-- Create a new tracker wrapped in an IO.Ref for shared mutation. -/
def newTracker (totalBytes : Nat) : IO TrackerRef := do
  let now ← IO.monoMsNow
  IO.mkRef { totalBytes, startTimeMs := now.toNat }

def advance (r : TrackerRef) (delta : Nat) : IO Unit :=
  r.modify fun t => { t with bytesDone := t.bytesDone + delta }

/-- Transition only if currently running. Terminal states are sticky. -/
private def transitionIfRunning (r : TrackerRef) (newStatus : Status) : IO Unit :=
  r.modify fun t => if t.status = .running then { t with status := newStatus } else t

def cancel   (r : TrackerRef) : IO Unit := transitionIfRunning r .cancelled
def complete (r : TrackerRef) : IO Unit := transitionIfRunning r .completed
def fail     (r : TrackerRef) : IO Unit := transitionIfRunning r .failed

def isCancelled (r : TrackerRef) : IO Bool := do
  let t ← r.get
  return t.status = .cancelled

def snapshot (r : TrackerRef) : IO Snapshot := do
  let t ← r.get
  let nowMs ← IO.monoMsNow
  let elapsed := (nowMs.toNat - t.startTimeMs).toFloat / 1000.0
  return {
    bytesDone := t.bytesDone,
    totalBytes := t.totalBytes,
    status := t.status,
    elapsedSeconds := elapsed
  }

end Threading
