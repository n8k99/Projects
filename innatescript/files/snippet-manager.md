# G099 — Code Snippet Manager

> The Rosetta Stone's fifteenth **Files-category** project. Introduces the **tab-stop expansion** model that every IDE snippet system uses (TextMate, VS Code, Sublime, yasnippet). A snippet body contains placeholders (`${N:default}`, `$N`, `$0`); expansion produces both the literal text AND a list of tab stops (cursor positions). The **stop-ordering rule** is subtle: non-zero stops in ascending order, then `$0` last — exactly what every editor does when the user presses Tab through a snippet.

```yaml
id: G099
title: Code Snippet Manager
category: files
requires: [G081-ecard, G083-template-maker, G095-excel-exporter]
provides: [tab-stop-parsing, stop-ordering-with-zero-last, snippet-library-indexed, placeholder-default-substitution]
```

## Insight: Tab Stops Are the Editor's Contract

A snippet expansion is **two things the editor needs**:
1. The literal text to insert.
2. The list of cursor positions to visit as the user tabs through.

Neither alone is sufficient. Just the text would mean the user has to manually place the cursor at each variable to fill in. Just the positions without text would have nothing to insert. The parser must produce both, in lockstep, with offsets that stay correct as default values are substituted in.

First Rosetta Stone project where **the return value is a two-part structured result** matching an external contract (the IDE's snippet protocol). G091's renderer returned pages + lines; G099 returns text + stops. Both are IRs that external tools consume.

## Insight: `$0` Is the Final Cursor, Not a Regular Stop

Non-zero stops (`$1`, `$2`, `$3`...) are ordered positions — the user tabs forward through them. `$0` is the **final resting place** — after the user exhausts the regular stops, the cursor lands on `$0` and snippet mode exits.

That ordering rule is **universal** across every snippet system going back to TextMate 1. If G099 got it wrong, users coming from any other editor would notice immediately. The sort comparator: non-zero stops ascending by number; `$0` always last regardless of its position in the text.

First Rosetta Stone project where **a convention from an entire software ecosystem is the correctness target**. Getting it wrong doesn't crash the program — it makes the snippet feel wrong to anyone who's used any other editor.

## Insight: Placeholder Syntax Has Three Forms

Every snippet parser handles three cases:
1. `${N:default}` — numbered stop with default text to pre-fill.
2. `${N}` — numbered stop with no default.
3. `$N` — shorthand for `${N}`; no braces, no default.

The body character-by-character: if `$` followed by `{`, find the matching `}` and parse `{...}` contents. If `$` followed by digits, scan digits and make a no-default stop. Otherwise, copy the character verbatim (including `$` followed by anything else).

First Rosetta Stone project where **the parser handles three variant forms of the same concept** without exploding into three separate parsers. The variants share structure (all produce a TabStop); only their presentation differs.

## Insight: Duplicate Stop Numbers Mirror Edits

`${1:name}` appearing twice in the same snippet is not a bug. It means "type once, mirror to both". When the user types at stop `$1`, both copies update simultaneously. This is how every snippet system handles variables that appear in multiple places.

G099 doesn't implement the mirroring itself (that's UI logic), but preserves the duplicates in the expansion — the editor groups stops by number and treats them as one logical cursor position with multiple visual locations. The parser's job is to emit **both stops** so the editor can do the grouping.

First Rosetta Stone project where **the model permits logically-duplicate data that downstream code will treat specially**. The library doesn't deduplicate; the consumer does.

## Insight: Library Indexes by Trigger, Language, Tag

A snippet store has three common query patterns:
* Trigger + language → "what does `for` expand to in Python?"
* Language only → "show me all Python snippets"
* Tag → "show me all looping snippets across languages"

G099 stores each index as a sorted list or a hash map by the relevant key. Insertion updates every index; queries hit exactly the one that matches the question. The library is small — a few dozen snippets, maybe a few hundred. A more scalable version would use a real inverted index, but the Rosetta Stone's scale doesn't demand it.

First Rosetta Stone project where **a small library indexes its contents multiple ways**. G093's tag store indexed music metadata; G099 applies the same pattern to code snippets. Same model, different domain.

## Choreographic Case: Vault Code Snippets

```innate
(@vault-code-snippets){
  @lib <- @snippet/library{}
  @files <- @fs/ls{path: "snippets/*.json"}
  @for file in @files {
    @data <- @json/parse{text: @file/read-string{path: @file.path}}
    @snippet/library-add{library: @lib, snippet: @data}
  }

  @on-user-types-trigger (@trigger @lang){
    @s <- @snippet/find-by-trigger{library: @lib, trigger: @trigger, language: @lang}
    @when (@s){
      @e <- @snippet/expand{snippet: @s}
      @ui/insert-with-stops{text: @e.text, stops: @e.stops}
    }
  }
}
```

The vault loads snippets from JSON files, looks up by trigger as the user types, and inserts the expansion with tab stops for the UI to navigate.

## Structures

```innate
(defstruct tab-stop
  number : Int        ;; 1, 2, 3, ... | 0 for final
  offset : Int        ;; char offset in expanded text
  length : Int)       ;; default text length

(defstruct expansion
  text  : String
  stops : [TabStop])

(defstruct snippet
  trigger     : String
  language    : String
  tags        : [String]
  body        : String
  description : String)
```

## Resolver Natives

```innate
@snippet/library{}                                     -> SnippetLibrary
@snippet/library-add{library, snippet}                 -> Unit
@snippet/find-by-trigger{library, trigger, language}   -> Snippet | null
@snippet/find-by-language{library, language}           -> [Snippet]
@snippet/find-by-tag{library, tag}                     -> [Snippet]
@snippet/expand{snippet}                               -> Expansion
```

## Demo

```innate
(@demo){
  @lib <- @snippet/library{}
  @snippet/library-add{library: @lib,
                        snippet: {trigger: "for", language: "python",
                                  tags: ["loop"],
                                  body: "for ${1:item} in ${2:collection}:\n    ${3:pass}\n$0"}}
  @s <- @snippet/find-by-trigger{library: @lib, trigger: "for", language: "python"}
  @snippet/expand{snippet: @s}
  ;; text: "for item in collection:\n    pass\n"
  ;; stops: [#1 @4/4, #2 @12/10, #3 @28/4, #0 @33/0]
}
```

## Where

Expansion MUST produce both text AND stops — either alone is useless to the editor. Stops MUST be sorted with 0 LAST, regardless of text position — this is the universal snippet contract; violating it breaks user expectations imported from every other editor. Placeholders MUST support three syntactic forms (`${N:default}`, `${N}`, `$N`) — TextMate's spec has been stable for 20 years and every downstream tool expects all three. Duplicate stop numbers MUST be preserved, NOT deduplicated — mirroring is a UI concern; the parser provides the raw grouping data. Text and offsets MUST use consistent character units (chars/codepoints, NOT bytes) — UTF-8 byte offsets confuse editors that expect character counts. Missing/malformed `$...` MUST fall through as literal text, NOT error — snippets frequently contain `$` as part of the language (Bash, Makefile, jQuery) and must not break on it.
