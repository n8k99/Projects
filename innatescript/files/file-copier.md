# G098 — File Copy Utility

> The Rosetta Stone's fourteenth **Files-category** project. Building on G087's virtual filesystem, G098 introduces **recursive tree copy with explicit policy**. A straightforward `cp -r` is three decisions stitched together: **overwrite policy** (skip / overwrite / rename-with-suffix), **filter** (include / exclude by substring), and **progress events** (one per item processed). G098 treats all three as first-class parameters, not defaults baked into the loop.

```yaml
id: G098
title: File Copy Utility
category: files
requires: [G087-file-explorer, G092-bulk-renamer]
provides: [recursive-subtree-copy, overwrite-policy-as-data, filter-as-predicate, event-log-progress]
```

## Insight: Copy Is Three Orthogonal Concerns

A naive `copy_tree(src, dest)` conflates:
1. **What to traverse** (the source subtree).
2. **What to include** (the filter).
3. **How to resolve conflicts** (the policy).
4. **How to report progress** (the event stream).

G098 separates them. `copy_tree(src_root, dest_root, policy, filter) → events`. Each concern is a first-class parameter or return value. Changing one doesn't require understanding the others.

This is the **separation of mechanism from policy** principle — rsync, robocopy, and every modern file-sync tool is organised around it. The mechanism (walk the tree, copy items) is fixed; the policy (what to do with collisions) and the filter (what to include) are configurable.

First Rosetta Stone project where **a single operation takes a policy enum and a filter struct as inputs**. G092 took one rule at a time; G098 takes a rule plus a filter, and its rule is one of several named variants.

## Insight: Overwrite Policy Is Three Named Cases

Every real copy tool has the same three choices:
* **Skip** — don't touch existing targets.
* **Overwrite** — replace existing targets.
* **Rename with suffix** — keep both; add `.1`, `.2`, etc.

G098 models them as an enum. The copy loop branches on the enum when a collision is detected; the branches are small (one event + one mutation each). Adding a fourth policy ("interactive: ask the user") would be one variant plus one callback — the structure stays flat.

First Rosetta Stone project where **a user-facing policy decision is explicit data**. G092's rules were action shapes; G098's `Overwrite` is "what to do when the action would conflict". Orthogonal concerns, both reified.

## Insight: Filter Is a Pure Predicate

`filter.accepts(path) → bool`. Given a path, return whether to process it. Include patterns require at least one match; exclude patterns require no matches. Both use simple substring — no regex, no glob translation. Pure function, no state, no side effects.

The filter is applied before the copy decision, so a filtered-out item never reaches the policy branch. That keeps policy logic from worrying about "is this item allowed?" — the filter already said yes.

First Rosetta Stone project where **a filter is a separate object with its own API**. Previous filtering was inline (G094's log queries) or embedded (G093's query AST). G098's `Filter` is a small struct with one method; it composes with the copier by being passed as an argument.

## Insight: Events Are the Replay Log

Every item processed produces an event: `Copied` / `Skipped` / `Renamed` / `CreatedDir`, each with source path, destination path, and bytes. The event list is returned to the caller, who can:
* **Sum bytes copied** for progress bars (`total_size_copied(events)`).
* **Count skipped items** for "you have N files already" warnings.
* **Replay the events** to reconstruct what happened after the fact.
* **Serialise** the events to a log file for audit trails.

Events are **not print output** and **not callbacks** — they're data. The caller chooses what to do with them. This is the same pattern Kafka, Git's reflog, Postgres's WAL, and every event-sourced system follows.

First Rosetta Stone project where **the operation's side effects are captured as a value**. G094's logger captured domain events; G098 captures operation events. Both return data that can be processed, not discarded.

## Insight: Rename-With-Suffix Handles Cascading Collisions

If `dest/a.txt` exists and the policy is rename, the copy goes to `dest/a.txt.1`. If *that* also exists (from a previous copy), `dest/a.txt.2`. And so on. `next_available_name(items, base)` loops until it finds a free slot.

This is the same algorithm every browser uses for downloads (`cover.jpg`, `cover (1).jpg`, `cover (2).jpg`), every email client for attachments, every file manager for drag-drop-to-same-folder. G098 picks the `.N` suffix convention; other conventions (`(N)`, `-copy`, date-stamp) layer over the same search logic.

First Rosetta Stone project with **cascading name resolution**. G092's bulk renamer had collision detection but rejected collisions; G098 resolves them by generating new names until a free slot is found.

## Choreographic Case: Vault Export

```innate
(@vault-export){
  @fs <- @fs/open{path: "~/Documents/Droplet-Org"}
  @events <- @fc/copy-tree{
    fs: @fs,
    src-root: "The Work",
    dest-root: "export/The Work",
    policy: "overwrite",
    filter: {include: [], exclude: [".tmp", ".cache"]}
  }
  @bytes <- @fc/total-size-copied{events: @events}
  @ui/render-progress{bytes: @bytes, events: @events}

  @log <- @events.map(@e => @log/format{event: @e})
  @vault/save{path: ".vault/last-export.log", content: @log.join("\n")}
}
```

Export copies a subtree while excluding tmp/cache files; the event list feeds both a progress UI and an audit log. Users can replay the log to see exactly what moved.

## Structures

```innate
(defenum overwrite SKIP | OVERWRITE | RENAME_WITH_SUFFIX)
(defenum event-kind COPIED | SKIPPED | RENAMED | CREATED_DIR)

(defstruct filter
  include : [String]
  exclude : [String])

(defstruct copy-event
  kind   : EventKind
  source : String
  dest   : String
  bytes  : Int)
```

## Resolver Natives

```innate
@fc/new-fs{}                                              -> Fs
@fc/add-file{fs, path, size}                              -> Unit
@fc/add-dir{fs, path}                                     -> Unit
@fc/filter{include, exclude}                              -> Filter
@fc/filter-allow-all{}                                    -> Filter
@fc/copy-tree{fs, src-root, dest-root, policy, filter}    -> [CopyEvent] | Error
@fc/total-size-copied{events}                             -> Int
```

## Demo

```innate
(@demo){
  @fs <- @fc/new-fs{}
  @fc/add-dir{fs: @fs, path: "src"}
  @fc/add-file{fs: @fs, path: "src/a.txt", size: 100}
  @fc/add-file{fs: @fs, path: "src/b.txt", size: 200}
  @fc/add-dir{fs: @fs, path: "src/sub"}
  @fc/add-file{fs: @fs, path: "src/sub/c.txt", size: 50}

  @events <- @fc/copy-tree{
    fs: @fs, src-root: "src", dest-root: "dest",
    policy: "overwrite", filter: @fc/filter-allow-all{}
  }
  @fc/total-size-copied{events: @events}
  ;; -> 350
}
```

## Where

Copy MUST take policy as a parameter, NOT hardcode one — every copy tool in existence needs all three policies, and forcing users to wrap the call is worse UX than passing an enum. Filter MUST be a pure predicate, applied before policy check — mixing filter and policy makes both harder to reason about. Events MUST be returned as data, NOT printed or called back — event lists can be re-processed, logged, tested; stdout prints cannot. RenameWithSuffix MUST cascade (`.1`, `.2`, ...) until a free slot exists — stopping at `.1` when it's also occupied would silently fail. Overwriting a target that is an untouched file is ALLOWED under OVERWRITE policy — the user asked for it. Skipped items MUST still produce an event — the caller needs to know *what* was skipped, not just that skips happened. Empty filter (no include, no exclude) MUST accept everything — this is the default and must be the "not restrictive" case.
