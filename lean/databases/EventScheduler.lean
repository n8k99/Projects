-- Event Scheduler and Calendar — intervals, recurrence, overlap detection.

namespace Databases.ES

def msPerDay : Int := 86400000

inductive Recurrence where | none | daily | weekly | monthly
  deriving Repr, BEq

structure Event where
  id : Int
  title : String
  startMs : Int
  endMs : Int
  recurrence : Recurrence := .none
  seriesEndMs : Option Int := none
  deriving Repr

def Event.overlaps (e : Event) (otherStart otherEnd : Int) : Bool :=
  e.startMs < otherEnd ∧ otherStart < e.endMs

def Event.durationMs (e : Event) : Int := e.endMs - e.startMs

structure Occurrence where
  eventId : Int
  title : String
  startMs : Int
  endMs : Int
  deriving Repr

def recurrenceStep : Recurrence → Int
  | .daily => msPerDay
  | .weekly => 7 * msPerDay
  | .monthly => 30 * msPerDay
  | .none => 0

partial def expandEvent (e : Event) (fromMs toMs : Int) : List Occurrence :=
  let duration := e.durationMs
  let cutoff := e.seriesEndMs.getD (1 <<< 62)
  match e.recurrence with
  | .none =>
    if e.overlaps fromMs toMs then
      [{ eventId := e.id, title := e.title, startMs := e.startMs, endMs := e.endMs }]
    else []
  | r =>
    let step := recurrenceStep r
    if step ≤ 0 then []
    else
      let mut start := e.startMs
      if start < fromMs then
        let gap := fromMs - start
        let diff := max 0 (gap - duration)
        let skips := diff / step
        start := start + skips * step
      let rec loop (cur : Int) (acc : List Occurrence) : List Occurrence :=
        if cur ≥ toMs ∨ cur > cutoff then acc
        else
          let end' := cur + duration
          let acc' := if end' > fromMs then
            acc ++ [{ eventId := e.id, title := e.title, startMs := cur, endMs := end' }]
          else acc
          loop (cur + step) acc'
      loop start []

structure Calendar where
  events : List Event := []

def Calendar.add (c : Calendar) (e : Event) : Calendar :=
  { events := (c.events.filter (fun x => x.id ≠ e.id)) ++ [e] }

def Calendar.eventsBetween (c : Calendar) (fromMs toMs : Int) : List Occurrence :=
  let all := c.events.foldl (init := ([] : List Occurrence)) fun acc e =>
    acc ++ expandEvent e fromMs toMs
  all.mergeSort fun a b =>
    if a.startMs ≠ b.startMs then a.startMs < b.startMs
    else a.eventId < b.eventId

def Calendar.findConflicts (c : Calendar) (startMs endMs : Int) : List Int :=
  let ids := c.events.filterMap fun e =>
    if (expandEvent e startMs endMs).any fun occ =>
      occ.startMs < endMs ∧ startMs < occ.endMs
    then some e.id
    else none
  ids.mergeSort (· < ·)

end Databases.ES
