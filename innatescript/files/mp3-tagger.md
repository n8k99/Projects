# G093 — Mp3 Tagger

> The Rosetta Stone's ninth **Files-category** project. Introduces **metadata as a queryable, indexed store**. Files have tags (title, artist, album, year, genre, track); the tag store builds explicit indexes on equality-heavy fields (artist, album, year) for O(1) lookups, and scans the free-form title field. The **missing-vs-empty distinction** (Option<String>) is preserved throughout — "genre not set" and "genre set to empty string" are different states, and the query layer can filter for each.

```yaml
id: G093
title: Mp3 Tagger
category: files
requires: [G047-dictionary, G057-hash-table, G088-sort-file-records, G089-transaction-averages]
provides: [metadata-separate-from-data, indexed-queries, missing-vs-empty-distinction, parameterised-queries]
```

## Insight: Metadata Lives Separately From Data

The tag store doesn't own the files. It owns a mapping from path → tag. Files can exist on disk with no tags; tags can exist in the store for paths that were deleted. That's on purpose — the metadata layer is an independent concern from the bytes.

This pattern underpins every real metadata system: Git's object store separates content (blobs) from metadata (commit messages, tree entries); iTunes keeps a library database separate from the actual MP3 files; every package manager has a manifest file separate from the binary. **The metadata is a first-class citizen, not a property of the data.**

First Rosetta Stone project where **the data and the metadata are two different layers with their own lifecycles**. G086's launcher had metadata (count, timestamp) but only ever alongside an entry; G087's filesystem had mtime but only attached to nodes. G093 explicitly decouples: a `TagStore` operates on paths as strings, with no requirement that the paths resolve to anything.

## Insight: Indexes Trade Space for Query Speed

For a store of 10,000 tagged files, a naïve "filter all by artist" is a linear scan — 10,000 compares. An artist index (`HashMap<Artist, Vec<Path>>`) turns it into O(1) hash lookup. The cost: a copy of each (artist, path) edge in the index.

Not every field justifies an index. **Artist, album, year** are equality-queried constantly; indexes earn their keep. **Title** is substring-queried (free-form search); an index doesn't help — a trigram or suffix-array structure would, but that's overkill for the Rosetta Stone. So title queries scan. The library decides per field.

First Rosetta Stone project where **some operations are fast and some are slow by explicit design**. Previous projects either avoided indexes (G087's filesystem) or optimised everything uniformly (G085's quiz scoring). G093 makes the trade-off visible: "if you query by artist, it's fast; if you query title substrings, you're scanning."

## Insight: Missing Is Not Empty

A tag field can be `None` (user never set it) or `Some("")` (user set it to an empty string). Semantically different: the first is "unknown, please fill in", the second is "I actively set this to blank". A UI that treats them as the same is hiding information from the user.

Rust's `Option<String>`, Python's `str | None`, Go's `*string`, CL's `(or null string)`, Lean's `Option String`. Every language models this distinction natively. The tag store's `MissingField` query returns paths where the field is `None` — regardless of what the other fields contain.

First Rosetta Stone project where **nullability is a first-class part of the data contract**. Previous projects treated missing fields as sentinel values (`""`, `0`, `-1`) or errors (parse-fails). G093 makes "not set" a valid, queryable state.

## Insight: Query Is a Data Structure, Not a Function Call

Instead of `store.by_artist("Björk")`, `store.by_year(2001)`, `store.title_contains("aeon")` etc., G093 has one `query(q)` method where `q` is a `Query` enum. Every query shape is a variant.

This is a bigger deal than it looks. It means:
1. **Queries are data** — they can be stored, logged, shipped between processes, serialised into SQL-like strings.
2. **New query types add variants**, not new functions — the store's public API is one method.
3. **Compound queries become natural** — a future `And`/`Or` variant can reference other `Query` values recursively, producing the filter AST every real database has.

First Rosetta Stone project where **the query is reified**. G088's sort spec was data (list of `SortKey`). G093 takes the same step for filtering: the action is a value you construct, then execute.

## Choreographic Case: Vault Music Library

```innate
(@vault-music-library){
  @store <- @tag-store/new{}
  @mp3s <- @fs/ls{path: "Music/"}
  @for file in @mp3s {
    @tag <- @id3/parse{bytes: @file/read-bytes{path: @file.path}}
    @tag-store/insert{store: @store, path: @file.path, tag: @tag}
  }

  @on-user-types-artist (@artist-name){
    @matches <- @tag-store/query{store: @store,
                                  query: {kind: "by-artist", artist: @artist-name}}
    @ui/render-results{paths: @matches}
  }

  @on-user-clicks-fix-missing {
    @needs-year <- @tag-store/query{store: @store,
                                     query: {kind: "missing-field", field: "year"}}
    @ui/show-batch-editor{paths: @needs-year, field: "year"}
  }
}
```

A music library is a tag store. User-facing queries are parameterised by the UI's current filter; batch-fix workflows surface paths with missing metadata. The store's API is one method; the UI composes `Query` values.

## Structures

```innate
(defstruct tag
  title  : String?
  artist : String?
  album  : String?
  year   : Int?
  genre  : String?
  track  : Int?)

(defenum query-kind
  BY_ARTIST | BY_ALBUM | BY_YEAR | YEAR_RANGE | TITLE_CONTAINS | MISSING_FIELD)

(defstruct query
  kind       : QueryKind
  artist     : String
  album      : String
  year       : Int
  year-from  : Int
  year-to    : Int
  needle     : String
  field-name : String)

(defstruct tag-store
  tags        : {String -> Tag}
  artist-idx  : {String -> [String]}
  album-idx   : {String -> [String]}
  year-idx    : {Int    -> [String]})
```

## Resolver Natives

```innate
@tag-store/new{}                              -> TagStore
@tag-store/insert{store, path, tag}           -> Unit
@tag-store/get{store, path}                   -> Tag | null
@tag-store/remove{store, path}                -> Tag | null
@tag-store/query{store, query}                -> [String]
@tag/missing-fields{tag}                      -> [String]
```

## Demo

```innate
(@demo){
  @s <- @tag-store/new{}
  @tag-store/insert{store: @s, path: "a.mp3", tag: {title: "Aeon", artist: "Björk",
                                                     album: "Vespertine", year: 2001,
                                                     genre: "Electronic", track: 1}}
  @tag-store/insert{store: @s, path: "b.mp3", tag: {title: "Pagan Poetry", artist: "Björk",
                                                     album: "Vespertine", year: 2001,
                                                     genre: "Electronic", track: 3}}
  @tag-store/insert{store: @s, path: "c.mp3", tag: {title: "Unplayed"}}

  @tag-store/query{store: @s, query: {kind: "by-artist", artist: "Björk"}}
  ;; -> ["a.mp3", "b.mp3"]

  @tag-store/query{store: @s, query: {kind: "missing-field", field: "genre"}}
  ;; -> ["c.mp3"]
}
```

## Where

Metadata MUST live in a separate store from the files themselves — Git's object model, iTunes' library, every package manager follows this. Indexed fields MUST be equality-queried (artist/album/year); free-form search fields (title) MUST scan — a substring index is a different data structure and costs too much at this scale. Missing and empty MUST be distinct — `None` (unknown) and `Some("")` (explicitly blank) are different user intents; collapsing them hides information. Insert MUST refresh all indexes atomically — a half-updated index is worse than no index (the query returns stale results silently). Remove MUST clean every index — a dangling entry in an index is a phantom result. Query MUST be a data structure, not a function call per query shape — the API is one method, new query kinds are new variants, compound queries can recurse into sub-queries. Query results MUST be sorted for deterministic output.
