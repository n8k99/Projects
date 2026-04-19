# G083 — Template Maker

> The Rosetta Stone's first **meta-level** project. Where G081 consumed pre-built templates, G083 is the builder for templates themselves — a tool that creates tools. Validation detects mismatches between the template's declared slots and the placeholders its HTML references, catching typos and unused declarations before they ever reach a card.

```yaml
id: G083
title: Template Maker
category: web
requires: [G069-wysiwyg, G081-ecard]
provides: [template-as-editable-data, cross-reference-validation, preview-with-samples, meta-level-tooling]
```

## Insight: The Code That Produces Templates Is Itself a Data Model

G081 treated a `Template` as a data record: slots, HTML, category. The template was produced once (perhaps by hand-writing a struct literal in code) and consumed many times. G083 makes the production step itself data-driven: a `TemplateBuilder` is an editable record holding the in-progress definition, and `TemplateMaker` is the tool that lets you mutate it, validate it, and produce the frozen `Template`.

First Rosetta Stone project where **the code path that creates artifacts is itself an editable, inspectable data model**. Same philosophical shift as G080's "tasks are data" and G081's "templates are data" — but at a higher level. Not "store the output as data"; "store the process as data."

This is the shape every good authoring tool has:
- Figma stores design files (data), not drawing commands (code).
- Excel stores spreadsheets (data), not calculation programs (code).
- The vault's note-template system stores templates (data), not note-generation scripts (code).

G083 presents the pattern at the minimum viable level: a simple linear model of "define slots, write HTML, set samples, preview, validate."

## Insight: Cross-Reference Validation Between Two Representations

The builder has two representations of "what slots exist":
1. **The `slots` list** — declarative: every slot has a name, kind, required flag, default.
2. **The HTML placeholders** — operational: every `{{name}}` in the HTML draft.

These two must agree. The validator finds the mismatches:
- `UnusedSlot(name)`: declared but never referenced.
- `UndeclaredPlaceholder(name)`: referenced but never declared.

Both are real bugs. Unused slots clutter the UI (the user fills a field that never shows up) and indicate the template was edited without updating the slot list. Undeclared placeholders produce the raw `{{typo}}` in output (G081's visible-bug behaviour) — the user's typo shows up in their cards.

First Rosetta Stone project where **two representations of the same concept are validated against each other**. G082 CMS had lifecycle states and scheduled publish — but those were different concepts. G083 has one concept (slots) with two representations (declared + referenced), and validation detects drift between them.

Every schema-migration tool does this: the database schema (one representation) and the ORM model (another) must agree, or the tool warns. Every protobuf-to-code generator does this: the `.proto` file and the generated code must agree. G083 is a tiny version of the same pattern.

## Insight: Preview Uses Sample Data, Not Real Data

The template author hasn't yet created any cards — there's nothing "real" to render. `preview` uses `sample_data` — per-slot example values the author provides. Missing samples fall back to slot defaults; if neither is set, the preview shows the raw placeholder (which is also what `validate` warns about).

This is **preview-with-placeholders**: the author sees roughly what cards will look like without needing a card first. The vault's note-template system does this — previewing the template shows sample text where the real frontmatter values will go.

First Rosetta Stone project with **design-time sample data** separate from runtime data. G081 only had real card values; G083 introduces samples as a first-class field on the builder (not on the card), so the author can preview without creating cards.

## Insight: Validation Catches the Full Lifecycle of Template Bugs

Four bug families:
- **Structural**: invalid slot names (empty, contain punctuation), duplicate slot names. These break the template before it even runs.
- **Completeness**: unused slot, undeclared placeholder. The template runs, but the author's intent and the HTML disagree.
- **Runtime-preview**: missing sample for required slot. The template works in production but can't be previewed because the author hasn't provided the necessary sample.

Each is a different *kind* of problem, produced by a different authoring mistake, fixed with a different action. Grouping them under a single `validate()` function that returns an itemised list lets the UI render them all at once with specific error messages per kind.

First Rosetta Stone project with **issue-kind-typed validation results**. G081 had single-issue errors (missing required, unknown slot); G083 has a structured list with enough type information to drive a UI that groups and highlights issues by kind.

Production linters (ESLint, Clippy, pylint) all work this way: validate everything, return a list, let the caller render. G083 is the pattern at minimum scale.

## Insight: Special Placeholders (_recipient, _sender) Are Reserved

The template can reference `{{_recipient}}` and `{{_sender}}` without declaring them as slots. The validator skips them (they're populated at card creation from fixed fields, not template-declared slots). Preview fills them with `[preview recipient]` / `[preview sender]` so the author sees where they appear.

This is the **reserved-name convention**: names beginning with `_` are special and not subject to the normal declare-then-reference rules. Same pattern as Python's `__special_methods__`, Ruby's `@@class_vars`, HTML's `data-*` attributes. G083 formalises the convention at the tiniest scale.

First Rosetta Stone project with **a naming convention as a schema extension mechanism**. The noosphere's choreography language can use similar reservations: `@_ctx`, `@_out`, `@_err` as built-in variables that don't need to be declared.

## Choreographic Case: Template Editor UI

```innate
(@template-editor){
  @maker <- @tm/new-maker

  @ui/on-user-creates-template (@data){
    @bid <- @tm/new-template{maker: @maker, name: @data.name, category: @data.category}
    @ui/navigate-to-editor{builder_id: @bid}
  }

  @ui/on-user-edits-slot (@bid, @slot){
    @tm/add-slot{maker: @maker, id: @bid, slot: @slot}
    @issues <- @tm/validate{maker: @maker, id: @bid}
    @preview <- @tm/preview{maker: @maker, id: @bid}
    @ui/render-editor{issues: @issues, preview: @preview}
  }

  @ui/on-user-saves (@bid){
    @issues <- @tm/validate{maker: @maker, id: @bid}
    where { no_issues: @issues.length == 0 }
    @template <- @tm/finalize{maker: @maker, id: @bid}
    @gallery/add-template{template: @template}
    @ui/notify{kind: "success", msg: "Template saved."}
  }
}
```

A live-editing template UI falls naturally out of G083's primitives: every edit triggers validate + preview, the UI shows both, saving requires a clean validation. Production WordPress theme editors, Ghost's theme system, Notion's block templates all work this way.

## Structures

```innate
(defstruct builder
  id               : Int
  name, category   : String
  slots            : [Slot]
  html-draft       : String
  sample-data      : {String -> String})

(defenum issue-kind
  UnusedSlot | UndeclaredPlaceholder | MissingSample
  | DuplicateSlot | InvalidSlotName)

(defstruct issue
  kind : IssueKind
  name : String)

(defstruct maker
  builders : [Builder]
  next-id  : Int)
```

## Resolver Natives

```innate
@tm/new-maker                                    -> Maker
@tm/new-template{maker, name, category}          -> BuilderId
@tm/add-slot{maker, id, slot}                    -> Ok | error
@tm/remove-slot{maker, id, slot-name}            -> Ok | error
@tm/update-html{maker, id, html}                 -> Ok | error
@tm/set-sample{maker, id, slot-name, value}      -> Ok | error
@tm/validate{maker, id}                          -> [Issue]
@tm/preview{maker, id}                           -> String?
@tm/builder{maker, id}                           -> Builder?
```

## Demo

```innate
(@demo){
  @m <- @tm/new-maker
  @bid <- @tm/new-template{maker: @m, name: "Classic Birthday",
                             category: "greeting"}
  @tm/add-slot{maker: @m, id: @bid,
                slot: {name: "name", kind: "text", required: true}}
  @tm/add-slot{maker: @m, id: @bid,
                slot: {name: "age", kind: "text",
                        default: "another year older"}}
  @tm/update-html{maker: @m, id: @bid,
                    html: "<h1>Happy Birthday, {{name}}!</h1>
                           <p>{{age}}. From {{sendr}}.</p>"}
  @tm/validate{maker: @m, id: @bid}
    ;; -> [UndeclaredPlaceholder("sendr"), MissingSample("name")]

  @tm/update-html{maker: @m, id: @bid,
                    html: "<h1>Happy Birthday, {{name}}!</h1>
                           <p>{{age}}. From {{_sender}}.</p>"}
  @tm/set-sample{maker: @m, id: @bid, slot-name: "name", value: "Alice"}
  @tm/validate{maker: @m, id: @bid}           ;; -> []   (clean)
  @tm/preview{maker: @m, id: @bid}
    ;; -> "<h1>Happy Birthday, Alice!</h1><p>another year older. From [preview sender].</p>"
}
```

## Where

Validation MUST detect both UnusedSlot AND UndeclaredPlaceholder — a template with a declared slot that no placeholder references is half-broken, and a placeholder with no declared slot is also half-broken; catching only one half misses half the bugs. Reserved names (prefix `_`) MUST be skipped by the UndeclaredPlaceholder check — they're system-provided, not user-declared. Preview MUST use sample data first, slot defaults second, and leave the raw placeholder in the output if neither is available — silently substituting empty string hides the absence of sample data. Duplicate slot names MUST be detected structurally — the template-level validator MUST NOT assume slots are unique; two `name` slots must produce a duplicate-slot issue even if one is required and one is optional. Slot names MUST be validated against a character whitelist — empty names, names with punctuation, names with leading digits are all bugs surfaced by the validator rather than silent failures.
