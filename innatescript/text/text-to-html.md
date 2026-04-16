# Text to HTML Generator

Format translation. Content preserved, representation changed.

## Resolver native

```dpn
@text_to_html{text: @text} -> @text
```

## Key insight: format translation as identity transformation

Text to HTML is FORMAT TRANSLATION — converting between two representations of the same content. This is G008's base conversion (binary/hex) and G010's unit conversion applied to documents. The content is the same; the format changes. The resolver's normalize-to-canonical pattern applies: parse markup into internal representation, then emit HTML.

```dpn
@text_to_html{text: "**bold** and *italic*"}
-> "<p><strong>bold</strong> and <em>italic</em></p>"
```

The input and output say the same thing. The difference is audience: markdown is for authors, HTML is for renderers. The resolver treats this as a morphism between two objects in the category of document formats — structure-preserving, content-preserving, representation-changing.

## Markdown-to-HTML as vault rendering pipeline

Markdown-to-HTML is how the VAULT RENDERS. Every vault note is markdown. Every rendered view is HTML. The text-to-html converter is the rendering pipeline that sits between vault content and user-visible output. The Rosetta Stone is building the vault's renderer.

```dpn
@vault_note{path: "The Work/projects/dragonpunk.md"}
  -> @read_file -> @text
  -> @text_to_html -> @html
  -> @display
```

This is the pipeline that turns a vault file into something you see on screen. The converter is the middle step — the transformation between storage format and display format. Every Quickshell panel that shows vault content runs this pipeline implicitly. Making it explicit means the resolver can intercept, cache, and compose renderings.

## Inline formatting as pattern-based transformation

The inline formatting rules (**bold**, *italic*, `code`, [link](url)) are PATTERN-BASED TRANSFORMATIONS — the same class of operation as G017's Pig Latin and G019's palindrome search. Scan text for patterns, transform matches, preserve non-matching content. This is `@map` over pattern matches with positional context.

```dpn
@apply_inline{text: "a **bold** word"} -> "a <strong>bold</strong> word"
@apply_inline{text: "use `code` here"} -> "use <code>code</code> here"
@apply_inline{text: "[click](url)"} -> "<a href=\"url\">click</a>"
```

Each rule is a find-and-replace with structure: find the pattern delimiters, extract the content between them, wrap the content in the target format's tags, leave everything else unchanged. The rules compose: bold inside a paragraph, code inside a list item, links inside headers. The resolver applies them in priority order — code first (to protect its contents from further parsing), then bold, then italic, then links.

## Block structure as multi-granularity parsing

The block structure (paragraphs, headers, lists) requires LINE-LEVEL PARSING — a different granularity than character-level inline formatting. The converter operates at two granularity levels simultaneously: block structure (line-level) and inline formatting (character-level). This is the first multi-granularity parser in the Rosetta Stone.

```dpn
@parse_blocks{text: "# Title\n\nParagraph one.\n\nParagraph two."}
-> [
  @header{level: 1, content: "Title"},
  @paragraph{content: "Paragraph one."},
  @paragraph{content: "Paragraph two."}
]
```

The block parser splits on blank lines and classifies each block by its first characters: `#` means header, `- ` means list item, anything else means paragraph. Then inline formatting runs *within* each block's content. This two-phase architecture — block parse then inline parse — is the standard approach because the two levels don't interfere: a `#` inside a paragraph doesn't make a header, and a blank line inside bold text ends the paragraph.

## Entity escaping as safety boundary

HTML entity escaping (`&` to `&amp;`, `<` to `&lt;`, `>` to `&gt;`) is the SAFETY BOUNDARY between content and format. Without escaping, content can accidentally become structure — a `<` in text becomes an HTML tag opener. Escaping is the resolver saying "this is data, not code." The same principle applies everywhere formats embed: SQL injection, shell injection, XSS. The Rosetta Stone's first security pattern.

```dpn
@escape_html{text: "a < b & c > d"} -> "a &lt; b &amp; c &gt; d"
```

## Where it becomes a choreography

Content publication pipeline. An agent writes in markdown (natural for authoring). The converter transforms to HTML (natural for display). A second agent reviews the HTML rendering. A third agent publishes. The format translation is one step in a multi-agent content pipeline.

```dpn
@choreography content_pipeline {
  author -> @write_markdown -> content
  converter -> @text_to_html{text: content} -> rendered
  reviewer <- @review{html: rendered} -> approved
  publisher -> @publish{html: approved, where: "site"}
}
```

Each agent works in the format natural to its role. The converter is the bridge between authoring format and display format. Without it, every agent must understand every format — an N*M problem. With format translation agents, each agent understands one format, and translators handle the edges — an N+M problem. This is why the resolver protocol exists: it's the universal translator between formats, making the choreography possible.
