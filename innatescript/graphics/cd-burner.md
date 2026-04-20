# G120 — CD Burning App

> The Rosetta Stone's seventh **Graphics project**. Models the **multi-session CD burn kernel** every disc-authoring tool ships: a disc is a capacity-bounded container of **sessions**, each session is a bounded list of **tracks** (files), sessions cost overhead (leadin + leadout), and discs finalize (lock) after which no more sessions can be added. The distinctive move: capacity accounting is **session-aware** — adding a new session subtracts overhead from usable space, so a half-full disc has less remaining room than a naive "capacity − file bytes" calculation would suggest.

```yaml
id: G120
title: CD Burning App
category: graphics
requires: [G068-bulk-thumbnail, G098-file-copier, G107-budget-tracker]
provides: [multi-session-capacity, ffd-bin-packing, iso-9660-names, finalize-fsm]
```

## Insight: Capacity Accounting Is Session-Aware

Naive: `remaining = capacity - sum(file_bytes)`. Real disc burning: `remaining = capacity - sum(session_total_bytes)` where `session_total_bytes = track_bytes + leadin + leadout`. Typical CD-R: 700 MB capacity, ~13 MB leadin, ~22 MB leadout — each new session costs ~35 MB before you put a single byte of data in it.

This is why multi-session CDs have "diminishing returns": a 700 MB disc with 5 appended sessions has used 175 MB on overhead alone.

First Rosetta Stone project where **a container's capacity has metadata-linked overhead** that scales with the number of logical units inside it. G107's budget tracker had sums without overhead; G120 introduces per-session fixed costs.

## Insight: First-Fit-Decreasing Is the Right Pack

Given a set of files and `N` blank discs, the classic bin-packing problem. **First-fit-decreasing (FFD)** — sort files by size descending, place each in the first bin that fits — is within 11/9 of optimal for bin packing and fast (O(n log n)). For disc authoring, it's the universal choice: deterministic, fast, preview-able.

Files that don't fit in any bin go to `overflow` — the caller decides whether to buy more discs, split files, or drop them.

First Rosetta Stone project to use **a well-known approximation algorithm whose name is the algorithm**. G111's Kahn's topo-sort was named; FFD is also named, and the choice (over first-fit, best-fit-decreasing, or full optimal) is a documented tradeoff.

## Insight: ISO 9660 8.3 Names Disambiguate via `~N`

CD file systems at Level 1 require 8.3 (8-char stem + 3-char extension), uppercase, alphanumeric + underscore only. Arbitrary filenames get **canonicalized**: strip invalid chars, uppercase, truncate. If two files canonicalize to the same ISO name, the second gets a `~1` suffix (then `~2`, etc.) — but to make room for the suffix, the stem is truncated to `8 - len(suffix)` chars.

So `report.txt` → `REPORT.TXT`. A second `report.txt` → `REPORT~1.TXT` (6 stem + 2 suffix = 8). After `~9`, `~10` consumes 3 chars, leaving 5 for the stem: `REPOR~10.TXT`.

First Rosetta Stone project where **a name transformation is lossy and collision-driven**. G083's package names were unique by construction; G120's ISO names are mechanically-derived from lossy inputs with structured disambiguation.

## Insight: Finalize Is a One-Way FSM

`Blank → SessionOpen ↔ SessionClosed → Finalized`. Once finalized, no more sessions, no more files. This is physical — a finalized disc has its lead-out written to the outer edge; there's nowhere to append.

The FSM is simpler than G117's 5-state streaming one and narrower than G118's player — three live states (Blank, SessionOpen, SessionClosed) + one absorbing state (Finalized). Every destructive op checks `state == Finalized` first and refuses.

First Rosetta Stone project with an **absorbing terminal state** in its FSM. G117's Ended was also terminal but implicit (no further events expected); G120's Finalized is explicit and every operation rejects it.

## Insight: Session-Open Is a Transient Mode

Between `open_session` and `close_session`, writes land in the current session. Outside that window, writes are rejected (`NoOpenSession`). This mirrors a transactional model: the session is a write boundary, and `close_session` is the commit.

Opening a second session after closing the first is allowed (until finalize); the disc tracks them in order.

First Rosetta Stone project where **a resource has a "write window" with explicit open/close ops** that gate all mutating operations. G098's file copier was always writable; G120 has per-session write gates.

## Choreographic Case: Vault Backup Burner

```innate
(@vault-backup-burner){
  @files <- @vault/recent-exports{since: @last-backup-date}
  @disc <- @burn/new-disc{capacity-bytes: 700_000_000}

  @plan <- @burn/pack-files-ffd{files: @files, capacity: 700_000_000, discs: 3}
  @ui/show-plan{
    discs: @plan.discs,
    overflow: @plan.overflow,
    total-size: (sum @files.size-bytes)
  }

  @on-user-confirms{
    (for @disc-idx in 0..@plan.discs.length{
      @burn/open-session{disc: @disc}
      (for @f in @plan.discs[@disc-idx]{
        @iso-name <- @burn/canonicalize-iso-name{
          name: @f.name,
          existing: @session.track-names
        }
        @burn/add-file{disc: @disc, file: {name: @iso-name, size-bytes: @f.size-bytes}}
      })
      @burn/finalize{disc: @disc}
      @ui/prompt-eject-and-insert-next{}
    })
  }
}
```

The vault's backup shell is a thin wrapper: FFD computes the plan, user approves, and for each disc the wrapper opens a session, canonicalizes each filename against the already-placed names, adds the file, and finalizes.

## Structures

```innate
(defstruct burn-file
  name         : String
  size-bytes   : Int)

(defenum session-state OPEN | CLOSED)

(defstruct session
  tracks : [BurnFile]
  state  : SessionState)

(defenum disc-state BLANK | SESSION_OPEN | SESSION_CLOSED | FINALIZED)

(defstruct disc
  capacity-bytes : Int
  sessions       : [Session]
  state          : DiscState)

(defenum burn-error
  DISC_FINALIZED | NO_OPEN_SESSION | SESSION_ALREADY_OPEN | NOT_ENOUGH_SPACE)

(defstruct pack-result
  discs    : [[BurnFile]]
  overflow : [BurnFile])
```

## Resolver Natives

```innate
@burn/new-disc{capacity-bytes}                    -> Disc
@burn/open-session{disc}                          -> Unit | BurnError
@burn/add-file{disc, file}                        -> Unit | BurnError
@burn/close-session{disc}                         -> Unit | BurnError
@burn/finalize{disc}                              -> Unit | BurnError
@burn/used-bytes{disc}                            -> Int
@burn/remaining-bytes{disc}                       -> Int
@burn/pack-files-ffd{files, capacity, discs}      -> PackResult
@burn/canonicalize-iso-name{name, existing}       -> String
```

## Demo

```innate
(@demo){
  @d <- @burn/new-disc{capacity-bytes: 700_000_000}
  @burn/open-session{disc: @d}
  @burn/add-file{disc: @d, file: {name: "album1.flac", size-bytes: 100_000_000}}
  @burn/add-file{disc: @d, file: {name: "album2.flac", size-bytes: 150_000_000}}
  @d.used-bytes   ;; -> 285_000_000 (250MB + 35MB overhead)

  @burn/close-session{disc: @d}
  @burn/open-session{disc: @d}
  @burn/add-file{disc: @d, file: {name: "bonus.mp3", size-bytes: 50_000_000}}
  @d.sessions.length   ;; -> 2
  @d.used-bytes        ;; -> 370_000_000 (300MB + 70MB overhead for 2 sessions)

  @burn/finalize{disc: @d}
  @d.state   ;; -> FINALIZED

  @plan <- @burn/pack-files-ffd{
    files: [{name: "big.iso", size: 500_000_000},
             {name: "mid.mp4", size: 200_000_000},
             {name: "small.txt", size: 1_000_000}],
    capacity: 700_000_000,
    discs: 2
  }
  ;; plan.discs[0] = [big.iso, small.txt]  (500M + 1M, fits in 665M usable)
  ;; plan.discs[1] = [mid.mp4]              (200M)
  ;; plan.overflow = []

  @canonical <- @burn/canonicalize-iso-name{
    name: "Report.Document.Txt", existing: {}
  }
  ;; -> "REPORTDO.TXT"  (8 chars stem + 3 chars ext, collapsed periods)

  @canonical2 <- @burn/canonicalize-iso-name{
    name: "report.txt", existing: {"REPORT.TXT": true}
  }
  ;; -> "REPORT~1.TXT"  (6 stem + ~1 = 8 stem chars)
}
```

## Where

Capacity accounting MUST include session overhead — naive "bytes - sum(files)" over-counts usable space by one leadin+leadout per session; users will run out of room mid-burn. FFD MUST sort descending before placing — first-fit without sorting leads to fragmentation; decreasing-size means big files go first and small files fill gaps. ISO name canonicalization MUST uppercase and strip — CD file systems reject non-ISO chars; silently producing a bad name fails the burn. Collision suffix MUST truncate stem to make room — `REPORT.TXT` vs `REPORT~1.TXT` both fit 8.3; leaving the stem full would make the resolved name exceed 8 chars. Finalize MUST be terminal — physical finalization writes the lead-out; software that pretends a finalized disc can accept more sessions corrupts the index. Write ops MUST check `state == Finalized` first — every destructive op needs the same guard, or a bug in one skips the check. Sessions MUST track their own state (Open/Closed) — a closed session's track list is immutable; re-opening it would violate the disc's TOC.
