# G069 — WYSIWYG Editor

> The Rosetta Stone's first **web-category** project. The essence of WYSIWYG is that **the document model IS the render model** — no distinction between "source" and "output," no parsing step between editing and displaying. This is the first project in the milestone where the data structure is itself the thing the user sees.

```yaml
id: G069
title: WYSIWYG Editor
category: web
requires: [G030-text-to-html-generator, G056-image-gallery]
provides: [run-based-text-model, edit-is-render, range-operations, attribute-composition, run-canonicalization]
```

## Insight: Edit IS Render — No Intermediate Representation

Every text format before WYSIWYG had a source form that differed from the rendered form. LaTeX source is not what the PDF looks like; HTML source is not the web page; Markdown is not its preview. A separate *parser* lifted the source into a render tree, and a separate *renderer* projected the tree back down to pixels.

WYSIWYG is the philosophical opposite: **the model you edit IS the model that displays**. There is no source form; there is no parse step; the user manipulates the display directly and the underlying representation is what they see. This is the first Rosetta Stone project where that property is the *point*.

Markdown-native systems (including the vault itself) live in the older "source is a projection" world — Obsidian edits Markdown text, which is rendered separately. True WYSIWYG systems (Google Docs, Notion's editor, rich text fields in most CMSes) store structured formatted text and never parse it from a source form. G069 shows the minimum structure needed to support this: a list of **runs**, each carrying its own attributes, manipulated by operations that preserve the run structure directly.

## Insight: Runs Are the Right Granularity

Character-by-character attribute storage is correct but wasteful. Region-by-region storage (store ranges like `bold: [0..5], [12..20]`) is compact but hard to update. The middle path — **runs** — stores each contiguous span of uniformly-formatted text once, with the attribute set, and reshapes the runs as operations demand.

A run's properties:
- All characters in a run have the same attribute set.
- Adjacent runs have *different* attribute sets (otherwise they'd merge).
- The concatenation of all run texts is the document's plain text.

These three invariants must hold after every operation. `normalize` is the pass that re-establishes them — drop empty runs, merge adjacent runs with equal attrs. It runs after every mutation. G069 is the first Rosetta Stone project where **a normalization pass is a mandatory part of the API contract**, not an optimisation.

## Insight: Range Operations Reshape Runs

User-facing operations work on character ranges (`bold positions 5..12`). The implementation:

1. Splits runs at both range boundaries (creating new run boundaries if necessary).
2. Modifies the attribute sets of runs entirely inside the range.
3. Re-normalises to merge any adjacent runs that now have identical attrs.

The user thinks in positions; the storage lives in runs; `split_at` is the bridge. First Rosetta Stone project where the **user coordinate system and the storage coordinate system are deliberately different** and an operation translates between them.

Parallels everywhere:
- Spreadsheet cells reference (row, col) but storage is sparse maps.
- DOM selection references character offsets; the DOM itself is a tree.
- Git operates on lines from the user's view but stores content-hashed blobs.
- Vault wiki-link resolution uses human-written `[[path/to/note]]` but resolves against a graph.

G069 is the minimal case where the translation is the core operation.

## Insight: Attributes Compose as Sets

Applying bold to a region that is already bold-and-italic must preserve the italic. Attributes compose as **set union**, not as scalar overwrite. Removing bold from bold-italic text leaves italic.

This is the first Rosetta Stone project where **orthogonal attributes combine on the same data**. G056 had tags as set-algebra, but tags were a single dimension (tag sets combined with tag sets). G069 has many orthogonal dimensions on the same character: bold, italic, underline, heading level, link href, color, font. Each is independent; each composes.

Most real text formats get this wrong. CSS `font-weight: bold` conflicts subtly with `font-weight: bold italic` (the latter is a font name, not a composition). HTML `<b>` nested inside `<strong>` produces ambiguous semantics. WYSIWYG done right stores the attributes as a set and renders them to whatever the output format requires — G069 renders to HTML, but the same model renders to RTF, DOCX, Markdown with extensions, or a terminal's ANSI codes. **The model is the source of truth; every render format is a projection.**

## Insight: Rendering Is Just Another Traversal

Given the document is already structured, `to_html` is a walk: for each run, emit the opening tags for its attrs, then the (escaped) text, then the closing tags in reverse order. There is no parsing. The render function is one-way: `Document → String`. Unlike source-to-render pipelines, there is no round-trip to implement.

`to_plain` is an even simpler walk: concatenate all run texts, skipping the attrs entirely. `to_markdown` (not implemented here, but mechanical) would be the same walk with different tag syntax. Every output format is a 20-line function because the model has already done the hard work.

This is the Rosetta Stone's first case of **the render as a trivial projection of the model**. In G058 Chart Making, render was a pipeline stage. Here it is a single walk. The difference is that G058's model was data (values to chart) while G069's model is already *structured display intent*.

## Insight: Opens the Web Category — Structure Over Syntax

The Web category (G069–G084) is primarily about **interfaces that render meaningful output users interact with directly**. Every project in the category will have some form of structured model that projects to an output format. G069 sets the convention:

- Model is the source of truth (runs with attrs).
- Operations mutate the model, never the rendered output.
- Rendering is a projection, not a round-trip.
- User coordinates differ from storage coordinates.

Subsequent Web projects — page scraper, content management system, template maker — all inherit this shape: structured internal model, render function, coordinate-translation at the API boundary.

## Choreographic Case: Note Editor With Live Preview

```innate
(@vault-note-editor){
  @doc <- @wysiwyg/new
  @doc <- @wysiwyg/insert{doc: @doc, pos: 0,
                          text: @note/content, attrs: []}

  @on-user-bold-selection {
    @doc <- @wysiwyg/apply-style{
      doc: @doc,
      start: @selection.start,
      end: @selection.end,
      attr: "bold"
    }
    @render/live-preview{html: @wysiwyg/to-html{doc: @doc}}
    @vault/persist{path: @note.path, runs: @doc.runs}
  }
}
```

The editor holds the document; every edit operation produces a new document; rendering happens whenever the display needs to refresh. There is no parsing, no source-form round-trip, no "save then reload to see changes." The model is the editor is the render.

## Structures

```innate
(defstruct run
  text   : String
  attrs  : Set<Attr>)        ;; Bold | Italic | Underline | H1 | H2 | ...

(defstruct document
  runs   : [Run])            ;; adjacent runs MUST have different attrs (invariant)

(defenum attr
  Bold | Italic | Underline | H1 | H2)
```

## Resolver Natives

```innate
@wysiwyg/new                                                 -> Document
@wysiwyg/length{doc}                                         -> Int
@wysiwyg/insert{doc, pos, text, attrs}                       -> Document
@wysiwyg/delete{doc, start, end}                             -> Document
@wysiwyg/apply-style{doc, start, end, attr}                  -> Document
@wysiwyg/remove-style{doc, start, end, attr}                 -> Document
@wysiwyg/to-plain{doc}                                       -> String
@wysiwyg/to-html{doc}                                        -> String
```

## Demo

```innate
(@demo){
  @d <- @wysiwyg/new
  @d <- @wysiwyg/insert{doc: @d, pos: 0,
                        text: "Hello, world! Welcome to WYSIWYG.", attrs: []}
  @d <- @wysiwyg/apply-style{doc: @d, start: 0, end: 6, attr: Bold}
  @d <- @wysiwyg/apply-style{doc: @d, start: 7, end: 12, attr: Italic}
  @d <- @wysiwyg/apply-style{doc: @d, start: 14, end: 21, attr: Underline}
  @d <- @wysiwyg/apply-style{doc: @d, start: 25, end: 32, attr: Bold}
  @wysiwyg/to-html{doc: @d}
  ;; -> <b>Hello,</b> <i>world</i>! <u>Welcome</u> to <b>WYSIWYG</b>.
}
```

## Where

Adjacent runs MUST NOT share attribute sets — the normalization pass after every operation enforces this, and any operation that leaves two adjacent runs with equal attrs has produced a denormalised document that is a bug, not a valid state. Empty runs MUST be dropped by normalization. Position arguments are character offsets (user-facing), not run indices; translating between coordinate systems is the library's responsibility and the user MUST never see run internals. Apply-style MUST be a set union (already-bold stays bold, now also italic); remove-style MUST be a set removal (italic-only becomes plain, bold-italic becomes italic). The rendered output MUST be a pure function of the document — no state, no context, same document always renders the same HTML. HTML special characters (`<`, `>`, `&`) in the document text MUST be escaped in the rendered output.
