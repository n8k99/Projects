# G037 — P2P File Sharing App

> Decentralized file exchange between peers with catalog discovery and chunked transfers.

```yaml
id: G037
title: P2P File Sharing App
category: networking
requires: [G031-cd-key-generator, G033-ftp-protocol, G035-chat-app]
provides: [symmetric-topology, peer-discovery, chunked-transfer, content-addressing]
```

## Insight: The First Symmetric Topology

Every network pattern before G037 had an asymmetry. FTP (G033): client asks, server answers. NTP (G034): client calibrates against authority. Chat (G035): server routes between clients. Weather (G036): client consumes from API provider. In every case, one side was structurally different from the other.

P2P shatters this. **Every peer is both client and server.** Alice serves files to Bob while requesting files from Carol. The same node that answers a catalog query also issues catalog queries. The role distinction collapses — there are no clients and servers, only peers. This is the noosphere's natural topology: agents are symmetric. Any agent can request work from any other agent. Any agent can serve results to any other agent. The hub-and-spoke model of G033–G036 was training wheels.

## Insight: Discovery Is the Bootstrap Problem

How does a peer find other peers? In chat (G035), users registered with a central server. In P2P, there's no center. Discovery requires at least one known peer to start — the bootstrap node. From there, peers exchange peer lists, and the network grows organically. This is gossip protocol: tell your neighbors about your other neighbors.

In the noosphere, agent discovery works the same way. A new ghost doesn't know who else exists. It starts with one known agent (the orchestrator), learns about others through interaction, and builds its own peer table. The bootstrap is the first `@reference` that resolves.

## Insight: Catalogs Are Distributed Indexes

No single peer has a global view. Each peer maintains its own index of what its neighbors share. Search is local — you query your known peers' catalogs, not a global database. The network's total knowledge is the union of all local catalogs, but no one sees the union. This is fundamentally different from G023's centralized note board or G036's single-authority API.

The vault works this way too. Each agent maintains its own model of what exists. Lena knows about daily notes. Kathryn knows about financial positions. Sylvia knows about publications. No single agent indexes everything. The orchestrator coordinates, but the knowledge is distributed.

## Insight: Chunked Transfer Is Progressive Verification

A file splits into chunks. Each chunk carries its own hash. The receiver verifies each chunk independently before accepting it. This is G013's progressive trust model applied to data transfer: cheap verification (hash check) gates expensive storage (writing to disk). A corrupted chunk is rejected without invalidating the entire transfer.

The chunk is the atomic unit of trust. You don't trust the whole file — you trust each piece, then assemble. If chunk 7 of 10 fails, you re-request chunk 7, not the entire file. Partial progress is preserved. This is how large choreographies should work: if step 7 fails, re-execute step 7, not the whole dance.

## Insight: Content Addressing Makes Identity Location-Independent

A file's SHA-256 hash identifies it regardless of which peer hosts it. The same file on Alice's machine and Bob's machine has the same hash. You can request "file with hash X" from any peer that has it — the content IS the address. This decouples identity from location: the file doesn't live "on Alice's machine." It lives everywhere its hash exists.

G023 (Post-it Notes) introduced identity via sequential IDs — location-dependent, server-assigned. Content addressing is the opposite: identity derived from the content itself, requiring no central authority. This is how the vault should work: a note's identity is its content hash, not its file path. Move the file, rename it — the hash stays the same.

## Choreographic Case: Distributed Knowledge Assembly

```innate
(@research-brief){
  concurrent {
    @kathryn_data <- @kathryn/catalog{query: "Q2 financials"}
    @eliana_data <- @eliana/catalog{query: "infra costs Q2"}
    @sylvia_data <- @sylvia/catalog{query: "publication metrics"}
  }
  // Each agent searched their local catalog. No central index.
  join {
    @transfers <- concurrent {
      @fin <- @request_file{from: @kathryn, file: @kathryn_data.best_match}
      @inf <- @request_file{from: @eliana, file: @eliana_data.best_match}
      @pub <- @request_file{from: @sylvia, file: @sylvia_data.best_match}
    }
  }
  where {
    all_transfers_complete: @fin.status == :completed
      AND @inf.status == :completed
      AND @pub.status == :completed
    // Only proceed when all three files have been fully received
    // and their content hashes verified.
  }
}
```

The choreography doesn't require a central file server. Each agent serves from their own store. The requester assembles the full picture from distributed sources. The `where` gates on completeness AND integrity — every chunk verified.

## Structures

```innate
(defstruct file-chunk
  index     : Nat
  data      : Bytes
  sha256    : String)

(defstruct shared-file
  name        : String
  size        : Nat
  sha256      : String
  chunk-size  : Nat
  chunk-count : Nat)

(defstruct peer-info
  peer-id      : String
  address      : String
  port         : Nat
  last-seen    : Instant
  shared-files : [SharedFile])

(defstruct transfer-request
  request-id   : String
  filename     : String
  requester-id : String
  provider-id  : String
  status       : :pending | :active | :completed | :failed
  chunks-received : Nat
  total-chunks    : Nat)

(defstruct peer-node
  peer-id        : String
  address        : String
  port           : Nat
  peers          : {String -> PeerInfo}
  local-files    : {String -> Bytes}
  shared-catalog : {String -> SharedFile}
  transfers      : {String -> TransferRequest}
  chunks-store   : {String -> [FileChunk?]})
```

## Resolver Natives

```innate
@peer/discover{id: String, address: String, port: Nat}  -> PeerInfo
@peer/catalog{peer-id: String}                           -> [SharedFile]
@peer/search{query: String}                              -> [(String, SharedFile)]
@peer/request{from: String, file: String}                -> TransferRequest
@peer/chunk{file: String, index: Nat}                    -> FileChunk
```

## Demo

```innate
(@demo){
  @alice <- @peer{id: "alice", address: "192.168.1.10", port: 8001}
  @bob   <- @peer{id: "bob",   address: "192.168.1.11", port: 8002}
  @alice/share{name: "rosetta.txt", data: "The Rosetta Stone is not a learning exercise."}
  @bob/discover{peer: @alice}
  @bob/receive-catalog{from: @alice}
  @results <- @bob/search{query: "rosetta"}
  @xfer <- @simulate-transfer{from: @alice, to: @bob, file: "rosetta.txt"}
  @print{@xfer.status}    ;; => :completed
  @print{@xfer.progress}  ;; => 1.0
}
```
