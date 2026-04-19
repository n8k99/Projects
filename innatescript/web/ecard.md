# G081 — E-Card Generator

> The Rosetta Stone's first **template-and-data rendering system**. A `Template` declares slots; a `Card` fills them; `render` substitutes values into placeholders. Validation happens at card creation — stored cards are always valid, so rendering can never fail on a stored card.

```yaml
id: G081
title: E-Card Generator
category: web
requires: [G069-wysiwyg, G076-bookmarks]
provides: [template-with-declared-slots, validation-at-creation, default-values, template-as-data, multiple-output-formats]
```

## Insight: Content and Presentation Separate via a Template

Every prior project that produced formatted output had the format baked into the code. G069 WYSIWYG's render function knew about `<h1>`, `<b>`, `<i>`; G074 Whiteboard's `to_svg` knew about `<polyline>`. G081 is the first project where **the rendering format is template-driven**: the HTML surface is data (a string in the template record), and swapping the template swaps the output without changing any code.

First Rosetta Stone project where **presentation is data, not code**. This is the same philosophical move as G080's "tasks are data" — choose declarative representation over imperative code for anything that might be user-visible or user-editable.

Every production CMS, email system, invoice generator, and document-generation pipeline works this way: templates live in a database or filesystem, data flows through them, rendering is a trivial substitution. The vault's note-template system uses this exact shape: note templates in `The Commons/Templates/` with frontmatter slots, new-note creation fills the slots, the result IS the note.

## Insight: Slots Are a Schema, Validated at Creation

A `Slot` declares its `name`, `kind` (Text / LongText / ImageUrl / Color / Date), `required`, and optional `default`. When a `Card` is created, the filled-in data is validated against the template's slot schema:
- **Unknown slot**: the user provided a value for something the template doesn't know about → reject.
- **Missing required**: the template required a slot the user didn't fill → reject.
- **Optional slot with default**: user didn't fill it, so the default is applied.

Validation happens **once, at creation**. A card stored in the gallery is always valid — every required slot is filled, no unknown slots are present, defaults are materialised. Rendering is a pure substitution; it can never fail on a stored card.

First Rosetta Stone project where **validation is a creation-time operation, not a render-time operation**. G069's WYSIWYG rendered whatever was in the model; if the model was broken, the render was broken. G081 won't let you store a broken card. This is the **fail-fast** design principle at the data-layer boundary.

The trade-off: strict validation means sloppy or exploratory card creation is harder. Production systems often add "draft" states where validation is relaxed; G081 keeps it simple with only valid cards. The vault's note-template system picks differently — it'll allow incomplete notes during drafting and warn-but-not-reject on publish.

## Insight: Defaults Are Materialised, Not Lazy

When a card is created without filling an optional slot, the default value is **copied into the card's filled map**. This matters:

- The card is **self-contained**: you can render it without referring back to the template's default.
- Template changes don't retroactively change existing cards. If you edit the default from "year older" to "another year older", yesterday's cards still say "year older".
- Serialising the card produces a complete snapshot; deserialising reproduces exactly what the user saw.

Alternative: store only user-provided values, fall back to template defaults at render time. This is more compact but less stable — edits to templates change the rendering of old cards, which surprises users.

First Rosetta Stone project where **defaults are materialised at creation** rather than looked up at render time. Same pattern: database NULL-with-default (materialised when row is inserted), React component default props (materialised when component mounts), vault note creation from template (slots filled, then frontmatter written).

## Insight: Multiple Output Formats from One Card

`render` produces HTML; `render_plain` produces text. Both are **projections over the same card**. More formats are mechanical to add: `render_markdown`, `render_email_safe`, `render_ascii_art`. None require changing the template or the card — they're new walk functions.

Pattern established in G069 (WYSIWYG) and G074 (Whiteboard): **model + multiple render functions = multiple output formats without duplicating content**. G081 applies it explicitly through the template mechanism.

## Insight: Unknown Placeholders Are Visible Bugs

If a template contains `{{nonexistent_slot}}` — a placeholder whose slot isn't declared — G081 leaves the placeholder as-is in the output. The template author's typo is **visible** in the rendered output, not silently hidden by substituting an empty string.

This is an explicit design choice: **fail loud, not quiet**. The alternative (silently drop unknown placeholders) produces cards where "From {{nonexistent}}." becomes "From ." which looks like correct output until someone notices the missing word.

First Rosetta Stone project where **a design choice actively preserves visible evidence of bugs** rather than silently papering over them. Parallels: Python's `KeyError` on missing dict access (vs JavaScript's `undefined`), Rust's `Result` forcing error handling (vs silently swallowing), strict HTML parsers (vs G071's forgiving one — different tradeoff per domain).

The domain matters. G071's page scraper is forgiving because it deals with external, often-broken input. G081's e-card renderer is strict because it deals with user-authored templates where a typo should be fixable, not hidden.

## Insight: Categories Are an Axis; Slots Are Not

Templates have a `category` (birthday, anniversary, get-well) for user browsing. Slots belong to individual templates; one template's "age" slot is not the same as another's. This is the **shared vocabulary vs per-instance vocabulary** distinction — categories are shared across templates, slots are scoped to one template.

First Rosetta Stone project where **the category-instance hierarchy is explicit**. Categories are like folders (G076) or tags — organisational across instances. Slots are like fields on a single record — meaningful only within that record. Mixing them creates the "tag that's really a field" anti-pattern.

## Choreographic Case: Automated Birthday Card Mailer

```innate
(@birthday-mailer){
  @gallery <- @ecard/load-gallery

  @every day at 08:00 {
    @birthdays-today <- @vault/query{filter: "birthday == today"}
    @for person in @birthdays-today {
      @template <- @ecard/pick-template{gallery: @gallery, category: "birthday"}
      @card <- @ecard/create-card{
        gallery: @gallery,
        template_id: @template.id,
        filled: {name: @person.name, age: "year ${@person.age_today}!"},
        recipient: @person.email,
        sender: "the Vault"
      }
      @html <- @ecard/render{gallery: @gallery, card_id: @card}
      @email/send{to: @person.email, subject: "Happy birthday!", html: @html}
    }
  }
}
```

Daily birthday automation composes on G080's scheduler + G081's template gallery + G072's dispatched I/O. The pattern is fully declarative until the email send — everything above is data transformation.

## Structures

```innate
(defenum slot-kind Text | LongText | ImageUrl | Color | Date)

(defstruct slot
  name      : String
  kind      : SlotKind
  required  : Bool
  default   : String?)

(defstruct template
  id               : Int
  name, category   : String
  slots            : [Slot]
  html-template    : String)     ;; with {{slot_name}} placeholders

(defstruct card
  id              : Int
  template-id     : Int
  filled          : {String -> String}
  recipient, sender : String
  created-at-ms   : Int)

(defenum validation-error
  UnknownTemplate(id) | MissingRequired(name) | UnknownSlot(name))

(defstruct gallery
  templates  : [Template]
  cards      : [Card])
```

## Resolver Natives

```innate
@ecard/new-gallery                                           -> Gallery
@ecard/add-template{gallery, name, category, slots,
                     html-template}                          -> TemplateId
@ecard/templates-in-category{gallery, category}              -> [Template]

@ecard/create-card{gallery, template-id, filled,
                    recipient, sender, now_ms}               -> CardId   ;; or error
@ecard/card{gallery, id}                                     -> Card?
@ecard/cards-by-template{gallery, template-id}               -> [Card]
@ecard/render{gallery, card-id}                              -> String?
@ecard/render-plain{gallery, card-id}                        -> String?
```

## Demo

```innate
(@demo){
  @g <- @ecard/new-gallery
  @tid <- @ecard/add-template{
    gallery: @g, name: "Classic Birthday", category: "birthday",
    slots: [
      {name: "name", kind: "text", required: true},
      {name: "age", kind: "text", default: "another year older"},
      {name: "color", kind: "color", default: "#ffcc00"}
    ],
    html_template: "<div style='background:{{color}}'><h1>Happy Birthday, {{name}}!</h1>
                     <p>{{age}}. From {{_sender}}.</p></div>"
  }

  @cid <- @ecard/create-card{
    gallery: @g, template-id: @tid,
    filled: {name: "Alice", age: "turning 30!"},
    recipient: "Alice Jones", sender: "Bob Smith",
    now_ms: @now
  }

  @ecard/render{gallery: @g, card-id: @cid}
    ;; -> "<div style='background:#ffcc00'><h1>Happy Birthday, Alice!</h1>
    ;;     <p>turning 30!. From Bob Smith.</p></div>"
}
```

## Where

Validation MUST happen at card creation, not at render time — a card that fails validation MUST NOT be created; a card that was created MUST render successfully (render is total once creation passes). Defaults MUST be materialised into the card at creation time — looking up defaults at render time couples the card's output to the template's current state, which surprises users when templates are edited. Unknown slots in the filled map MUST be rejected, not silently discarded — typos are bugs and MUST be surfaced. Missing required slots MUST be rejected — partial cards are not representable in the gallery. Unknown placeholders in a template (slots referenced in HTML but not declared) MUST be left as-is in output — silent removal hides template-author bugs; visible placeholders let them be found and fixed. The template's slot list MUST be authoritative — a card's filled map MUST have no keys the template doesn't declare, and the render substitution MUST only use the filled map (and the special `_recipient` / `_sender`), not arbitrary template-time variables.
