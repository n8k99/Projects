-- P2P File Sharing App — decentralized file exchange between peers.
-- Pure functional model: peers, catalogs, chunked transfers, and integrity checks.

namespace Networking

-- Simple hash: sum of byte values as a hex string (placeholder for SHA-256).
-- Lean's stdlib doesn't include crypto, so we use a structural fingerprint.
private def contentHash (data : String) : String :=
  let h := data.foldl (init := 5381) fun acc c =>
    (acc * 33 + c.toNat) % (2^64)
  s!"{h}"

structure P2PFileChunk where
  index : Nat
  data : String
  hash : String
  deriving Repr, BEq

namespace P2PFileChunk

def fromData (index : Nat) (data : String) : P2PFileChunk :=
  { index, data, hash := contentHash data }

def verify (c : P2PFileChunk) : Bool :=
  contentHash c.data == c.hash

end P2PFileChunk

structure P2PSharedFile where
  name : String
  size : Nat
  hash : String
  chunkSize : Nat := 64
  chunkCount : Nat := 0
  deriving Repr, BEq

def mkSharedFile (name : String) (size : Nat) (hash : String) (chunkSize : Nat := 64) : P2PSharedFile :=
  let count := if size == 0 then 0 else (size + chunkSize - 1) / chunkSize
  { name, size, hash, chunkSize, chunkCount := count }

structure P2PPeerInfo where
  peerId : String
  address : String
  port : Nat
  lastSeen : Nat := 0
  sharedFiles : List P2PSharedFile := []
  deriving Repr

inductive P2PXferStatus where
  | pending | active | completed | failed
  deriving Repr, BEq

structure P2PTransferReq where
  requestId : String
  filename : String
  requesterId : String
  providerId : String
  status : P2PXferStatus := .active
  chunksReceived : Nat := 0
  totalChunks : Nat := 0
  deriving Repr

def P2PTransferReq.progress (t : P2PTransferReq) : Float :=
  if t.totalChunks == 0 then 0.0
  else t.chunksReceived.toFloat / t.totalChunks.toFloat

structure P2PPeerNode where
  peerId : String
  address : String := "127.0.0.1"
  port : Nat := 0
  peers : List P2PPeerInfo := []
  localFiles : List (String × String) := []    -- (name, data)
  sharedCatalog : List P2PSharedFile := []
  transfers : List P2PTransferReq := []
  chunksStore : List (String × Array (Option P2PFileChunk)) := []
  nextReqId : Nat := 0
  deriving Repr

namespace P2PPeerNode

-- Local file management

def addFile (n : P2PPeerNode) (name data : String) (chunkSize : Nat := 64) : P2PPeerNode × P2PSharedFile :=
  let hash := contentHash data
  let sf := mkSharedFile name data.length hash chunkSize
  let n := { n with
    localFiles := n.localFiles.filter (·.1 != name) ++ [(name, data)]
    sharedCatalog := n.sharedCatalog.filter (·.name != name) ++ [sf] }
  (n, sf)

def removeFile (n : P2PPeerNode) (name : String) : P2PPeerNode :=
  { n with
    localFiles := n.localFiles.filter (·.1 != name)
    sharedCatalog := n.sharedCatalog.filter (·.name != name) }

def getChunk (n : P2PPeerNode) (filename : String) (idx : Nat) : Option P2PFileChunk :=
  match n.localFiles.find? (·.1 == filename), n.sharedCatalog.find? (·.name == filename) with
  | some (_, data), some sf =>
    let start := idx * sf.chunkSize
    if start >= data.length then none
    else
      let endPos := min (start + sf.chunkSize) data.length
      let slice := data.extract ⟨start⟩ ⟨endPos⟩
      some (P2PFileChunk.fromData idx slice)
  | _, _ => none

-- Peer discovery

def discoverPeer (n : P2PPeerNode) (peerId address : String) (port : Nat) : P2PPeerNode :=
  let info : P2PPeerInfo := { peerId, address, port }
  { n with peers := n.peers.filter (·.peerId != peerId) ++ [info] }

def removePeer (n : P2PPeerNode) (peerId : String) : P2PPeerNode :=
  { n with peers := n.peers.filter (·.peerId != peerId) }

-- Catalog exchange

def getCatalog (n : P2PPeerNode) : List P2PSharedFile := n.sharedCatalog

def receiveCatalog (n : P2PPeerNode) (peerId : String) (catalog : List P2PSharedFile) : P2PPeerNode :=
  { n with peers := n.peers.map fun p =>
    if p.peerId == peerId then { p with sharedFiles := catalog } else p }

def searchNetwork (n : P2PPeerNode) (query : String) : List (String × P2PSharedFile) :=
  let q := query.toLower
  n.peers.foldl (init := []) fun acc peer =>
    acc ++ (peer.sharedFiles.filter (·.name.toLower.containsSubstr q)).map (peer.peerId, ·)

-- File transfer

def requestFile (n : P2PPeerNode) (providerId filename : String) : Except String (P2PPeerNode × String) := do
  let peer ← match n.peers.find? (·.peerId == providerId) with
    | some p => .ok p
    | none => .error s!"Unknown peer: {providerId}"
  let sf ← match peer.sharedFiles.find? (·.name == filename) with
    | some f => .ok f
    | none => .error s!"File not in catalog: {filename}"
  let reqId := s!"xfer-{n.nextReqId + 1}"
  let xfer : P2PTransferReq := {
    requestId := reqId, filename, requesterId := n.peerId,
    providerId, totalChunks := sf.chunkCount }
  let chunks : Array (Option P2PFileChunk) := Array.mkArray sf.chunkCount none
  let n := { n with
    nextReqId := n.nextReqId + 1
    transfers := n.transfers ++ [xfer]
    chunksStore := n.chunksStore ++ [(reqId, chunks)] }
  .ok (n, reqId)

def receiveChunk (n : P2PPeerNode) (requestId : String) (chunk : P2PFileChunk) : Except String P2PPeerNode := do
  unless chunk.verify do throw "Chunk verification failed"
  let xfer ← match n.transfers.find? (·.requestId == requestId) with
    | some t => if t.status == .active then .ok t else .error "Transfer not active"
    | none => .error "Unknown transfer"
  let (_, chunks) ← match n.chunksStore.find? (·.1 == requestId) with
    | some pair => .ok pair
    | none => .error "No chunk store"
  if chunk.index >= chunks.size then throw "Chunk index out of range"
  let chunks := chunks.set! chunk.index (some chunk)
  let received := chunks.foldl (init := 0) fun acc c => if c.isSome then acc + 1 else acc
  let completed := received == xfer.totalChunks
  let status := if completed then P2PXferStatus.completed else P2PXferStatus.active
  let n := { n with
    transfers := n.transfers.map fun t =>
      if t.requestId == requestId then { t with chunksReceived := received, status } else t
    chunksStore := n.chunksStore.map fun (rid, cs) =>
      if rid == requestId then (rid, chunks) else (rid, cs) }
  if completed then
    let assembled := chunks.foldl (init := "") fun acc c =>
      match c with | some ch => acc ++ ch.data | none => acc
    .ok { n with localFiles := n.localFiles.filter (·.1 != xfer.filename) ++ [(xfer.filename, assembled)] }
  else .ok n

end P2PPeerNode

-- Simulate a full transfer between two peers
def simulateP2PTransfer (provider requester : P2PPeerNode) (filename : String)
    : Except String (P2PPeerNode × P2PPeerNode × String) := do
  let (requester, reqId) ← requester.requestFile provider.peerId filename
  let xfer ← match requester.transfers.find? (·.requestId == reqId) with
    | some t => .ok t
    | none => .error "Transfer not found"
  let mut req := requester
  for i in [:xfer.totalChunks] do
    match provider.getChunk filename i with
    | some chunk => req ← req.receiveChunk reqId chunk |>.toOption |>.getD req
    | none => pure ()
  .ok (provider, req, reqId)

-- Demo
def p2pDemo : IO Unit := do
  let (alice, _) := P2PPeerNode.addFile { peerId := "alice" } "rosetta.txt" "The Rosetta Stone." 8
  let bob := (P2PPeerNode.discoverPeer { peerId := "bob" } "alice" "127.0.0.1" 8001)
  let bob := bob.receiveCatalog "alice" alice.getCatalog
  let results := bob.searchNetwork "rosetta"
  IO.println s!"Found {results.length} result(s)"
  match simulateP2PTransfer alice bob "rosetta.txt" with
  | .ok (_, bob', reqId) =>
    IO.println s!"Transfer {reqId} complete"
    IO.println s!"Bob has {bob'.localFiles.length} file(s)"
  | .error e => IO.println s!"Error: {e}"

end Networking
