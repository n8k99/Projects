# G072 — File Downloader

> The Rosetta Stone's first project on **durable writes**. Where G066 managed a pool of downloads and G068 aggregated workers into a shared index, G072 focuses on the correctness of **a single transfer** as a system-failure-survivable operation. Three orthogonal guarantees: atomic rename at end, resumable from `.partial`, hash-verified before promotion.

```yaml
id: G072
title: File Downloader
category: web
requires: [G063-josephus, G068-bulk-thumbnail]
provides: [atomic-file-creation, resumable-transfer, hash-verified-promotion, partial-as-state, transport-as-parameter]
```

## Insight: The File on Disk Is the Durable State

In G065 and G066, the worker's state lived in memory. If the process died, the state died. G072 is the first project where **the state survives the process** — the `.partial` file on disk is the state, and the process can die arbitrarily without losing work.

This is the boundary between "memory is state" and "disk is state." A download that takes 10 minutes must not restart from zero when the network blips at minute 9. The solution: the bytes already received live on disk in a file whose name (`.partial` suffix) encodes "transfer in progress," and the transfer resumes from that file's size.

This is the pattern that every production download-manager, rsync, torrent client, and backup system uses. G072 presents it at minimal scale. The noosphere's future file-sync and media-fetch will use this exact pattern.

## Insight: Atomic Rename Is the Promotion Primitive

The download never writes to `dest` directly. It writes to `dest.partial` during the transfer, then calls `rename(.partial, dest)` when the bytes are verified. `rename` on POSIX is atomic with respect to observers of `dest`: there is no instant where `dest` exists as a partial file. An observer checking for `dest` sees either absence (transfer in progress) or the complete file (transfer done).

This is the standard primitive for **making a write appear to happen instantaneously** from the observer's perspective, even though the underlying work took ten minutes. It is how databases commit (WAL + fsync + rename), how config files update (write to `.tmp`, rename), how package managers replace binaries (write to `.new`, rename), how editors save files (write to backup, rename over original).

First Rosetta Stone project where **atomic promotion** is the correctness property. The vault's document-write flow will eventually need this — a half-saved note is worse than no save. G072 shows the primitive.

## Insight: Resume Requires the Hash to Account for Already-Downloaded Bytes

This is the subtle correctness bug in resume: if the hasher starts fresh on the second attempt, it hashes only the bytes received on the second attempt, not the bytes already on disk from the first attempt. The final hash is wrong even though the bytes on disk are correct.

The fix: on resume, **open `.partial`, read its bytes, seed the hasher with them, then continue appending new bytes to both the file AND the hasher**. The final hash is now the hash of the entire byte stream, whether it was downloaded in one attempt or ten.

This is the first Rosetta Stone project where an invariant **spans the process lifetime**. The hash state must be recoverable across restart, which means either serialising it to disk alongside `.partial` (more complex) or re-deriving it from `.partial` at resume time (what G072 does). Every real incremental-hash system faces this choice.

Parallels in production:
- Rsync's block-level checksum table recovers state by reading both ends.
- `git fsck` rederives the integrity state from stored objects.
- BitTorrent clients re-hash piece-by-piece on startup to find which pieces they already have.

## Insight: Transport Is a Parameter

The downloader doesn't know HTTP, FTP, local copy, or in-memory test transports. It just asks a `Transport` trait / interface / function for "bytes starting at offset N." What protocol lives behind the trait is irrelevant to the correctness properties G072 guarantees.

First Rosetta Stone project with **dependency injection as the transport boundary**. Tests use an in-memory `FakeTransport` that can be told to fail at specific offsets to exercise resume paths. Production uses HTTP clients. The downloader code is identical.

This pattern — "algorithm is a parameter over an abstract transport" — is the foundation of testable networked systems. Every integration test of the noosphere's agent-dispatch will use this: the "transport" is a fake that plays back pre-recorded interactions, and the production transport is real HTTP. G072 presents it explicitly.

## Insight: Hash-Mismatch Keeps the .partial for Inspection

If the expected hash doesn't match the actual hash, the download does NOT rename. `dest` stays absent; `.partial` stays present. This is deliberate: the corrupt bytes are available for inspection, debugging, or content-addressed recovery (G068 could identify the mismatch if the bytes had been seen correctly before).

The alternative — delete `.partial` on hash mismatch — loses the evidence. A corrupt download is interesting; if it happens repeatedly at the same offset, the upstream transport is broken and the user needs that signal. G072 keeps the bytes by default; a future `clean_up_on_mismatch` option is trivial to add.

This is the first Rosetta Stone case where **failure produces diagnostic artifacts rather than returning to a clean state**. Parallels: core dumps on crashes, `.rej` files from patch conflicts, failed-migration logs in database tools. All leave evidence; the principle is "fail noisily, with forensics."

## Choreographic Case: Vault Media Ingestion with Integrity

```innate
(@ingest-remote-media){
  @url <- @params/url
  @dest <- @vault/media-path{url: @url}
  @expected-hash <- @manifest/lookup-hash{url: @url}

  @outcome <- @download/fetch{
    transport: @http/ranged{url: @url},
    dest: @dest,
    opts: {resume: true, expected_hash: @expected-hash}
  } catch @corruption (hash-mismatch) {
    @log{level: "error", url: @url, mismatch: @corruption}
    @manifest/flag-corrupt{url: @url}
    @retry-with-full-refetch
  }

  where { integrity_verified: @outcome.verified }
  @vault/register-media{path: @dest, hash: @expected-hash}
}
```

The choreography reads naturally because the primitive gives the right guarantees: resume picks up interrupted transfers; hash verification catches corruption; atomic rename means the vault never sees a partial file. Media ingestion without these properties silently fails in ways that only show up later as broken links.

## Structures

```innate
(defstruct download-opts
  resume         : Bool           ;; pick up from .partial if present
  chunk-size     : Int            ;; bytes per transport read
  expected-hash  : Int?)          ;; if set, verified before rename

(defstruct outcome
  bytes-written  : Int
  final-path     : Path
  verified       : Bool)          ;; true iff expected-hash was set and matched
```

## Resolver Natives

```innate
@download/fetch{transport, dest, opts, on_progress?}       -> Outcome
@download/byte-sum-hash{bytes}                             -> Int    ;; integrity helper
```

## Demo

```innate
(@demo){
  @expected <- @download/byte-sum-hash{bytes: @known-content}

  ;; First attempt: simulated network failure at byte 300.
  try {
    @download/fetch{transport: @flaky-transport, dest: "/tmp/file.bin"}
  } catch @err {
    @print "first attempt failed; .partial has some bytes"
  }

  ;; Second attempt: resume from .partial, verify hash.
  @outcome <- @download/fetch{
    transport: @good-transport,
    dest: "/tmp/file.bin",
    opts: {resume: true, expected_hash: @expected}
  }
  ;; -> {bytes_written: 1000, final_path: "/tmp/file.bin", verified: true}
}
```

## Where

The downloader MUST write to `<dest>.partial` during transfer, never directly to `dest` — observers MUST see either absence of `dest` or the complete file, never a half-written `dest`. On success, `rename(.partial, dest)` MUST be the final step (POSIX atomicity); on Windows, the equivalent is `MoveFileEx` with `MOVEFILE_REPLACE_EXISTING`, which is also atomic. On interruption (transport error, process death), `.partial` MUST persist untouched — deleting it on error loses the resumable state. On resume, the hasher MUST be seeded with the bytes already on disk — otherwise the final hash reflects only the bytes received in the current attempt and the verification is wrong. On hash mismatch, the rename MUST be refused — promoting a file whose hash doesn't match is a correctness bug. The transport parameter MUST be abstract — hardcoding HTTP makes the downloader untestable and unreusable; every real use case has a different transport and the correctness properties are identical.
