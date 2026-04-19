-- Download Manager — N concurrent workers sharing one job queue.
--
-- Lean 4's concurrency primitive for CPU-bound work is `Task`. For the
-- producer-consumer pattern here, we use `IO.Mutex` on a list-as-queue and
-- `Task.spawn` to run workers; the runtime schedules them on the thread pool.
--
-- The queue is the rendezvous point. Workers don't know about each other —
-- they only know the mutex-protected queue. Load balancing is emergent: a
-- slow simulate function on one worker just means that worker pops less
-- frequently; the fast workers keep making progress.

namespace Threading

structure Download where
  url : String
  expectedBytes : Nat
  deriving Repr

inductive OutcomeStatus
  | completed | cancelled | failed
  deriving Repr, DecidableEq

structure DownloadResult where
  url : String
  status : OutcomeStatus
  bytes : Option Nat := none       -- set when status = completed
  error : Option String := none    -- set when status = failed
  deriving Repr

structure ManagerState where
  queue : List Download := []
  results : List DownloadResult := []
  cancelled : Bool := false
  deriving Repr

abbrev Manager := IO.Mutex ManagerState

def newManager : IO Manager := IO.Mutex.new {}

def enqueue (m : Manager) (d : Download) : IO Unit :=
  m.atomically do
    modify fun s => { s with queue := s.queue ++ [d] }

def cancel (m : Manager) : IO Unit :=
  m.atomically do modify fun s => { s with cancelled := true }

def queuedCount (m : Manager) : IO Nat :=
  m.atomically do return (← get).queue.length

def resultCount (m : Manager) : IO Nat :=
  m.atomically do return (← get).results.length

/-- Pop one job (nil if empty) and whether cancellation has been requested. -/
private def popJob (m : Manager) : IO (Option Download × Bool) :=
  m.atomically do
    let s ← get
    match s.queue with
    | [] => return (none, s.cancelled)
    | d :: rest =>
      set { s with queue := rest }
      return (some d, s.cancelled)

private def pushResult (m : Manager) (r : DownloadResult) : IO Unit :=
  m.atomically do
    modify fun s => { s with results := r :: s.results }

/-- One worker's loop: pop, simulate, push, repeat until queue empty or cancelled. -/
private partial def workerLoop (m : Manager)
    (simulate : Download → IO (Except String Nat)) : IO Unit := do
  let (d?, cancelled) ← popJob m
  match d? with
  | none => return ()
  | some d =>
    let r ←
      if cancelled then
        pure { url := d.url, status := OutcomeStatus.cancelled : DownloadResult }
      else
        match (← simulate d) with
        | .ok bytes =>
          pure { url := d.url, status := OutcomeStatus.completed,
                 bytes := some bytes : DownloadResult }
        | .error e =>
          pure { url := d.url, status := OutcomeStatus.failed,
                 error := some e : DownloadResult }
    pushResult m r
    workerLoop m simulate

/-- Spawn `workerCount` concurrent Tasks; wait for all of them; return results
    in completion order. -/
def runWith (m : Manager) (workerCount : Nat)
    (simulate : Download → IO (Except String Nat)) : IO (List DownloadResult) := do
  let mut tasks : List (Task (Except IO.Error Unit)) := []
  for _ in [0:workerCount] do
    let t ← IO.asTask (workerLoop m simulate)
    tasks := t :: tasks
  for t in tasks do
    let _ ← IO.wait t
  m.atomically do return (← get).results.reverse

end Threading
