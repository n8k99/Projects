# G076 — Bookmark Collector and Sorter

> The Rosetta Stone's first project with **two orthogonal organisation axes** on the same entities — folders (hierarchy, one per bookmark) and tags (flat, many-to-many). Every design decision sits on whether to treat one as primary; G076 treats them as equal citizens. First project where **sort is always a view, never a storage property**: the stored list has one natural order, every sorted presentation is a derived projection.

```yaml
id: G076
title: Bookmark Collector and Sorter
category: web
requires: [G056-image-gallery, G064-family-tree]
provides: [multi-axis-organisation, sort-as-view, url-natural-key, folder-vs-tag-duality, stable-sort-ties]
```

## Insight: Folders and Tags Are Orthogonal — Both Are First-Class

Early bookmark systems forced a choice. Netscape's hierarchical folders won the UI war but lost the model war — the same bookmark often belongs to multiple conceptual categories ("programming" AND "rust" AND "language"), and a tree can only place it once. Tag-based systems (delicious, pinboard) went the other way and abandoned hierarchy entirely, but users still want a default-location story for where a bookmark "lives."

G076 does both. A bookmark has **one folder** (the hierarchical home; think of it as the primary filing location) and **a set of tags** (the secondary, many-to-many categorisation). Queries by folder find the filing location; queries by tag find everything with that label regardless of filing. Neither is subordinate.

First Rosetta Stone project where **two organisational axes coexist without hierarchy between them**. G056 Image Gallery had tags without folders; G064 Family Tree had hierarchy without tags. G076 is the first where both are present and intentional.

Parallels elsewhere in the vault:
- Notes have a folder path (filesystem location) AND wiki-link tags (free-floating categorisation).
- Tasks have a project (hierarchy) AND status/labels (flat sets).
- Agents have a team (hierarchy) AND capability tags.

Every real "stuff organiser" ends up with both; G076 makes the model explicit.

## Insight: URL Is the Natural Key

Adding a bookmark with an existing URL **updates** the existing entry; it doesn't create a duplicate. The URL is the natural key — two bookmarks with the same URL are the same bookmark, regardless of title, folder, tags, or when they were added. Re-add is update, not insert.

First Rosetta Stone project where **natural keys** (semantic keys the domain already provides) are preferred over **surrogate keys** (numeric ids we assign). The surrogate id exists for reference stability (moving, removing, tagging), but the identity is the URL.

This distinction matters throughout the vault. Notes have natural keys (file paths), and the vault identifies two notes as "the same" if they're at the same path — moving a note is a rename, not a delete-and-recreate. Agents have natural keys (names). Conversations have natural keys (timestamps + participants). Choosing "what makes two of these the same thing?" is the deepest schema question, and G076 answers it the way every sensible real system does: by the natural identifier, not the assigned id.

## Insight: Sort Is a View, Not a Storage Property

The collection stores bookmarks in insertion order. No "sorted by title" storage mode, no re-shuffling on add, no index. Every sort is a **view** — computed on query, consumed by the caller, discarded. The storage's natural order remains insertion order.

This separation has big implications. It means:
- Multiple callers can request different sorts without fighting over "the order."
- Adding a bookmark is O(1) — no sort maintenance.
- Sort is referentially transparent (same collection, same sort → same result).

First Rosetta Stone project where **sort is deliberately decoupled from storage**. G074 Whiteboard had the opposite — insertion order WAS the render order. G076 has neither storage order being semantic nor any sort being canonical; every sort is a lens.

**Stable sort** is the right default: when two bookmarks tie on the sort key (same title, same date), they retain their relative insertion order. The alternative (unstable sort) produces different orderings on different runs, which breaks tests and confuses users. Every language used here (Rust stable by default, Python's Timsort, Go's sort.SliceStable, CL's stable-sort, Lean's mergeSort) provides stable sort primitives; G076 uses them explicitly.

## Insight: Folder Paths as Strings, Not Trees

Folders are stored as path-like strings: `/news`, `/dev/languages`, `/personal`. No tree structure, no parent pointers, no folder entities. Queries like "bookmarks in /dev" (recursive) work by prefix-match: any bookmark whose folder is `/dev` or starts with `/dev/`.

This is a deliberately **flat representation** of a hierarchical concept, and it's what every real system uses (filesystem paths, URL paths, CSS selectors, vault wiki-links). The tree is implicit in the string structure; it's not materialised as a separate data structure.

Contrast G064 Family Tree's explicit graph: there the relationships mattered for traversal algorithms (ancestors, descendants, common ancestors). Bookmarks don't need those algorithms — "what's in this folder" is a prefix query, "move this bookmark to a different folder" is a string reassignment. Representing the tree explicitly would be expensive bookkeeping for no benefit.

First Rosetta Stone project where **a hierarchical concept is represented flatly** because no hierarchical operation is actually needed. The lesson generalises: don't materialise structure you don't query.

## Insight: Search Crosses Axes

The `search(query)` function matches against title, URL, AND tags with a single substring query. It doesn't ask "which field?" — it searches everywhere. This is what users expect from a search box; it's also what every real bookmark manager does.

First Rosetta Stone project where **the query crosses the schema boundaries**. G064 had separate ancestors/descendants/siblings queries; G067 had separate history/inbox queries; G071 selectors had precise (tag/class/id) targeting. G076 introduces the "show me anything that matches this string" query that union-scans multiple fields.

This is the pattern behind the vault's eventual full-text search: one query, many field types (note title, note body, tags, metadata). The implementation walks all fields; the user doesn't have to say which one. Matching precision can be tuned (exact vs fuzzy, case-sensitive vs not) but the union-of-fields shape is the default.

## Insight: Dedup on Add Is Soft Idempotence

Re-adding a bookmark with the same URL overwrites title/folder/tags but keeps the id and position. This is **soft idempotence**: the operation has the same identity as before (same id), but its content may differ. It's not strict idempotence (which would require exact-match re-adds to be no-ops).

The design choice: re-adding is "I want this URL here, with these properties" — not "I'm making a new bookmark that happens to have this URL." The second interpretation would require explicit dedup opt-in ("add-if-not-exists" vs "upsert"); G076 picks upsert as the default because it matches user intuition ("I already had this one, just update it").

First Rosetta Stone project with **upsert semantics by default**. Previous projects (G056 Image Gallery, G068 Bulk Thumbnail) had explicit content-addressed dedup; G076 is upsert at the API level, visible to the caller. Production REST APIs often have both (POST = insert-only, PUT = upsert); G076 picks PUT semantics for simplicity.

## Choreographic Case: Research Session Capture

```innate
(@capture-research){
  @collection <- @bookmarks/new-collection

  @on-page-visit (@page){
    @bookmarks/add{
      collection: @collection,
      url: @page.url,
      title: @page.title,
      folder: @classify-folder{topic: @session.topic},
      tags: @extract-tags{page: @page},
      added_at_ms: @now
    }
  }

  @on-session-end {
    @by-topic <- @bookmarks/by-folder{collection: @collection,
                                       folder: @session.topic-folder,
                                       recursive: true}
    @sorted-by-time <- @bookmarks/sort-by{collection: @collection,
                                            criterion: "added_at"}
    @vault/persist{path: "research/${@session.id}.md",
                    content: @render-report{
                      bookmarks: @sorted-by-time,
                      grouping: @by-topic
                    }}
  }
}
```

Capturing a research session composes on G076's primitives: every page visit is an upsert (re-visits are not duplicates), folders cluster by topic, tags capture cross-cutting concerns, final report is a sort-by-time combined with folder-based grouping.

## Structures

```innate
(defstruct bookmark
  id            : Int           ;; surrogate for references
  url           : String        ;; natural key — identity
  title         : String
  tags          : Set<String>   ;; sorted, deduplicated
  folder        : String        ;; path-like "/a/b/c"
  added-at-ms   : Int)

(defenum sort-by
  Title | URL | AddedAt | Folder)

(defstruct collection
  bookmarks     : [Bookmark]    ;; insertion order preserved
  next-id       : Int)
```

## Resolver Natives

```innate
@bookmarks/new-collection                               -> Collection
@bookmarks/add{coll, url, title, folder, tags, at_ms}   -> BookmarkId  ;; upsert
@bookmarks/remove{coll, id}                             -> Bool
@bookmarks/get{coll, id}                                -> Bookmark?
@bookmarks/set-tags{coll, id, tags}                     -> Bool
@bookmarks/move-folder{coll, id, folder}                -> Bool
@bookmarks/search{coll, query}                          -> [Bookmark]  ;; cross-field substring
@bookmarks/by-tag{coll, tag}                            -> [Bookmark]
@bookmarks/by-folder{coll, folder, recursive?}          -> [Bookmark]
@bookmarks/sort-by{coll, criterion}                     -> [Bookmark]  ;; stable view
@bookmarks/all-tags{coll}                               -> [String]    ;; sorted, dedup
@bookmarks/all-folders{coll}                            -> [String]    ;; sorted, dedup
```

## Where

URL is the natural key; add with an existing URL MUST update the existing bookmark and return its existing id — creating a duplicate would violate the domain's "same URL = same bookmark" intuition. Sort MUST be stable (ties preserve insertion order) — different runs producing different orderings breaks tests and confuses users. Sort MUST NOT mutate storage — the stored list's insertion order is the ground truth; every sort is a view. Folder recursive queries MUST prefix-match on `/` boundaries — `by-folder("/dev", recursive=true)` must match `/dev`, `/dev/languages`, `/dev/systems` but NOT `/development` (which starts with `/dev` as a substring but not on a `/` boundary). Search MUST cross title/url/tag fields with a single query — field-scoped search is a separate function (e.g., `by-tag`), not a flag on search. All-tags and all-folders MUST return sorted, deduplicated results — they're summary queries whose ordering and dedup are expected.
