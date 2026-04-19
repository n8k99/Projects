# G056 — Image Gallery

> The entity's primary content lives outside the entity. Identity-by-content via hash. Tag queries are set algebra. Albums are curated; tags are intrinsic.

```yaml
id: G056
title: Image Gallery
category: classes
requires: [G048-product-inventory, G053-library-catalog]
provides: [external-blob-ref, content-addressed-identity, set-algebra-tags, curated-vs-intrinsic]
```

## Insight: The Entity Is a Metadata Envelope Around External State

Every prior entity in the Rosetta Stone was self-contained. The product had a price; that price *was* the datum. The recipe had lines; those lines *were* the recipe. G056 introduces **external references**: the image's bytes live at a path on disk, or at a URL, or on S3. The `Image` record is a description — filename, dimensions, mime type, a content hash, and a *pointer* (`BlobRef`) to where the bytes actually are.

The database doesn't own the bytes. It doesn't know if the file still exists at the path, if it's been modified, if the bytes match the hash. The entity is a *pointer*, not the content. This is the first stale-reference primitive in the repo: `BlobRef.exists` is a last-known-availability flag, not a live property. Callers re-check before fetching.

This is how every vault references the outside world. A daily note references a screenshot at some filesystem path. A conversation references an article by URL. A project references a codebase directory. An annotation references a Figma design. Whenever authoritative state lives elsewhere, you need stale-reference semantics, and the entity becomes a metadata envelope.

## Insight: Derived Artifacts Are Cache Entries, Not Canonical State

A thumbnail is a function of the source image. A preview is a function of the source image. If the source changes, they become stale and need regeneration. The source is *canonical*; the derivatives are *cache views*.

G055's scaling was pure — `scaled(recipe, 2)` produced a new value from an old one, no caching involved. G056's thumbnail relationship is similar in structure (derivation from source) but **materialized** (the result persists). This introduces the first implicit cache in the Rosetta Stone and therefore the first implicit staleness question: when was the thumbnail generated? Does it still match the source hash? If not, regenerate.

In the noosphere: every "rendered" artifact is a cache of its source. A project summary rendered from YAML frontmatter is a cache. A compiled script from a choreography definition is a cache. A daily-note PDF export is a cache. G056 doesn't solve cache invalidation — it names the pattern. Source identity (content hash) is the invalidation key.

## Insight: Identity by Content, Not by Assignment

A `content_hash` field turns the image into a **content-addressed** entity. Two uploads with the same bytes have the same hash and are, for dedup purposes, the same content. This is a different identity model from everything before:

- G048–G055: identity by *assignment* — `"SKU-001"` is whatever product was first assigned that SKU. IDs are external and arbitrary.
- G056: identity by *content* — the hash is computed from the bytes. Same bytes, same id.

Git blobs work this way. The vault's conversations could dedup quote-replies by content hash. Any time you want to answer "is this the same stuff?" without comparing all the bytes, you want content addressing. The hash is a fingerprint; two records with the same fingerprint are the same thing, even if they live in different places, belong to different users, or arrived via different paths.

`duplicates()` is the query this primitive enables: group by hash, return hashes with more than one image. A vault with content-addressed conversations could run the same query to detect re-used quotes. A codebase with content-addressed blobs runs this to detect copy-paste. G056 shows the pattern in its simplest form; the rest of the noosphere will reuse it.

## Insight: Tag Queries Are Set Algebra, Not Tree Navigation

G053's library subjects formed a **tree**: queries were prefix matches over a dot-delimited path. G056's tags form a **flat set**: each image has a bag of labels, each label shared across many images. Queries are set operations:

```
with_all_tags({travel, beach})    -> T ∩ B   (intersection — both required)
with_any_tag({travel, beach})     -> T ∪ B   (union — either qualifies)
without_tags({travel}, {private}) -> T \ P   (difference — travel minus private)
```

Tree queries navigate hierarchy; set queries combine orthogonal axes. Both are needed — and in the vault, both already exist. A conversation at `[[The Forge/Temporal/Daily]]` lives in a tree AND has tags `{daily, music-related, urgent}` in a flat set. The two query shapes compose: "all urgent daily notes under The Forge" is a subject-prefix filter followed by an all-tags filter.

InnateScript will need both query primitives. `@scope{path: "The Forge/..."}` is tree navigation. `@filter{tags: {urgent, daily}}` is set algebra. Together they form a richer query language than either alone.

## Insight: Tags Are Intrinsic; Albums Are Curated

An image **is** tagged "sunset" because its content depicts a sunset. The tag is a property of the image itself. Plug the image into a classifier and the tag falls out.

An image **belongs to** "Summer 2026 Trip" because a human grouped it there, intentionally, for a purpose. The album membership is not inherent to the image — it is an act of curation.

Two distinct models of collection:

| | Tags (intrinsic) | Albums (curated) |
|---|---|---|
| Origin | Property of the content | Decision by a curator |
| Auto-inferrable? | Often yes (classification) | No — requires intent |
| Ordering | Set (unordered) | Sequence (curator order) |
| Overlap | Many images share a tag | An image can be in many albums |
| Removal | Untagging is a statement about content | Removing is a statement about the collection |

Every tag-able entity in the noosphere has this distinction. A conversation's topic tags (intrinsic) vs its appearance in the "Urgent Follow-Ups" queue (curated). A project's domain tags (intrinsic) vs its inclusion in Q3 planning (curated). Muddling the two is a category error: you can't auto-infer curated membership, and you shouldn't require human intent for intrinsic classification. G056 makes the distinction structural: tags live on the image record; albums are their own entity with explicit curator and order.

## Choreographic Case: Trip Report with Content Dedup

```innate
(@weekly-trip-report){
  @candidates <- @gallery/with-all-tags{tags: ["travel", @week-tag]}
                   .without({tags: ["private"]})

  @unique <- @gallery/dedup-by-hash{images: @candidates}

  @album <- @gallery/create-album{
    id: @week-id,
    name: "Trip — " + @week-name,
    curator: @user,
    image-ids: @unique.map(.id)
  }

  concurrent {
    @sarah/draft-captions{images: @unique}
    @ellie/oversight{album: @album}
  } join as @draft

  where {
    no_private_leaked:    @album.images.none(.tags contains "private")
    duplicates_collapsed: @unique.length == @unique.distinct_hashes.length
    curator_matches:      @album.curator == @user
  }
}
```

Set algebra finds candidates. Content hash dedups. An album is curated from the survivors. The `where` catches leaks (private images) and dedup failures. Every operation uses exactly one primitive G056 introduces.

## Structures

```innate
(defstruct blob-ref
  path        : String
  size-bytes  : Nat
  exists      : Bool)                         ;; last-known

(defstruct image
  id            : String
  filename      : String
  source        : BlobRef                     ;; the bytes live here, not in the record
  thumbnail     : BlobRef?                    ;; cache — derivable from source
  preview       : BlobRef?                    ;; cache
  width         : Nat
  height        : Nat
  mime-type     : String
  content-hash  : String                      ;; identity-by-content
  tags          : Set<String>)                ;; intrinsic classification

(defstruct album
  id          : String
  name        : String
  image-ids   : [String]                      ;; ordered, curator-controlled
  curator     : String)

(defstruct gallery
  images : {String -> Image}
  albums : {String -> Album})
```

## Resolver Natives

```innate
@gallery{}                                                    -> Gallery
@gallery/add-image{image}                                     -> Gallery
@gallery/tag{image-id, ...tags}                               -> Image
@gallery/untag{image-id, ...tags}                             -> Image
@gallery/with-all-tags{tags}                                  -> [Image]   ;; intersection
@gallery/with-any-tag{tags}                                   -> [Image]   ;; union
@gallery/without-tags{include, exclude}                       -> [Image]   ;; difference
@gallery/duplicates                                           -> {Hash -> [ImageId]}
@gallery/find-by-hash{content-hash}                           -> [Image]
@gallery/create-album{album}                                  -> Album
@gallery/add-to-album{album-id, image-id}                     -> Album
@gallery/album-contents{album-id}                             -> [Image]
```

## Demo

```innate
(@demo){
  @g <- @gallery{}
    .add-image{id: "IMG-A", filename: "beach.jpg",
               source: @blob-ref{path: "/photos/beach.jpg", size-bytes: 4200000},
               content-hash: "a1b2c3",
               tags: {"sunset", "beach", "travel"}}
    .add-image{id: "IMG-B", filename: "beach-copy.jpg",
               source: @blob-ref{path: "/photos/beach-copy.jpg", size-bytes: 4200000},
               content-hash: "a1b2c3",          ;; same content
               tags: {"beach"}}

  @dups <- @g/duplicates
  ;; {"a1b2c3" -> ["IMG-A", "IMG-B"]}

  @travel-but-public <- @g/without-tags{include: ["travel"], exclude: ["private"]}
}
```

## Where

The entity MUST carry a BlobRef, not the bytes. The `content_hash` MUST be the dedup primary key for identity-by-content queries. Tag queries MUST implement the three set operations (all/any/without) as distinct primitives rather than requiring the caller to compose them. Albums MUST preserve curator-order; tags MUST remain an unordered set. Those four rules are what separates a gallery from a checkbox list of files.
