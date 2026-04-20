# G114 — Slide Show

> The Rosetta Stone's first **Graphics project**. Opens the category with the **navigation-state** and **dual-track content** patterns. A presentation is an ordered list of slides with a bounded `current_index`; `advance` / `back` / `goto` move through the deck without overflow. Each slide has two content tracks — **visible** (audience sees) and **speaker notes** (presenter-only) — selected at render time via **presentation mode** (Speaker / Audience / Handout). This is the kernel every slide app (PowerPoint, Keynote, reveal.js, Beamer) ships.

```yaml
id: G114
title: Slide Show
category: graphics
requires: [G091-pdf-generator, G099-code-snippet-manager, G101-sql-query-analyzer]
provides: [bounded-navigation-state, dual-track-content, presentation-mode, slide-element-enum]
```

## Insight: Navigation Is Bounded State

The only mutable state in a presentation is `current_index`. Every navigation action (advance / back / goto) checks bounds before mutation. `advance` at end returns `false` without moving; `back` at start returns `false`; `goto(n)` only moves if `n` is valid. The index is never out of range.

This is the **bounded state machine** pattern — every button click is a transition with a defined valid set. The same pattern runs under every media player (can't seek past end), every wizard (can't click Next without filling the form), every paginated UI.

First Rosetta Stone project where **navigation state is explicitly bounded** with return-value signalling. G073's telnet had state transitions but via protocol dispatch; G114 has numeric index bounds and reports success/failure per action.

## Insight: Dual-Track Content Separates Audience from Presenter

A slide has a **visible** track (what goes on screen) and a **notes** track (what only the presenter sees on the confidence monitor). The two are stored together in the `Slide` struct and rendered separately based on mode.

`render_current(SPEAKER)` → visible + notes. `render_current(AUDIENCE)` → visible only. `render_handout()` → full deck as a document with notes interleaved. Same data, three views.

First Rosetta Stone project where **the same data structure has multiple render modes** that selectively include or exclude parts. G091's rendered-document IR was mode-free; G114's render functions take a `PresentationMode` parameter.

## Insight: Slide Elements Are a Closed Enum

`Heading { text, level }`, `Bullet { text, indent }`, `Image { src, alt }`, `Code { text, language }`, `Spacer { lines }`. Five variants, enumerable. Adding a new variant (Video? Math equation?) is a new enum case plus a render branch — localised change.

This is the same algebraic-data-type pattern as G085's quiz questions, G091's PDF blocks, G099's tab stops. The Rosetta Stone keeps finding domains where a closed enum of content variants is the right model.

First Rosetta Stone project where **slide content is a closed enum of element kinds**, not a free-form text or tagged map. Classical, well-scoped.

## Insight: Presentation Mode Is an External Parameter

`render_slide(slide, mode)` takes mode from the caller. The slide itself doesn't know what mode it's in; modes are chosen per render call. This lets the same presentation drive the audience projector *and* the speaker's confidence monitor simultaneously — two render calls, two different outputs.

First Rosetta Stone project with **the same content, rendered differently in parallel contexts**. G112 had dialect-parameterised rendering (choose one target at a time); G114 renders simultaneously for two audiences.

## Insight: Search Spans Title and Content

`search(needle)` returns the indices of slides that mention the needle in either the title or any visible element's text. Speaker notes are *not* searched — those are meta, not content. This matches how audience-facing search works in presentation tools: find the slide they saw.

First Rosetta Stone project where **search scope is deliberately narrower than the stored data** — notes are stored but excluded from search because they're presenter-meta, not content.

## Insight: Fluent Builder Keeps Slide Construction Readable

`Slide::new("Intro").heading("Welcome", 1).bullet("Agenda", 0).with_notes("...")`. Same builder pattern as G112. Each call returns `self` so flags and elements compose linearly. Without it, slide construction becomes a positional-arg nightmare.

First Rosetta Stone project where **the builder pattern applies to content construction, not just configuration**. G112's builders were for column metadata; G114's are for content elements.

## Choreographic Case: Vault Talk Builder

```innate
(@vault-talk-builder){
  @talk <- @slide/presentation{title: "My Research"}
  @slides <- @vault/find-by-tag{tag: "talk-slides"}
  @for s in @slides {
    @slide/add{presentation: @talk, slide: @slide/from-vault-note{note: @s}}
  }

  @on-user-presents {
    @ui/open-two-windows{
      audience: @slide/render{presentation: @talk, mode: "audience"},
      speaker: @slide/render{presentation: @talk, mode: "speaker"}
    }
  }

  @on-keyboard-right {
    @slide/advance{presentation: @talk}
    @ui/refresh{}
  }

  @on-handout-export {
    @vault/save{path: "talks/my-research-handout.md",
                content: @slide/handout{presentation: @talk}}
  }
}
```

The vault's talk-building flow is: gather slides from tagged notes, open two windows (audience + speaker), drive both from the same presentation, export a handout on demand.

## Structures

```innate
(defenum element-kind HEADING | BULLET | IMAGE | SPACER | CODE)

(defstruct slide-element
  kind     : ElementKind
  text     : String
  level    : Int         ;; HEADING
  indent   : Int         ;; BULLET
  src      : String      ;; IMAGE
  alt      : String      ;; IMAGE
  language : String      ;; CODE
  lines    : Int)        ;; SPACER

(defenum transition NONE | FADE | SLIDE | DISSOLVE)

(defstruct slide
  title      : String
  content    : [SlideElement]
  notes      : String
  transition : Transition)

(defenum mode SPEAKER | AUDIENCE | HANDOUT)

(defstruct presentation
  title         : String
  slides        : [Slide]
  current-index : Int)
```

## Resolver Natives

```innate
@slide/new{title}                         -> Slide
@slide/heading{slide, text, level}        -> Slide      ;; fluent
@slide/bullet{slide, text, indent}        -> Slide
@slide/with-notes{slide, notes}           -> Slide
@slide/presentation{title}                -> Presentation
@slide/add{presentation, slide}           -> Unit
@slide/advance{presentation}              -> Bool
@slide/back{presentation}                 -> Bool
@slide/goto{presentation, index}          -> Bool
@slide/current{presentation}              -> Slide | null
@slide/render-current{presentation, mode} -> String
@slide/handout{presentation}              -> String
@slide/search{presentation, needle}       -> [Int]
```

## Demo

```innate
(@demo){
  @p <- @slide/presentation{title: "My Talk"}
  @slide/add{presentation: @p,
              slide: @slide/new{title: "Intro"}
                      .heading{text: "Welcome", level: 1}
                      .bullet{text: "Agenda", indent: 0}
                      .with-notes{notes: "Pause for laughs"}}
  @slide/render-current{presentation: @p, mode: "audience"}  ;; notes hidden
  @slide/render-current{presentation: @p, mode: "speaker"}   ;; notes shown
}
```

## Where

Navigation MUST be bounded — advance at end, back at start, and out-of-range goto all return false without changing state. Notes MUST be stored alongside visible content, NOT in a separate structure — they travel with their slide. Mode MUST be a render-time parameter, NOT stored on the presentation — the same presentation drives multiple simultaneous views. Element kinds MUST be a closed enum — new kinds are new variants, not string discriminators. Search MUST cover title and visible content but NOT notes — notes are presenter-meta, not searchable content. Progress MUST be 1-indexed for display — users think "slide 3 of 10", not "slide 2 of 10". Empty presentations MUST return `(0, 0)` progress and `None` for current — a zero-slide deck is valid, not an error.
