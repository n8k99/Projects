-- Scheduled Auto Login and Action — cron-like task scheduler.
-- Pure-functional: tick returns (new scheduler, runs fired).

namespace Web.Sched

def dayMs : Nat := 86_400_000

inductive Kind where | everyMs | atMs | daily
  deriving Repr, DecidableEq, BEq

structure Schedule where
  kind : Kind
  intervalMs : Nat := 0
  atMs : Nat := 0
  hour : Nat := 0
  minute : Nat := 0
  deriving Repr

def Schedule.everyMs (ms : Nat) : Schedule := { kind := .everyMs, intervalMs := ms }
def Schedule.atMs (t : Nat) : Schedule := { kind := .atMs, atMs := t }
def Schedule.daily (h m : Nat) : Schedule := { kind := .daily, hour := h, minute := m }

structure Task where
  id : Nat
  name : String
  schedule : Schedule
  action : String
  credentialRef : Option String
  nextFireMs : Nat
  enabled : Bool := true
  deriving Repr

inductive RunStatus where | succeeded | failed
  deriving Repr, DecidableEq, BEq

structure Run where
  taskId : Nat
  startedMs : Nat
  finishedMs : Nat
  status : RunStatus
  message : String
  deriving Repr

structure Scheduler where
  tasks : List Task := []
  history : List Run := []
  nextId : Nat := 1
  deriving Repr

def Scheduler.empty : Scheduler := {}

private def nextDaily (baseMs hour minute : Nat) : Nat :=
  let day := baseMs / dayMs
  let dayStart := day * dayMs
  let target := hour * 3_600_000 + minute * 60_000
  let candidate := dayStart + target
  if candidate > baseMs then candidate else candidate + dayMs

private def computeInitial (s : Schedule) (baseMs : Nat) : Nat :=
  match s.kind with
  | .everyMs => baseMs + s.intervalMs
  | .atMs => s.atMs
  | .daily => nextDaily baseMs s.hour s.minute

private partial def scheduleNext (s : Schedule) (prev now : Nat) : Nat × Bool :=
  match s.kind with
  | .everyMs =>
    let rec loop (nxt : Nat) : Nat :=
      if nxt > now then nxt else loop (nxt + s.intervalMs)
    (loop (prev + s.intervalMs), false)
  | .atMs => (2^31, true)
  | .daily => (nextDaily now s.hour s.minute, false)

def Scheduler.addTask (s : Scheduler) (name : String) (sched : Schedule)
    (action : String) (credentialRef : Option String)
    (firstFireBaseMs : Nat) : Scheduler × Nat :=
  let id := s.nextId
  let task : Task := {
    id, name, schedule := sched, action, credentialRef,
    nextFireMs := computeInitial sched firstFireBaseMs
  }
  ({ s with tasks := s.tasks ++ [task], nextId := id + 1 }, id)

def Scheduler.removeTask (s : Scheduler) (id : Nat) : Scheduler × Bool :=
  if s.tasks.any (·.id == id) then
    ({ s with tasks := s.tasks.filter (·.id ≠ id) }, true)
  else (s, false)

def Scheduler.setEnabled (s : Scheduler) (id : Nat) (enabled : Bool)
    : Scheduler × Bool :=
  if s.tasks.any (·.id == id) then
    ({ s with tasks := s.tasks.map fun t =>
      if t.id == id then { t with enabled } else t }, true)
  else (s, false)

def Scheduler.getTask (s : Scheduler) (id : Nat) : Option Task :=
  s.tasks.find? (·.id == id)

def Scheduler.tick (s : Scheduler) (nowMs : Nat)
    (dispatch : Task → Nat → Bool × String) : Scheduler × List Run :=
  let mut cur := s
  let mut runs : List Run := []
  for task in s.tasks do
    if task.enabled && task.nextFireMs ≤ nowMs then
      let (ok, msg) := dispatch task nowMs
      let r : Run := {
        taskId := task.id, startedMs := nowMs, finishedMs := nowMs,
        status := if ok then .succeeded else .failed, message := msg
      }
      runs := r :: runs
      let (nf, disable) := scheduleNext task.schedule task.nextFireMs nowMs
      cur := { cur with
        history := cur.history ++ [r],
        tasks := cur.tasks.map fun t =>
          if t.id == task.id then
            { t with nextFireMs := nf, enabled := t.enabled && !disable }
          else t }
  (cur, runs.reverse)

end Web.Sched
