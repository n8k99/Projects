-- YouTube Downloader — concurrency-limited queue with resumable downloads.

namespace Graphics.YouTube

structure VideoFormat where
  id : String
  resolution : Nat
  codec : String := "avc1"
  container : String := "mp4"
  sizeBytes : Nat
  bitrateBps : Nat := 0
  deriving Repr

inductive FormatPolicy where
  | maxQuality
  | lowest
  | bestUnder (capBytes : Nat)
  deriving Repr

partial def pickFormat (formats : List VideoFormat) (policy : FormatPolicy) : Option VideoFormat :=
  match formats with
  | [] => none
  | first :: rest =>
    match policy with
    | .maxQuality =>
      let best := rest.foldl (fun a f => if f.resolution > a.resolution then f else a) first
      some best
    | .lowest =>
      let best := rest.foldl (fun a f => if f.resolution < a.resolution then f else a) first
      some best
    | .bestUnder cap =>
      let candidates := formats.filter (fun f => f.sizeBytes ≤ cap)
      match candidates with
      | [] => none
      | c :: cs =>
        some (cs.foldl (fun a f => if f.resolution > a.resolution then f else a) c)

structure RetryPolicy where
  maxAttempts : Nat := 3
  initialMs : Nat := 1000
  multiplier : Nat := 2
  deriving Repr

def RetryPolicy.backoffForAttempt (p : RetryPolicy) (n : Nat) : Nat :=
  if n == 0 then 0
  else
    let rec loop (ms k : Nat) : Nat :=
      if k == 0 then ms else loop (ms * p.multiplier) (k - 1)
    loop p.initialMs (n - 1)

inductive DownloadStateKind where
  | pending | inFlight | retrying | succeeded | failed
  deriving Repr, BEq

structure DownloadState where
  kind : DownloadStateKind := .pending
  readyAtMs : Nat := 0
  deriving Repr

structure Download where
  id : Nat
  videoId : String
  format : VideoFormat
  bytesDone : Nat := 0
  bytesTotal : Nat
  attempts : Nat := 0
  state : DownloadState := {}
  deriving Repr

inductive EventKind where
  | enqueued | started | progress | completed | failed
  | retryScheduled | abandoned
  deriving Repr, BEq

structure ManagerEvent where
  kind : EventKind
  id : Nat
  detail : String := ""
  deriving Repr

structure Manager where
  maxConcurrent : Nat
  bandwidthBps : Nat
  retryPolicy : RetryPolicy
  downloads : List Download := []
  pendingQueue : List Nat := []
  elapsedMs : Nat := 0
  events : List ManagerEvent := []
  nextId : Nat := 1
  deriving Repr

def Manager.enqueue (m : Manager) (videoId : String) (f : VideoFormat)
    : Manager × Nat :=
  let id := m.nextId
  let d : Download := {
    id, videoId, format := f,
    bytesTotal := f.sizeBytes
  }
  let m' := { m with
    downloads := m.downloads ++ [d],
    pendingQueue := m.pendingQueue ++ [id],
    events := m.events ++ [{ kind := .enqueued, id }],
    nextId := m.nextId + 1
  }
  (m', id)

def Manager.download (m : Manager) (id : Nat) : Option Download :=
  m.downloads.find? (fun d => d.id == id)

def Manager.activeCount (m : Manager) : Nat :=
  (m.downloads.filter (fun d => d.state.kind == .inFlight)).length

def updateDownload (downloads : List Download) (id : Nat)
    (f : Download → Download) : List Download :=
  downloads.map fun d => if d.id == id then f d else d

def Manager.promoteRetries (m : Manager) : Manager :=
  let now := m.elapsedMs
  let readyIds := (m.downloads.filter (fun d =>
    d.state.kind == .retrying ∧ d.state.readyAtMs ≤ now)).map (·.id)
  let newDownloads := m.downloads.map fun d =>
    if readyIds.contains d.id then
      { d with state := { kind := .pending } }
    else d
  { m with
    downloads := newDownloads,
    pendingQueue := m.pendingQueue ++ readyIds
  }

partial def Manager.fillSlots (m : Manager) : Manager :=
  if m.activeCount ≥ m.maxConcurrent then m
  else match m.pendingQueue with
  | [] => m
  | id :: rest =>
    let m' := { m with pendingQueue := rest }
    match m.download id with
    | none => m'.fillSlots
    | some d =>
      if d.state.kind != .pending then m'.fillSlots
      else
        let newAttempts := d.attempts + 1
        let updated := { d with
          attempts := newAttempts,
          state := { kind := .inFlight }
        }
        let m'' := { m' with
          downloads := updateDownload m.downloads id (fun _ => updated),
          events := m.events ++ [{ kind := .started, id,
                                    detail := s!"attempt {newAttempts}" }]
        }
        m''.fillSlots

def Manager.reportFailure (m : Manager) (id : Nat) : Manager :=
  match m.download id with
  | none => m
  | some d =>
    if d.state.kind != .inFlight then m
    else if d.attempts < m.retryPolicy.maxAttempts then
      let backoff := m.retryPolicy.backoffForAttempt d.attempts
      let ready := m.elapsedMs + backoff
      { m with
        downloads := updateDownload m.downloads id (fun d =>
          { d with state := { kind := .retrying, readyAtMs := ready } }),
        events := m.events ++ [
          { kind := .failed, id, detail := s!"attempt {d.attempts}" },
          { kind := .retryScheduled, id, detail := s!"ready {ready}" }
        ]
      }
    else
      { m with
        downloads := updateDownload m.downloads id (fun d =>
          { d with state := { kind := .failed } }),
        events := m.events ++ [
          { kind := .failed, id, detail := s!"attempt {d.attempts}" },
          { kind := .abandoned, id }
        ]
      }

def Manager.tick (m : Manager) (ms : Nat) : Manager :=
  let m1 := { m with elapsedMs := m.elapsedMs + ms }
  let m2 := m1.promoteRetries
  let m3 := m2.fillSlots
  let active := m3.downloads.filter (fun d => d.state.kind == .inFlight)
  if active.isEmpty then m3
  else
    let perStreamBps := m3.bandwidthBps / active.length
    let bytesThisTick := (perStreamBps * ms) / 8000
    active.foldl (init := m3) fun acc d =>
      let newDone := min (d.bytesDone + bytesThisTick) d.bytesTotal
      let newState := if newDone ≥ d.bytesTotal
        then ({ kind := .succeeded } : DownloadState)
        else d.state
      let progressed := newDone > d.bytesDone
      let completed := newDone ≥ d.bytesTotal
      let newEvents :=
        (if progressed then
          acc.events ++ [{ kind := .progress, id := d.id,
                            detail := s!"{newDone}/{d.bytesTotal}" }]
         else acc.events) ++
        (if completed then [{ kind := .completed, id := d.id }] else [])
      { acc with
        downloads := updateDownload acc.downloads d.id (fun dd =>
          { dd with bytesDone := newDone, state := newState }),
        events := newEvents
      }

end Graphics.YouTube
