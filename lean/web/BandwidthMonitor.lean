-- Bandwidth Monitor — time-series rate calculation over samples.
-- Pure-functional: the monitor is a list of samples; all queries are folds.

namespace Web.BW

structure Sample where
  timestampMs : Nat
  bytesTotal : Nat
  deriving Repr

structure Monitor where
  samples : List Sample := []       -- oldest first
  capacity : Nat
  deriving Repr

def Monitor.mk! (capacity : Nat) : Monitor :=
  if capacity < 2 then panic! "need at least 2 samples"
  else { samples := [], capacity }

def Monitor.record (m : Monitor) (ts bytes : Nat) : Monitor :=
  let newS : Sample := { timestampMs := ts, bytesTotal := bytes }
  let samples := m.samples ++ [newS]
  let trimmed := if samples.length > m.capacity then samples.drop 1 else samples
  { m with samples := trimmed }

def Monitor.sampleCount (m : Monitor) : Nat := m.samples.length

def Monitor.totalBytes (m : Monitor) : Nat :=
  let folded := m.samples.foldl (init := (none : Option Sample, 0 : Nat))
    fun (prev, total) s =>
      match prev with
      | none => (some s, total)
      | some p =>
        let delta := if s.bytesTotal ≥ p.bytesTotal then s.bytesTotal - p.bytesTotal else 0
        (some s, total + delta)
  folded.2

def Monitor.instantaneousRate (m : Monitor) : Float :=
  let n := m.samples.length
  if n < 2 then 0.0
  else
    let a := m.samples.get! (n - 2)
    let b := m.samples.get! (n - 1)
    if b.timestampMs ≤ a.timestampMs || b.bytesTotal < a.bytesTotal then 0.0
    else
      let dtS := (b.timestampMs - a.timestampMs).toFloat / 1000.0
      (b.bytesTotal - a.bytesTotal).toFloat / dtS

/-- Average rate over the last `windowMs` ms from the latest sample. -/
def Monitor.averageRate (m : Monitor) (windowMs : Nat) : Float :=
  match m.samples.getLast? with
  | none => 0.0
  | some latest =>
    let minTs := if latest.timestampMs ≥ windowMs then latest.timestampMs - windowMs else 0
    -- Walk backward; take samples with timestamp ≥ minTs.
    let within := m.samples.filter (·.timestampMs ≥ minTs)
    match within.head? with
    | none => 0.0
    | some earliest =>
      if earliest.timestampMs == latest.timestampMs then 0.0
      else if latest.bytesTotal < earliest.bytesTotal then 0.0
      else
        let dtS := (latest.timestampMs - earliest.timestampMs).toFloat / 1000.0
        if dtS ≤ 0.0 then 0.0
        else (latest.bytesTotal - earliest.bytesTotal).toFloat / dtS

/-- Peak instantaneous rate across consecutive pairs in the window. -/
def Monitor.peakRate (m : Monitor) (windowMs : Nat) : Float :=
  match m.samples.getLast? with
  | none => 0.0
  | some latest =>
    let minTs := if latest.timestampMs ≥ windowMs then latest.timestampMs - windowMs else 0
    let folded := m.samples.foldl (init := (none : Option Sample, 0.0 : Float))
      fun (prev, peak) s =>
        match prev with
        | none => (some s, peak)
        | some p =>
          if s.timestampMs ≥ minTs && s.timestampMs > p.timestampMs
              && s.bytesTotal ≥ p.bytesTotal then
            let dtS := (s.timestampMs - p.timestampMs).toFloat / 1000.0
            let rate := (s.bytesTotal - p.bytesTotal).toFloat / dtS
            (some s, if rate > peak then rate else peak)
          else
            (some s, peak)
    folded.2

end Web.BW
