-- Bulk Thumbnail Creator — CPU-bound fan-out with content-addressed dedup.
--
-- Workers compute a content hash and check a shared index before running
-- the expensive thumbnailer. Identical content processed twice produces one
-- thumbnail; the second request just records the existing result.
--
-- Reservation pattern: under the mutex, claim the slot with a placeholder;
-- release; run the thumbnailer; reacquire and install. Concurrent workers
-- with the same hash see the placeholder and skip generation.

namespace Threading.Thumbs

structure Request where
  sourceId : String
  content : ByteArray
  deriving Repr

structure IndexState where
  thumbnails : List (UInt64 × String) := []       -- hash -> thumbnail
  mappings : List (String × UInt64) := []         -- source_id -> hash
  generated : Nat := 0
  deduplicated : Nat := 0
  deriving Repr

structure QueueState where
  queue : List Request := []
  deriving Repr

structure Creator where
  queue : IO.Mutex QueueState
  index : IO.Mutex IndexState
  workerCount : Nat

def newCreator (workerCount : Nat) : IO Creator := do
  if workerCount < 1 then
    throw (IO.userError "workerCount must be >= 1")
  let q ← IO.Mutex.new {}
  let i ← IO.Mutex.new {}
  return { queue := q, index := i, workerCount }

def enqueueThumb (c : Creator) (r : Request) : IO Unit :=
  c.queue.atomically do
    modify fun s => { s with queue := s.queue ++ [r] }

private def popRequest (c : Creator) : IO (Option Request) :=
  c.queue.atomically do
    let s ← get
    match s.queue with
    | [] => return none
    | r :: rest =>
      set { s with queue := rest }
      return some r

private def claim (c : Creator) (hash : UInt64) : IO Bool :=
  c.index.atomically do
    let s ← get
    if s.thumbnails.any (·.1 == hash) then
      return false
    else
      set { s with thumbnails := s.thumbnails ++ [(hash, "")] }
      return true

private def installThumb (c : Creator) (hash : UInt64)
    (thumb sourceId : String) : IO Unit :=
  c.index.atomically do
    modify fun s => {
      s with
      thumbnails := s.thumbnails.map (fun p => if p.1 == hash then (p.1, thumb) else p),
      mappings := s.mappings ++ [(sourceId, hash)],
      generated := s.generated + 1
    }

private def recordDedup (c : Creator) (hash : UInt64) (sourceId : String)
    : IO Unit :=
  c.index.atomically do
    modify fun s => {
      s with
      mappings := s.mappings ++ [(sourceId, hash)],
      deduplicated := s.deduplicated + 1
    }

private partial def workerLoop (c : Creator)
    (hasher : ByteArray → UInt64)
    (thumbnailer : UInt64 → ByteArray → IO String) : IO Unit := do
  match (← popRequest c) with
  | none => return ()
  | some req =>
    let hash := hasher req.content
    if (← claim c hash) then
      let thumb ← thumbnailer hash req.content
      installThumb c hash thumb req.sourceId
    else
      recordDedup c hash req.sourceId
    workerLoop c hasher thumbnailer

def runWithThumb (c : Creator)
    (hasher : ByteArray → UInt64)
    (thumbnailer : UInt64 → ByteArray → IO String) : IO IndexState := do
  let mut tasks := #[]
  for _ in [0:c.workerCount] do
    let t ← IO.asTask (workerLoop c hasher thumbnailer)
    tasks := tasks.push t
  for t in tasks do
    let _ ← IO.wait t
  c.index.atomically do return (← get)

end Threading.Thumbs
