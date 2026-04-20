# G092 — Bulk Renamer and Organizer

> The Rosetta Stone's eighth **Files-category** project. The defining pattern: **preview, then apply, with undo**. Bulk renames are destructive — one typo in a regex can mangle a hundred files. G092 separates the non-destructive **preview** (compute what would change) from the destructive **apply** (actually do it), and makes every apply reversible via an **undo log**. This pattern is how every safe bulk operation works: database migrations, file managers, batch refactors.

```yaml
id: G092
title: Bulk Renamer and Organizer
category: files
requires: [G057-queue, G076-bookmarks, G087-file-explorer]
provides: [preview-then-apply, undo-log, collision-detection, atomic-batch-rename]
```

## Insight: Preview Is the Non-Destructive Surface

Every bulk operation has a **destructive phase**. Once you rename 100 files, reversing it is expensive if you can do it at all. The preview phase is read-only: it takes the same inputs (directory, rule) and produces the same output (list of rename ops) without touching anything. A GUI can show the user the preview, get confirmation, then apply.

This isn't a cosmetic choice. Preview must produce **exactly** what apply will do — same ops, same order, same error conditions. If preview misses a collision that apply hits, the user's trust in preview is destroyed. So preview and apply share the same underlying logic; apply is "run preview, then mutate".

First Rosetta Stone project where **the same logic must be executed twice** — once to show, once to do. G073's telnet had commands with effects; G084's captcha had verification as a single action. G092 explicitly separates "show me what would happen" from "do it", and makes that separation part of the API.

## Insight: Undo Log = Reverse Ops

When apply runs, it returns an **undo log** — a list of rename ops that, if applied, restore the original state. The log's structure is identical to the input ops, with `from` and `to` swapped. To undo: apply the log.

This is minimal. A richer undo system (multi-step history, redo) layers over the primitive. But the primitive is sufficient for every real tool's "I meant to rename `.txt` → `.md` but accidentally matched my `.tex` files too" scenario: preview was wrong, apply ran, undo immediately reverses.

First Rosetta Stone project where **mutation returns its own inverse**. G077's password safe had state changes but no inverse; G082's CMS had revisions as history but no undo primitive. G092 makes undo trivial: do, keep the log, apply the log to reverse.

## Insight: Collision Detection Happens Before Any Mutation

Two rename rules that collapse distinct filenames into the same target name is a catastrophic bug. `{foo, bar}` both rewriting to `baz` means one of them gets silently overwritten in most real filesystems. G092's preview detects this before any mutation and returns a `Collision` error.

There's a second kind of conflict: a rename whose target is **an existing untouched file**. `a.txt → b.txt` when `b.txt` already exists (and isn't being renamed in the batch) is also a collision. Preview catches it with the `ToExists` error.

Combined: the preview is a **total validator**. If it returns ops, apply is guaranteed to succeed. If it errors, the user gets a specific reason and can fix the rule.

First Rosetta Stone project where **validation is a separate data path from execution**. G085's parse errors happened during consumption; G083's template validation was standalone. G092 explicitly runs validation in the preview phase, and the apply phase trusts preview's output.

## Insight: Atomic Batch Rename Handles Circular Swaps

A rename batch can include ops like `{a → b, b → a}`. A naive "rename each op in sequence" approach fails: `a → b` leaves both `a` and `b` pointing at the same content; `b → a` then fails because `a` was just removed.

The atomic approach:
1. Remove all `from` names from the directory (captures state).
2. Install all `to` names.

Because removal happens first for all ops, circular swaps work. This is the same pattern SQL table renames use when swapping two tables (drop constraints, rename, rename, re-add constraints).

First Rosetta Stone project where **batch atomicity is explicit**. G068's thumbnails had per-item atomicity; G077's safe had transactional locking. G092 applies atomicity to batch mutation: the batch succeeds entirely or fails entirely, and circular dependencies inside the batch just work.

## Choreographic Case: Vault Photo Importer

```innate
(@vault-photo-importer){
  @photos <- @fs/ls{path: "Inbox/"}
  @dir <- @brn/directory{names: @photos.map(.name)}

  ;; Sequence rule: rename Inbox/IMG_001.jpg → photo_001.jpg ... etc.
  @rule <- @brn/sequence{template: "photo_{n}.jpg"}
  @ops <- @brn/preview{dir: @dir, rule: @rule}
  @ui/show-preview{ops: @ops}

  @on-user-confirms {
    @undo-log <- @brn/apply{dir: @dir, ops: @ops}
    @for op in @ops {
      @fs/rename{from: "Inbox/${op.from}", to: "Photos/${op.to}"}
    }
    @vault/save{path: ".vault/last-undo.log",
                 content: @log/to-text{log: @undo-log}}
  }

  @on-user-clicks-undo {
    @log <- @log/from-text{text: @vault/read-string{path: ".vault/last-undo.log"}}
    @brn/undo{dir: @dir, log: @log}
    ;; ... reverse the filesystem operations
  }
}
```

The vault's photo importer shows the user what would happen, waits for confirm, applies, and saves the undo log. Users can always get back to the original state.

## Structures

```innate
(defenum rule-kind REPLACE | PREFIX | SUFFIX_BEFORE_EXT | SEQUENCE)

(defstruct rule
  kind    : RuleKind
  find    : String
  replace : String
  prefix  : String
  suffix  : String
  template: String)

(defstruct rename-op
  from : String
  to   : String)

(defstruct directory
  entries : {String})    ;; set of names
```

## Resolver Natives

```innate
@brn/directory{names}                -> Directory
@brn/replace{find, replace}          -> Rule
@brn/prefix{prefix}                  -> Rule
@brn/suffix{suffix}                  -> Rule          ;; before extension
@brn/sequence{template}              -> Rule          ;; {n} replaced by index+1
@brn/preview{dir, rule}              -> [RenameOp] | RenameError
@brn/apply{dir, ops}                 -> [RenameOp] | RenameError     ;; undo log
@brn/undo{dir, log}                  -> Unit | RenameError
```

## Demo

```innate
(@demo){
  @d <- @brn/directory{names: ["img_01.png", "img_02.png", "img_03.png", "notes.txt"]}

  @rule <- @brn/replace{find: "img_", replace: "photo_"}
  @ops <- @brn/preview{dir: @d, rule: @rule}
  ;; -> [{img_01.png → photo_01.png}, {img_02.png → photo_02.png}, {img_03.png → photo_03.png}]

  @undo-log <- @brn/apply{dir: @d, ops: @ops}
  @brn/directory-names{dir: @d}
  ;; -> [notes.txt, photo_01.png, photo_02.png, photo_03.png]

  @brn/undo{dir: @d, log: @undo-log}
  @brn/directory-names{dir: @d}
  ;; -> [img_01.png, img_02.png, img_03.png, notes.txt]
}
```

## Where

Preview MUST produce exactly what apply will do — the user's trust depends on the two being synchronous, and any divergence makes preview worse than useless. Collisions (two sources → same target) MUST fail the whole preview, NOT silently pick one winner — ambiguity in a bulk operation is the most dangerous kind of bug. Target-exists MUST fail unless the target is also being renamed — a `a→b` when `b` exists and isn't in the batch is a silent overwrite waiting to happen. Apply MUST be atomic: all ops succeed or none do. Apply MUST remove all sources before installing any targets, so circular swaps (`a→b, b→a`) work. Undo MUST be exact: applying the undo log restores the exact original directory state with no drift. Sequence rule MUST use 1-based indexing — users expect "first file = 1", not "first file = 0".
