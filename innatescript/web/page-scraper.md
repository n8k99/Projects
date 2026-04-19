# G071 — Page Scraper

> The Rosetta Stone's first **parser** — the inverse of G069's render. Where G069 took a structured model and projected it to a string, G071 takes a string and reconstructs the structure. The first project where the input is **untrusted, unstructured, potentially malformed**, and the system must recover gracefully rather than refusing to process.

```yaml
id: G071
title: Page Scraper
category: web
requires: [G021-text-editor, G069-wysiwyg]
provides: [forgiving-parser, selector-dsl, tree-query-language, text-extraction, postels-principle]
```

## Insight: Parse Is Inverse of Render, but Render is Total and Parse is Not

Render (G069) is a **total function**: every document has exactly one correct rendering; there is no such thing as an "unrenderable" document. Parse is **partial**: not every string is a valid HTML document. The implementation has to choose what to do with malformed input.

Two philosophies:
- **Strict parser**: refuse malformed input. Produce a parse error. The caller must supply valid input.
- **Forgiving parser**: accept anything, produce the best interpretation, recover from errors.

HTML parsing is the canonical forgiving-parser domain. Real web pages are messy — browsers encounter every kind of malformed input imaginable and users expect them to show *something*, not a blank page with "Parse error: unexpected character at 0:127." Postel's principle: be strict in what you send, liberal in what you accept.

G071 is a forgiving parser. Unknown constructs become text; stray close tags are dropped; unclosed tags let their content escape into the parent context. The output is usable for almost any input — perhaps not what a strict reader would produce, but structurally valid and queryable.

**First Rosetta Stone project where recovery is the spec**, not an afterthought. The noosphere will have several domains like this: parsing user-typed search queries, ingesting external RSS feeds with broken XML, reading Markdown files with invalid frontmatter, accepting choreographies from third-party sources. All need forgiving parsers; G071 presents the pattern.

## Insight: Selectors Are a Domain-Specific Query Language

Once the tree is parsed, querying it needs a language. CSS selectors are the natural fit because web developers already know them — `h1`, `.lead`, `#intro`, `article p`, `a.external`. G071 implements the subset that covers most real scraping: tag, class, id, combinations, descendant combinator.

This is the **first Rosetta Stone project with a domain-specific query language**. Previous projects queried collections through direct method calls (`room.user_count()`, `tree.ancestors(id)`); G071 queries the tree with a string that gets parsed into a selector AST, then evaluated against the node tree. Two parsers — one for HTML, one for selectors — and both are forgiving in the same way.

DSLs are everywhere in the noosphere: wiki-link path syntax `[[namespace/note#section]]`, tag queries `#foo -#bar`, search queries. Every DSL has the same shape: parse the user's string into an AST, walk the AST against the data. G071 is the minimal teaching case.

## Insight: Descendant Combinator Is Path-Based Pattern Matching

`article p` selects any `p` that is anywhere inside any `article`, at any depth. The implementation walks the tree; every time it matches the first element of the selector chain, it continues searching the descendants with the rest of the chain. If the tail of the chain doesn't match, backtrack.

This is **path-based pattern matching on trees**, and it's more powerful than it looks. The same algorithm generalises to XPath, to JSONPath, to the vault's wiki-link resolution, to filesystem glob descent (`src/**/*.ts`), to git ref-matching (`refs/heads/feature/*`). G071 presents the minimum viable case.

The subtlety: a selector can **match at multiple depths**. `p` matches every `p` in the tree, at any depth, not just top-level ones. `article p` matches every `p` under every `article`, transitively. The recursion has to be careful to:
1. Continue searching under a matched element (for more matches at deeper levels).
2. Keep searching under unmatched elements (so nested structures aren't skipped).

Both rules apply simultaneously. G071's `select_recursive` encodes them explicitly; a naive implementation forgetting either produces subtly-wrong results.

## Insight: Text Extraction Is a Different Walk with the Same Tree

`text_of(node)` returns the concatenation of all text descendants. It ignores attributes. It ignores structure except to traverse. It is a **different projection** of the same tree than `to_html` would be.

First Rosetta Stone case where **the same data structure supports multiple independent traversals for different questions**. G064 Family Tree had this at small scale (ancestors, descendants, siblings were all BFS queries), but G071 generalises: tree-walk is the universal interface to structured data, and each "question you could ask" is a specific walk.

This is why the vault's note-graph is useful: one tree, many walks. `text-of-this-note`, `links-from-this-note`, `tags-on-this-note`, `notes-reachable-from-this-note-within-3-hops` — all different walks on the same structure. G071 introduces the pattern at minimal scale.

## Insight: Parsing Is the First Place Untrusted Input Enters the System

G069 was trusted: the operations produced valid documents by construction. G071 takes arbitrary strings from outside — potentially from the network, potentially crafted by an adversary, potentially just broken through no one's fault. The parser is the first line of defence and must not crash or run forever on malicious input.

Safety properties G071 must maintain:
- **Termination**: every input, no matter how malformed, produces an output in bounded time. Loops have break conditions; recursion is bounded by input size.
- **Memory bound**: output size is linear in input size (no pathological amplification).
- **No crashes**: mismatched tags, truncated input, invalid UTF-8-like sequences don't panic.

A production parser would add rate limiting, depth limits, and sanity-checking against enormous attribute values. G071 has the basic properties in place; the others are production-hardening orthogonal to the teaching point.

## Choreographic Case: Extract Article Content for Summarisation

```innate
(@summarise-article){
  @url <- @params/url
  @html <- @http/get{url: @url}
  @tree <- @scraper/parse{html: @html}

  @title <- @scraper/text-of{node: (first @scraper/select{tree: @tree, selector: "h1"})}
  @body  <- @scraper/select{tree: @tree, selector: "article p"}
           .map(@scraper/text-of)
           .join("\n\n")
  @links <- @scraper/select{tree: @tree, selector: "article a"}
           .map(.attrs.href)

  where { content_substantive: @body.length > 200 }

  @summary <- @llm/summarise{text: @body}
  @emit{title: @title, summary: @summary, citations: @links}
}
```

The choreography reads naturally because the primitives are the right ones: parse, select, extract text. Every web-ingestion choreography in the noosphere will use these three operations.

## Structures

```innate
(defenum node
  Element { tag: String, attrs: {String -> String}, children: [Node] }
  Text    { text: String })

(defstruct simple-selector
  tag    : String?
  class  : String?
  id     : String?)

(defstruct selector
  chain  : [SimpleSelector])        ;; descendant-combined
```

## Resolver Natives

```innate
@scraper/parse{html}                         -> [Node]              ;; forgiving
@scraper/select{tree, selector}              -> [Element]           ;; CSS subset
@scraper/text-of{node}                       -> String              ;; recursive text concat
@scraper/attr{element, name}                 -> String?
```

## Demo

```innate
(@demo){
  @tree <- @scraper/parse{html: "
    <article id='main'>
      <h1>The Rosetta Stone</h1>
      <p class='lead'>A corpus demonstrating InnateScript.</p>
      <ul>
        <li><a href='/numbers'>Numbers</a></li>
        <li><a href='/classes' class='featured'>Classes</a></li>
      </ul>
    </article>
  "}
  @scraper/text-of{node: (first @scraper/select{tree: @tree, selector: "h1"})}
    ;; -> "The Rosetta Stone"
  @scraper/select{tree: @tree, selector: "ul a"}.map(.attrs.href)
    ;; -> ["/numbers", "/classes"]
  @scraper/text-of{node: (first @scraper/select{tree: @tree, selector: ".featured"})}
    ;; -> "Classes"
}
```

## Where

The parser MUST terminate on every input, including malformed input — loops MUST have break conditions that advance on every iteration (no "forever" loops on input the parser doesn't recognise). Unknown constructs MUST be treated as text and the parser MUST recover at the next valid boundary — refusing to parse is not an option for a scraper. Selector matching MUST continue under a matched element to find deeper matches at the same depth; the recursion MUST also continue under unmatched elements to find matches in nested structures; forgetting either rule produces subtly wrong results. Text extraction MUST recurse through all descendants and MUST NOT emit attribute values or tag names — attributes are queried separately via `attr{element, name}`. Void tags (br, img, hr, input, meta, link) MUST be parsed as self-closing without requiring a closing tag in the input. Self-closing syntax `<tag/>` MUST be accepted for any tag, not just void tags.
