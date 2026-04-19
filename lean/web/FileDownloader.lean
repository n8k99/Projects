-- File Downloader — resumable transfer with atomic file creation.
-- Pure-functional model: the in-memory simulation of download state.
-- Real filesystem operations would be in IO but the state machine is
-- what G072 teaches.

namespace Web

structure DLOpts where
  resume : Bool := true
  chunkSize : Nat := 8192
  expectedHash : Option UInt64 := none
  deriving Repr

structure DLState where
  bytesWritten : Nat
  partial : ByteArray
  hashState : UInt64 := 0
  bytesHashed : Nat := 0
  deriving Repr

def DLState.empty : DLState :=
  { bytesWritten := 0, partial := ByteArray.empty }

/-- Stream-hash update: fold each byte with `h := h*31 + byte`. -/
private def updateHash (state : DLState) (chunk : ByteArray) : DLState :=
  let mut h := state.hashState
  for i in [0:chunk.size] do
    h := h * 31 + chunk.get! i |>.toUInt64
  { state with hashState := h, bytesHashed := state.bytesHashed + chunk.size }

/-- Finalise: append length as last mix-in, matching the byte-sum-hash
    that other Rosetta Stone languages use. -/
def DLState.finalHash (s : DLState) : UInt64 :=
  s.hashState + s.bytesHashed.toUInt64

/-- Simulate receiving a chunk — append to partial, update hash. -/
def DLState.receive (s : DLState) (chunk : ByteArray) : DLState :=
  let merged := s.partial ++ chunk
  let s' := { s with partial := merged, bytesWritten := merged.size }
  updateHash s' chunk

/-- Simulate "open for resume": seed the hash state from bytes already on disk. -/
def DLState.fromPartial (existing : ByteArray) : DLState :=
  let base : DLState := {
    bytesWritten := existing.size,
    partial := existing
  }
  updateHash base existing

inductive DLOutcome where
  | completed (bytesWritten : Nat) (verified : Bool)
  | hashMismatch (expected actual : UInt64)
  deriving Repr

/-- Finalise the download: check hash (if provided) and report outcome. -/
def finalize (s : DLState) (opts : DLOpts) : DLOutcome :=
  let actual := s.finalHash
  match opts.expectedHash with
  | none => .completed s.bytesWritten false
  | some exp =>
    if exp == actual then .completed s.bytesWritten true
    else .hashMismatch exp actual

end Web
