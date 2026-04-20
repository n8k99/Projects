# G100 — Versioning Manager

> The Rosetta Stone's sixteenth **Files-category** project — and the one that **closes the category**. Introduces the **content-addressed storage** model that Git, IPFS, Nix, Mercurial, and every hash-indexed version system uses. A repo is three content-addressed stores (blobs, trees, commits) plus a HEAD pointer; commits form a linked history by referencing their parent's hash; identical content collapses to one blob, so the storage is automatically deduplicated. **Cross-language hash equivalence** is the Rosetta Stone's most ambitious contract — six languages all producing the exact same FNV-1a hex hashes for the exact same canonical bytes.

```yaml
id: G100
title: Versioning Manager
category: files
requires: [G082-cms, G090-zip-file-maker, G092-bulk-renamer, G096-rpg-character]
provides: [content-addressed-storage, hash-deduped-blobs, commit-chain-history, canonical-encoding-cross-language]
```

## Insight: Content Addressing Is the Dedupe Primitive

A blob's identity **is its content hash**. Same bytes → same hash → same blob. Two commits that both contain the file `hello.txt` with content `"hello world"` point to the **same blob object** — no copies made. The store deduplicates automatically; there's no "check if this already exists" logic because the hash already answered that question.

This is the same idea powering Git's object store, IPFS's entire protocol, Nix's package cache, Docker image layers, and every system that wants "identical things stored once". G100 reduces it to three tables (blob, tree, commit) keyed by hash.

First Rosetta Stone project where **the identifier for a piece of data is derived from the data itself**. G093's tag store keyed by path (external); G094's log entries keyed by insertion order; G100's objects are keyed by their content hash.

## Insight: The Commit Chain Is a Linked List of Trees

A commit has a tree hash + parent hash. The tree hash is "what files existed at this moment"; the parent hash is "what commit came before". Walking parent pointers from HEAD reconstructs the full history — that's what `git log` does.

G100's log is a linear chain (one parent per commit). Real Git allows multiple parents (merges), which is the DAG generalisation. The Rosetta Stone version is deliberately simpler — the pattern of "DAG of content-addressed snapshots" is established; branching/merging is layered on top in a production system.

First Rosetta Stone project with **a deliberate simplification of a well-known system**. G079's text game didn't try to be Zork; G094's log didn't try to be syslog. G100 picks the subset of Git that fits the category: the content-addressed object store and linear history. Full Git is a tractable extension.

## Insight: Canonical Encoding Is the Cross-Language Contract

For the hash of a commit to match across six languages, **the byte sequence fed into the hash function must match byte-for-byte**. That means:
* Tree entries must be sorted by filename (BTreeMap in Rust, sorted dict in Python, etc).
* The canonical format is literal: `"tree\n"`, `"{name}\t{hash:016x}\n"` per entry.
* Hex is lowercase. Newlines are `\n`. No trailing spaces.
* The commit format is: `"commit\n"`, `"tree {hex}\n"`, optional `"parent {hex}\n"`, `"ts {num}\n"`, `"msg {str}\n"`.

Every language must produce these exact bytes. The Rosetta Stone test is: commit the same files in Python and Rust with the same timestamp and message; both produce the same commit hash. This is a **behavioural equivalence test** at the bit level, the strongest cross-language contract the milestone has.

First Rosetta Stone project where **all six languages must produce identical bytes** from the same input. G085 had textual round-trip; G090 had byte-level round-trip within one language; G100 requires byte-level equivalence across six languages via canonical encoding.

## Insight: Blob Storage Is Deduplicated by Construction

Two files with the same content in the same commit create **one blob**. Two commits that both contain the same content reuse that one blob across all commits. The repo's `blob_count()` grows only when truly-new content arrives.

The Rosetta Stone test cases verify this: a commit with ten files all containing `"hello"` produces one blob, not ten. Committing the same file a second time produces zero new blobs (the tree still points at the existing blob). This is where Git's "your whole history fits in the repo size" efficiency comes from — the more repetition, the tighter the storage.

First Rosetta Stone project where **storage efficiency is an emergent property of the data model**, not an explicit optimisation. G089 kept integer cents for accounting correctness; G100 uses content addressing for natural dedup. Both are cases where choosing the right representation makes the hard problem go away.

## Insight: Diff Is Tree Comparison, Not Byte Comparison

`diff(from, to)` compares the two commits' **tree entries**, not their blob contents. If a file's hash differs between trees, it's marked `Changed`. If a name exists in one tree but not the other, it's `Added` or `Removed`. This is O(n) in the number of tree entries, not O(total bytes).

That's the same speedup Git gets from its object model: `git diff` is tree comparison; the blob contents are only fetched for files that changed. Real Git layers line-by-line blob diff on top, but the file-level diff (`git diff --name-status`) works purely from trees.

First Rosetta Stone project where **a comparison operation exploits content addressing for speed**. Two commits are identical iff their trees hash identically — a single 64-bit comparison replaces a full file-by-file walk. This is the kind of algorithmic win that makes content-addressed storage worth its complexity.

## Choreographic Case: Vault History

```innate
(@vault-history){
  @repo <- @vm/new-repo{}
  @on-vault-save (@path @content @message){
    ;; Gather all currently-tracked files and their current bytes.
    @files <- @vault/list-tracked{}.map(@f => [@f.path, @vault/read-bytes{path: @f.path}])
    @files <- @files.with([@path, @content])  ;; override one
    @vm/commit{repo: @repo, files: @files, message: @message, timestamp: @clock/now}
  }

  @on-user-asks-history (@file){
    @commits <- @vm/log{repo: @repo}.filter(@c =>
      @vm/checkout{repo: @repo, commit-hash: @c.hash}.has-key(@file))
    @ui/render-history{commits: @commits}
  }

  @on-user-restores (@commit-hash){
    @files <- @vm/checkout{repo: @repo, commit-hash: @commit-hash}
    @for (path, bytes) in @files {
      @vault/write-bytes{path: @path, content: @bytes}
    }
  }
}
```

The vault's file history view is a repo; every save commits; restore checks out a historical commit and writes files back. No external Git invocation — the versioning primitive lives inside the vault's own engine.

## Structures

```innate
(defstruct blob
  data : Bytes)

(defstruct tree
  entries : {String -> Hash})   ;; sorted by key

(defstruct commit
  tree-hash    : Hash
  parent       : Hash?
  message      : String
  timestamp-ms : Int)

(defenum diff-kind ADDED | REMOVED | CHANGED)

(defstruct diff-op
  kind      : DiffKind
  name      : String
  from-hash : Hash
  to-hash   : Hash)

(defstruct repo
  blobs    : {Hash -> Blob}
  trees    : {Hash -> Tree}
  commits  : {Hash -> Commit}
  head     : Hash?)
```

## Resolver Natives

```innate
@vm/fnv1a-hash{bytes}                           -> Hash
@vm/new-repo{}                                  -> Repo
@vm/commit{repo, files, message, timestamp}     -> Hash
@vm/checkout{repo, commit-hash}                 -> {String -> Bytes} | null
@vm/diff{repo, from, to}                        -> [DiffOp] | null
@vm/log{repo}                                   -> [(Hash, Commit)]
@vm/blob-count{repo}                            -> Int
@vm/commit-count{repo}                          -> Int
```

## Demo

```innate
(@demo){
  @r <- @vm/new-repo{}
  @c1 <- @vm/commit{repo: @r, files: [["a.txt", "hello"], ["b.txt", "world"]],
                     message: "initial", timestamp: 1000}
  @c2 <- @vm/commit{repo: @r, files: [["a.txt", "hello!"], ["b.txt", "world"],
                                       ["c.txt", "new"]],
                     message: "update a, add c", timestamp: 2000}
  @vm/diff{repo: @r, from: @c1, to: @c2}
  ;; -> [Changed(a.txt, ...), Added(c.txt, ...)]

  @vm/checkout{repo: @r, commit-hash: @c1}
  ;; -> {a.txt: "hello", b.txt: "world"}  (historical state restored)

  @vm/blob-count{repo: @r}   ;; -> 4  (hello, world, hello!, new)
}
```

## Where

Identifier MUST be derived from content — `hash(data) == hash(same-data)` across any language, always. Canonical encoding MUST be byte-identical across all six languages — hex is lowercase, newlines are `\n`, field separators are `\t`, fields are sorted. Blob dedup MUST be automatic — putting the same bytes twice MUST yield the same hash and MUST NOT duplicate storage. The commit chain MUST be discoverable from HEAD — walking parents reconstructs the full linear history. Tree diff MUST work on hash equality — if two trees have the same hash, they're identical with zero file reads needed. Empty repos MUST have no HEAD and produce an empty log — initialised but never committed is a valid state, not an error. Checkout of an unknown hash MUST return null, NOT crash — callers need to test-and-use, and crashes on missing commits turn UI bugs into panics.

## Closing the Files Category

G100 closes **Files**. Sixteen projects, from the round-trip-equivalence opener of G085 Quiz Maker to G100's content-addressed version store. The category's journey:

* **Round-trip foundation** (G085 Quiz, G086 Launcher, G087 Explorer) — write a file, read it back unchanged.
* **Tabular and aggregated** (G088 Sort, G089 Transaction Averages) — uniform records, sorting, grouping.
* **Compression and documents** (G090 Zip, G091 PDF) — binary round-trip, flow layout.
* **Batch operations** (G092 Rename, G098 File Copy) — preview/apply/undo, policy-as-data.
* **Metadata and queryable stores** (G093 Mp3 Tags, G094 Logger) — metadata/data separation, append-only.
* **Spreadsheet-style derivations** (G095 Excel, G096 RPG Character) — lazy evaluation, derived stats.
* **Shape-polymorphic clickable regions** (G097 Image Map) — predicate polymorphism, ray casting.
* **Snippet and version stores** (G099 Snippets, G100 Versioning) — tab-stop IR, content-addressed storage.

Sixteen patterns that every file-format-adjacent codebase draws on. The noosphere's own file handling (vault files, save states, binary assets, backups, git-style history) will compose these patterns rather than invent new ones.

**Files 16/16 closed. 100/130 complete.** Remaining: Databases (G101–G113) and Graphics (G114–G130) — 30 projects to close the milestone.
