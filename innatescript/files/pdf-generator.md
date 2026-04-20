# G091 — PDF Generator

> The Rosetta Stone's seventh **Files-category** project. The defining problem of PDF generation is **paginating a linear content stream into fixed-size pages**. Every real PDF library (iText, ReportLab, wkhtmltopdf, LaTeX) solves this problem; G091 extracts the kernel. Skips binary PDF output entirely — a **rendered document model** (pages + placed lines) is the abstraction a renderer needs. Any output format (PDF bytes, HTML, PostScript, terminal) layers on top.

```yaml
id: G091
title: PDF Generator
category: files
requires: [G069-wysiwyg-editor, G081-ecard, G088-sort-file-records]
provides: [pagination-via-flow-layout, word-wrap, rendered-document-model, renderer-independence]
```

## Insight: Pagination Is Flow Layout, Nothing More

The kernel of PDF generation is trivial:
1. Start with `y = 0` on a fresh page.
2. For each block, compute its height. If it fits, place it at `y` and advance. If not, emit the page and start fresh.
3. Some blocks (headings, images) may force a break early to avoid orphans.

That's it. The fancy parts of PDF (fonts, glyphs, colour spaces, embedded images, annotations, forms, digital signatures) layer over this kernel. LaTeX is this algorithm with better heuristics; ReportLab is this algorithm with a richer block vocabulary; iText is this algorithm with full PDF binary output.

First Rosetta Stone project where **the domain's apparent complexity reduces to a simple loop**. G070's browser tab model looked complex; G077's encryption looked complex; G082's CMS looked complex. G091's pagination looks complex and reduces to "fit or overflow". The lesson: domain complexity often hides a simple invariant.

## Insight: Word Wrap Is the Paragraph's Own Layout

A paragraph has to become some number of lines. The chunking is word-by-word: start a line with the first word; each subsequent word either appends (if `current + 1 + word <= width`) or starts a new line. Long unbreakable words overflow the line rather than getting truncated — a 30-char word on a 10-char line is bad typography but preserves information, and truncation silently loses content.

First Rosetta Stone project where **the layout of a leaf block is itself non-trivial**. G083's template maker had substitution but no layout; G081's e-card had rendering but no wrap. G091's paragraph block delegates `wrap_text(text, width)` → list of lines, then the pagination loop treats each line as a unit height. The separation between "content to lines" and "lines to pages" is clean.

## Insight: Rendered Document Is Renderer-Independent

G091 doesn't emit PDF bytes. It emits a **list of pages**, each page with a **list of placed lines** (text, y-position, is-heading). That's the input any renderer needs — a PDF renderer turns it into PDF's content-stream operators, an HTML renderer turns it into `<div style="top: Npx">` spans, a terminal renderer prints it with position markers.

This separation is every publishing pipeline's killer feature. LaTeX's `.aux` intermediate, HTML's `layout tree`, PDF's `content stream` — all are "rendered" representations between the source and the final bytes. Having them as **data** rather than an output stream means you can serialise them, test them deterministically, ship them to another process, or render to multiple targets from one source.

First Rosetta Stone project with an explicit **IR (intermediate representation)**. G082's CMS had revisions as an IR for content history; G091 has `Vec<Page>` as an IR for visual layout. The pattern — source → IR → output — is the foundation of every modern compiler and renderer.

## Insight: Heading Orphan Avoidance Is Built-In

A heading at the bottom of a page with no body text below it is an **orphan** — an ugly typography sin. G091 handles it naïvely: if a heading needs 2 lines (title + blank line below) and only 1 is left on the current page, start a new page before placing the heading. Not perfect (doesn't look ahead to see whether the body would fit either), but sufficient for the Rosetta Stone.

First Rosetta Stone project where **visual aesthetics constrain layout logic**. Everything prior (sorting, compression, path resolution) had a pure correctness target. Layout's correctness target is partially aesthetic — and aesthetics are rules the algorithm has to encode.

## Choreographic Case: Vault Report

```innate
(@vault-report){
  @doc <- @pdf/new{width: 80, height: 66}
  @pdf/heading{doc: @doc, text: "Weekly Report", level: 1}
  @pdf/paragraph{doc: @doc, text: @weekly-summary}
  @sections <- @vault/find{tag: "report"}
  @for section in @sections {
    @pdf/heading{doc: @doc, text: @section.title, level: 2}
    @pdf/paragraph{doc: @doc, text: @section.body}
  }
  @pages <- @pdf/render{doc: @doc}
  ;; Hand off to whichever renderer the vault has configured —
  ;; markdown, HTML, terminal, real PDF via an external library.
  @renderer/emit{format: @preferred-format, pages: @pages}
}
```

The vault gathers report sections from the graph, builds a document, renders to pages, dispatches to a format-specific emitter. The pagination logic is stable; the emitter plugs in per context.

## Structures

```innate
(defenum block-kind HEADING | PARAGRAPH | SPACER | PAGE_BREAK)

(defstruct block
  kind   : BlockKind
  text   : String
  level  : Int           ;; heading level
  height : Int)          ;; spacer height

(defstruct page-size
  width-chars  : Int
  height-lines : Int)

(defstruct rendered-line
  text       : String
  y          : Int
  is-heading : Bool)

(defstruct page
  lines  : [RenderedLine]
  number : Int)

(defstruct document
  page-size : PageSize
  blocks    : [Block])
```

## Resolver Natives

```innate
@pdf/new{width, height}                     -> Document
@pdf/heading{doc, text, level}              -> Document    ;; fluent
@pdf/paragraph{doc, text}                   -> Document
@pdf/spacer{doc, height}                    -> Document
@pdf/page-break{doc}                        -> Document
@pdf/render{doc}                            -> [Page]
@pdf/page-count{doc}                        -> Int
@pdf/wrap-text{text, width}                 -> [String]
```

## Demo

```innate
(@demo){
  @doc <- @pdf/new{width: 40, height: 10}
  @pdf/heading{doc: @doc, text: "Rosetta Stone", level: 1}
  @pdf/paragraph{doc: @doc, text: "The quick brown fox jumps over the lazy dog. Pack my box with five dozen liquor jugs."}
  @pdf/heading{doc: @doc, text: "Section 1", level: 2}
  @pdf/page-break{doc: @doc}
  @pdf/paragraph{doc: @doc, text: "Content starts on a fresh page."}
  @pdf/render{doc: @doc}
  ;; -> two pages, layout deterministic across languages
}
```

## Where

Paragraphs MUST wrap at word boundaries preserving word integrity — breaking mid-word is bad typography and silently loses information when the word becomes two fragments. Words longer than line width MUST overflow, NOT truncate — losing characters silently is worse than an overlong line (editors can see overflow; they can't see truncation). Headings MUST force a page break if they don't fit with a trailing blank line — orphans are a recognised typography sin and must be prevented at the kernel layer. Render output MUST be deterministic — same document in any language produces identical pages in identical order, because the rendered IR is a contract between layers. Empty documents MUST render zero pages, NOT a blank page one — "nothing to render" and "a page with nothing on it" are different outputs and the distinction matters to downstream emitters. `page_break` on an empty page MUST be a no-op, NOT an emit-blank-page — a fresh document's first `page_break` is common UX (defining where the title page ends) and should not produce spurious blanks.
