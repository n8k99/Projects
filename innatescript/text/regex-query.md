# Regex Query Tool

Universal pattern matching. The primitive underneath all text operations.

## Resolver native

```dpn
@regex{text: @text, pattern: @pattern} -> @matches
@regex_replace{text: @text, pattern: @pattern, replacement: @text} -> @text
@regex_split{text: @text, pattern: @pattern} -> [@text]
@is_match{text: @text, pattern: @pattern} -> @bool
@extract_groups{text: @text, pattern: @pattern} -> [[@text]]
```

## Pre-built patterns: domain schemas as regex

```dpn
@patterns.EMAIL     -> "[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\\.[a-zA-Z]{2,}"
@patterns.URL       -> "https?://[^\\s<>\"']+"
@patterns.PHONE     -> "\\+?1?[-.\\s]?\\(?\\d{3}\\)?[-.\\s]?\\d{3}[-.\\s]?\\d{4}"
@patterns.DATE      -> "\\d{4}-\\d{2}-\\d{2}"
@patterns.IP_ADDRESS -> "\\b(?:\\d{1,3}\\.){3}\\d{1,3}\\b"
```

These are DOMAIN SCHEMAS expressed as regex. An email address has a structure: `local@domain.tld`. The regex encodes that structure. This is the text equivalent of G022's RSS schema -- structured content described by its pattern. The resolver's text analysis reduces to: "does this text match this pattern?"

```dpn
@regex{text: "mail alice@example.com today", pattern: @patterns.EMAIL}
-> ["alice@example.com"]

@is_match{text: "192.168.1.1", pattern: @patterns.IP_ADDRESS}
-> true
```

## Key insight: regex is the universal text query language

Every text operation in the Rosetta Stone (reverse, palindrome, cipher, HTML conversion) could be expressed as a sequence of regex operations. Regex is the primitive underneath text manipulation, the way Dijkstra was the primitive underneath graph operations. The Text category ends where it began: with pattern matching on strings.

```dpn
;; G017 Pig Latin as regex
@regex_replace{text: "hello world", pattern: "\\b([^aeiou]*)(\\w+)", replacement: "$2$1ay"}
-> "ellohay orldway"

;; G030 text-to-html bold as regex
@regex_replace{text: "**bold**", pattern: "\\*\\*(.*?)\\*\\*", replacement: "<strong>$1</strong>"}
-> "<strong>bold</strong>"

;; G019 palindrome check as regex? Not directly -- but the TEST is regex:
@is_match{text: @reverse{text: word}, pattern: @escape{text: word}}
```

The converters and transformers of the Text category are all special cases of regex replace. G030's text-to-html `**bold** -> <strong>bold</strong>` is just `s/\*\*(.*?)\*\*/<strong>\1<\/strong>/g`. Replace with regex is G030 at full generality.

## extract_groups: structured extraction from unstructured text

`extract_groups()` introduces STRUCTURED EXTRACTION from unstructured text -- pulling typed fields out of free text using capture groups. This is how agents parse natural language into structured data. The regex is the schema. The text is the input. The groups are the fields. This is the ETL (extract-transform-load) pattern for the noosphere.

```dpn
@extract_groups{
  text: "[2026-04-16 08:30:00] ERROR: disk full",
  pattern: "\\[([\\d-]+) ([\\d:]+)\\] (\\w+): (.+)"
}
-> [["2026-04-16", "08:30:00", "ERROR", "disk full"]]

;; The regex IS the schema:
;;   group 1 = date
;;   group 2 = time
;;   group 3 = level
;;   group 4 = message
;; No separate schema definition needed -- the pattern encodes structure.
```

This is the bridge between unstructured and structured data. Raw log files, natural language, freeform text -- all become structured records when a regex with capture groups is applied. The resolver doesn't need a separate parser for each format; it needs a regex for each format.

## META-OBSERVATION: the Text category architecture

G016-G032 discovered the content architecture:

```
String manipulation (G016-G020)
  -> State machines (G021)
  -> Structured formats (G022)
  -> Data stores (G023-G027)
  -> Security (G028)
  -> Recommendation (G029)
  -> Format translation (G030)
  -> Self-verifying tokens (G031)
  -> Universal pattern matching (G032)
```

The Text category's substrate is regex. Everything else is a named pattern. A palindrome checker is `@is_match{text: reversed, pattern: original}`. A vowel counter is `@regex{text: word, pattern: "[aeiou]"}.length`. A cipher is `@regex_replace` with a translation table. An RSS parser is `@extract_groups` with the RSS schema as pattern. The entire category collapses into: match, extract, replace, split.

## Choreographic case: log analysis pipeline

An agent ingests raw log files, applies regex patterns to extract structured events (timestamps, error codes, IP addresses), feeds them to specialist agents who analyze patterns. The regex tool is the parser at the ingestion boundary.

```dpn
@pipeline{
  source: @read_file{path: "/var/log/app.log"},
  stages: [
    @extract_groups{pattern: "\\[([\\d-]+) ([\\d:]+)\\] (\\w+): (.+)"},
    @filter{field: "level", value: "ERROR"},
    @group_by{field: "date"},
    @count
  ]
}
-> {
  "2026-04-15": 3,
  "2026-04-16": 7
}
```

The regex tool sits at the boundary between raw bytes and structured data. Every ingestion pipeline starts here. The specialist agents downstream never see raw text -- they see structured records, because the regex tool parsed them at the gate. This is the same architecture as the dpn-api-client: raw IPC bytes come in, structured JSON goes out. The regex is the parser. The groups are the schema. The matches are the records.

## The capstone insight

The Text category (G016-G032) is complete. It discovered that all text operations reduce to pattern matching on strings. The regex tool is not just another utility -- it is the CATEGORY PRIMITIVE. Every tool in the Text category is a specialization of regex:

- **reverse_string**: regex can detect palindromes, but reversal is below regex (it's character-level)
- **count_vowels/words**: `@regex{pattern: "[aeiou]"}.length`
- **palindrome**: `@reverse` + `@is_match`
- **pig_latin**: `@regex_replace{pattern: "\\b([^aeiou]*)(\\w+)", replacement: "$2$1ay"}`
- **ciphers**: `@regex_replace` with a character translation map
- **rss_feed**: `@extract_groups` with XML tag patterns
- **text_editor**: `@regex_replace` for find-and-replace
- **text_to_html**: `@regex_replace` for each markup rule
- **cd_key_generator**: `@is_match` for validation

The Text category ends where it began: with pattern matching on strings. The circle closes.
