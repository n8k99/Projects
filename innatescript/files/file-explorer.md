# G087 — File Explorer

> The Rosetta Stone's third **Files-category** project. Opens the core file-manager primitive: **path resolution**. User input (`../docs`, `/home/./x`, `..`) becomes a canonical segment list. Introduces the virtual-filesystem abstraction used by every subsequent Files project. Establishes **root escape is impossible** as a hard invariant.

```yaml
id: G087
title: File Explorer
category: files
requires: [G047-dictionary, G056-tree, G086-quick-launcher]
provides: [path-resolution, virtual-filesystem, root-escape-prevention, dirs-first-listing]
```

## Insight: Path Resolution Is the Filesystem's Type Checker

Every file operation (`cd`, `ls`, `read`, `write`) starts with a string the user typed. Turning that string into a location the filesystem understands is the single most important job a file manager does. Get it wrong and the user ends up somewhere unexpected, can't find their file, or — worst case — escapes the root and accesses data outside the intended scope.

`resolve(path)` takes user input and returns a canonical segment list: absolute paths start from root, relative paths start from cwd, `.` and empty segments are dropped, `..` pops one segment, and `..` at root is a no-op. The output is type-correct — a list of plain segment names with no metacharacters. Every subsequent operation takes that canonical list, not the raw string.

First Rosetta Stone project where **parsing and semantic analysis are separate phases**. The quiz parser (G085) and launcher parser (G086) turned text into structures in one step; the file explorer separates `resolve(path) -> segments` from `lookup(segments) -> node`. Splitting these is what makes root escape impossible — the normaliser sees `..` and does the right thing before lookup runs.

## Insight: Virtual Filesystem Is the Right Test Surface

Real disk I/O is slow, nondeterministic, and depends on the host machine. The Rosetta Stone needs implementations that produce byte-identical output across six languages; real-disk behaviour doesn't satisfy that. So the explorer operates on a **tree of Node objects in memory**, no file descriptors, no syscalls.

This is the same trade every test suite makes: mock the side effects, test the logic. But G087 goes further — the virtual filesystem isn't just for tests, it's the production abstraction. The explorer doesn't know or care whether its data lives in RAM, on disk, in Postgres, or in a vault markdown file. Later projects can plug in a storage backend; G087's logic is storage-agnostic.

First Rosetta Stone project where **the data model is an internal tree, not a reflection of the OS**. The vault's file views (Projects widget, Kanban, Daily Note browser) will all layer over this abstraction — what the user sees as a "folder" might be a SQL query, a glob pattern, or a real directory.

## Insight: Dirs First, Alphabetical Second

Every file manager ever shipped has some presentation order. Naive alphabetical mixes folders and files (`docs/`, `notes.md`, `Pictures/`, `README.md`) and makes it hard to scan. **Dirs first, then alphabetical within each group** is the convention — folders cluster at top, files below, each in a stable order.

The listing comparator is two-stage: first sort by kind (dir < file), then alphabetical within each kind. One pass, O(n log n), stable. Every test that reads listings gets the same order on every language.

First Rosetta Stone project with a **category-before-alphabetical** sort. G076 bookmarks used axis-based sorting; G087 uses kind-based grouping. Same pattern: partition first, then order within partitions.

## Insight: Recursive Size Is a Fold

`total_size(path)` on a directory walks every descendant file and sums. It's the same fold pattern as scoring (G085) — visit each leaf, combine with an accumulator. For a dir, fold over children; for a file, return size. Two-line recursion, works on arbitrary tree depth.

First Rosetta Stone project that **recurses over a non-linear structure**. G064 family tree did transitive closure on a DAG; G087 is a pure tree, simpler. But the pattern — a self-recursive function that dispatches on node kind — shows up again in G089 (walking file records), G097 (image maps as nested regions), and throughout Graphics (G114+).

## Choreographic Case: Vault File Pane

```innate
(@vault-file-pane){
  @explorer <- @fs/load-vault{path: "~/Documents/Droplet-Org"}

  @on-user-types-path (@path){
    @resolved <- @fs/resolve{explorer: @explorer, path: @path}
    @listing <- @fs/ls{explorer: @explorer, path: @path}
    @ui/render-path{resolved: @resolved, listing: @listing}
  }

  @on-user-selects-entry (@name){
    @full-path <- @fs/join{cwd: @explorer.cwd, name: @name}
    @stat <- @fs/stat{explorer: @explorer, path: @full-path}
    @when (@stat.kind == "dir"){ @fs/cd{explorer: @explorer, path: @name} }
    @when (@stat.kind == "file"){ @ui/open-file{path: @full-path} }
  }
}
```

The vault's file pane is a thin renderer over the explorer. User types path → explorer resolves and lists → UI draws. User clicks entry → explorer dispatches on kind → navigate or open. The logic is in the explorer; the UI is oblivious to paths.

## Structures

```innate
(defenum kind DIR | FILE)

(defstruct node
  kind     : Kind
  size     : Int
  mtime    : Int
  children : {String -> Node})

(defstruct explorer
  root : Node
  cwd  : [String])

(defstruct listing
  name  : String
  kind  : Kind
  size  : Int
  mtime : Int)
```

## Resolver Natives

```innate
@fs/new{}                                   -> Explorer
@fs/cwd{explorer}                           -> String
@fs/resolve{explorer, path}                 -> [String] | null
@fs/cd{explorer, path}                      -> Unit | FsError
@fs/ls{explorer, path}                      -> [Listing] | FsError
@fs/mkdir{explorer, path}                   -> Unit | FsError
@fs/touch{explorer, path, size, mtime}      -> Unit | FsError
@fs/stat{explorer, path}                    -> Listing | FsError
@fs/total-size{explorer, path}              -> Int | FsError
```

## Demo

```innate
(@demo){
  @e <- @fs/new{}
  @fs/mkdir{explorer: @e, path: "/home"}
  @fs/mkdir{explorer: @e, path: "/home/nathan"}
  @fs/mkdir{explorer: @e, path: "/home/nathan/docs"}
  @fs/touch{explorer: @e, path: "/home/nathan/docs/a.txt", size: 100, mtime: 1000}
  @fs/touch{explorer: @e, path: "/home/nathan/notes.md", size: 50, mtime: 500}

  @fs/cd{explorer: @e, path: "/home/nathan/docs"}
  @fs/cd{explorer: @e, path: "../.."}
  @fs/cwd{explorer: @e}                       ;; -> "/home"

  @fs/resolve{explorer: @e, path: "/a/./b/../c"}   ;; -> ["a", "c"]

  @fs/ls{explorer: @e, path: "/home/nathan"}
  ;; -> [{name:"docs", kind:"dir"}, {name:"notes.md", kind:"file", size:50}]

  @fs/total-size{explorer: @e, path: "/home"}      ;; -> 150
}
```

## Where

`resolve` MUST separate path normalisation from lookup — `..` handling belongs to the normaliser, not the traversal, because that split is what makes root escape impossible. `..` at root MUST be a no-op, NOT an error — users type `cd ..` compulsively at root and the UX must not punish them. Listings MUST be dirs-first then alphabetical — every file manager in existence follows this convention and violating it makes the UI feel wrong. `cd` on a file MUST return `NotADir`, NOT silently succeed — the cwd invariant (cwd always points to a directory) must hold for all later operations. `mkdir` on an existing path MUST return `AlreadyExists`, NOT silently succeed or overwrite — the user expected to create something new; an ambiguous no-op hides bugs. `total_size` on a single file MUST return that file's size, not zero or an error — the contract is "sum of reachable leaves" and a file is a reachable leaf.
