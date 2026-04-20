# G108 — Address Book

> The Rosetta Stone's eighth **Databases project**. Introduces the **contact deduplication + fuzzy merge** pattern that every CRM, address-book app, and mailing-list tool eventually builds. The distinctive moves: **canonical identifiers** (gmail-style dot-stripping on emails, digit-only phones with US `+1` normalisation) and **union-find grouping** (two contacts share an identifier → same group, transitively). Merging preserves the union of fields with a longest-name heuristic and earliest-timestamp preservation.

```yaml
id: G108
title: Address Book
category: databases
requires: [G047-dictionary, G057-hash-table, G093-mp3-tagger, G103-card-collector]
provides: [canonical-identifier-normalisation, union-find-deduplication, field-union-merge, longest-name-heuristic]
```

## Insight: Deduplication Requires Canonicalisation

Two contacts with emails `Alice.Foo@Gmail.com` and `alicefoo@gmail.com` are the same person to gmail's routing, but not to naïve string equality. A correct address book maintains a **canonical form** of every identifier — lowercase, trimmed, gmail-dot-stripped, `+tag`-stripped — and compares on that form.

G108 canonicalises per field type:
* **Email**: lowercase, trim, strip gmail dots, strip `+tag`, rewrite `googlemail.com` → `gmail.com`.
* **Phone**: digit-only, drop leading `1` for 11-digit US numbers.

The original is preserved for display; the canonical form powers comparison. This is the pattern iCloud, Google Contacts, Salesforce, Outlook, every serious contacts system uses.

First Rosetta Stone project where **the same data has two representations — display form and canonical form — and comparisons always happen on canonical**. G093's tag store had exact-match queries; G108 defines an equivalence relation on strings.

## Insight: Duplicate Grouping Is Union-Find

Two contacts can share an email OR a phone. Transitively: if A and B share a phone and B and C share an email, then A, B, and C should be one group. This is the classic **union-find** problem.

G108's `find_duplicates` builds an index per identifier (`by_email: email → {ids}`, `by_phone: phone → {ids}`), then for each index entry, unions all the IDs that share it. The union-find structure (parent pointers with path compression) collapses the transitive closure in near-linear time.

First Rosetta Stone project that **uses union-find as a primary algorithm**. G097's point-in-polygon was a named algorithm; G108's union-find is another. Classic algorithms translate cleanly across languages; their cross-language parity is a confidence test.

## Insight: Merge Is Union of Fields + Longest Name

When the user accepts a duplicate group, merging produces **one contact** with:
* **Union of emails**, **phones**, **tags** (deduplicated, sorted).
* **Longest non-empty name** — heuristic: `"Alice J. Smith"` beats `"Alice"`.
* **Earliest creation timestamp** — the contact has existed since the first of its sources.
* **Concatenated addresses and notes** — joined with `|` and `---` respectively.

Why longest name? Because duplicates often accumulate richer metadata over time — one record has the full name, another has just a first name. Preserving the fuller form recovers detail.

First Rosetta Stone project where **merge produces new data from multiple sources** with explicit conflict-resolution rules (longest wins for name, earliest wins for timestamp, union wins for lists, concat wins for free text).

## Insight: The Original Survives Canonicalisation

G108 stores the original `emails: ["Alice.Foo@Gmail.com"]` and computes canonical forms on demand via `canonical_emails()`. Canonicalisation is **derived**, not stored.

If stored, what happens when the canonicalisation rules change (new gmail policy, new phone format)? All records become stale. By deriving on demand, rule updates are a library deploy, not a database migration.

First Rosetta Stone project where **a computed view is the comparison basis while the source form is the source of truth**. G095's spreadsheet formulae derived values; G108 derives canonical identifiers — same shape, different domain.

## Insight: Synthetic IDs Let Merged Contacts Round-Trip

Merging contacts 1 and 2 doesn't reuse either ID — the merged contact gets a new ID (say 47). The sources are deleted. Why:

* **Auditability** — "contact 47 was merged from 1+2" is explicit, not implicit.
* **Reversibility** — storing the merge as a new record with a merge-history field (not shown in G108) lets undo be "delete 47, restore 1 and 2".
* **Reference stability** — external systems referencing contact 1 would break if 1 got silently repurposed.

First Rosetta Stone project where **merge is a creation operation, not a mutation**. G092's bulk renamer mutated in place; G108's merge creates fresh records and removes sources. Different mutation model for a different domain.

## Insight: Search Goes Through the Same Canonicalisation

`search_by_email(needle)` canonicalises the needle and matches against contacts' canonical forms. Search for `Alice.Foo+work@GMAIL.com` correctly finds a contact stored as `alicefoo@gmail.com`.

This is the same pipeline the duplicate detector uses, which means the invariant "searching for a stored email always finds it" holds trivially — as long as the canonicaliser is deterministic and idempotent.

First Rosetta Stone project where **a read-side and write-side both go through the same normalisation function**. Same pattern G100 used (canonical encoding of trees for content addressing). Normalisation functions reused for both directions are the robust choice.

## Choreographic Case: Vault People Gallery

```innate
(@vault-people-gallery){
  @ab <- @contacts/load{path: "people/*.vcard"}
  @dupes <- @contacts/find-duplicates{book: @ab}
  @for group in @dupes {
    @names <- @group.map(@id => @contacts/get{book: @ab, id: @id}.name)
    @ui/suggest-merge{ids: @group, names: @names}
  }

  @on-user-confirms-merge (@ids){
    @merged-id <- @contacts/merge{book: @ab, ids: @ids}
    @contacts/save{book: @ab, path: "people/${@merged-id}.vcard"}
    @for id in @ids { @fs/delete{path: "people/${@id}.vcard"} }
  }

  @on-search (@query){
    @by-name <- @contacts/search-by-name{book: @ab, needle: @query}
    @by-email <- @contacts/search-by-email{book: @ab, email: @query}
    @ui/render-results{ids: @by-name ∪ @by-email}
  }
}
```

Import finds duplicate suggestions; user confirms merges; searches span name and canonical email. Entire pipeline is the address-book primitives — no ORM, no CRM, just functions.

## Structures

```innate
(defstruct contact
  id          : Int
  name        : String
  created-ms  : Int
  emails      : [String]
  phones      : [String]
  tags        : [String]
  address     : String
  notes       : String)

(defstruct address-book
  contacts : {Int -> Contact}
  next-id  : Int)
```

## Resolver Natives

```innate
@contacts/canonical-email{email}                -> String
@contacts/canonical-phone{phone}                -> String
@contacts/new{}                                 -> AddressBook
@contacts/add{book, contact}                    -> Int     ;; returns id
@contacts/get{book, id}                         -> Contact | null
@contacts/find-duplicates{book}                 -> [[Int]]
@contacts/merge{book, ids}                      -> Int | Error
@contacts/search-by-name{book, needle}          -> [Int]
@contacts/search-by-email{book, email}          -> [Int]
```

## Demo

```innate
(@demo){
  @ab <- @contacts/new{}
  @contacts/add{book: @ab, contact: {id: 1, name: "Alice", created-ms: 100,
                                       emails: ["alice@gmail.com"],
                                       phones: ["555-1111"]}}
  @contacts/add{book: @ab, contact: {id: 2, name: "Alice J. Smith", created-ms: 200,
                                       emails: ["a.l.i.c.e@gmail.com"],
                                       phones: ["555-2222"]}}

  @contacts/find-duplicates{book: @ab}
  ;; -> [[1, 2]]  (same person — canonical gmail match)

  @merged <- @contacts/merge{book: @ab, ids: [1, 2]}
  @contacts/get{book: @ab, id: @merged}
  ;; name: "Alice J. Smith" (longest), emails: union, created-ms: 100 (earliest)
}
```

## Where

Canonical forms MUST be derived, NOT stored — rule updates shouldn't require data migration. Email canonicalisation MUST handle gmail's dot-insensitivity and `+tag` convention — these are real-world patterns that cause real false negatives otherwise. Phone canonicalisation MUST be digit-only with country-code normalisation — formatting differences (dashes, parens, spaces, dots) are presentation, not identity. Duplicate detection MUST use union-find for transitive grouping — if A shares phone with B and B shares email with C, all three are one person. Merge MUST produce a new ID, NOT reuse a source ID — auditability requires the merge event to be traceable. Longest non-empty name MUST be the merge heuristic — "Alice J. Smith" is richer than "Alice" and the user rarely wants to lose detail. Search MUST canonicalise the query — if the user types `Alice.Foo+work@GMAIL.com`, they expect to find the contact stored as `alicefoo@gmail.com`, and that only works if read and write go through the same normaliser.
