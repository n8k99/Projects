---
id: G024
title: Quote Tracker
domain: text
type: rosetta-stone
status: active
depends_on: [G013, G018, G021, G023]
concepts:
  - curated collection
  - multi-label classification
  - nondeterminism
  - provenance
  - metadata enrichment
  - set-valued properties
---

# G024 — Quote Tracker

A curated collection of quotes with attribution, categorization, and random retrieval.

## Insight: Curated Collection, Not Just CRUD

G023's notes tracker was basic CRUD — add, list, search. The quote tracker adds a **curation layer**: tags, sources, author attribution. These metadata fields transform a flat list into a navigable knowledge structure. The difference between a pile of notes and a library is curation — the metadata that makes content findable, relatable, and meaningful.

## Insight: Multi-Dimensional Classification via Tags

A quote can belong to multiple categories simultaneously. G018's `include_y` was a binary flag. G013's card network was single-dispatch. Tags are **multi-label classification** — each item lives in multiple overlapping sets at once. The resolver needs to support set-valued properties and multi-label queries. This is how the noosphere actually works: every entity has multiple facets, multiple roles, multiple memberships.

## Insight: Nondeterminism — The First Non-Deterministic Function

`random_quote()` is the first function in the Rosetta Stone that doesn't return the same result for the same input. Every previous function was deterministic: same input, same output. Random selection is a **new primitive** — the resolver makes a choice that isn't determined by the inputs. This connects directly to agent autonomy: when an agent chooses which approach to try, which quote to surface, which path to explore, it's making a non-deterministic selection. Nondeterminism is the seed of agency.

## Insight: Attribution is Provenance

Tracking where content came from — who said it, in what context. In the noosphere, every piece of content has **provenance**: which agent produced it, when, in what context. The quote tracker's author field is the simplest model of provenance. Source adds context — not just who, but where and when. Provenance is what makes knowledge trustable and traceable.

## Choreographic Case: Domain-Specific Knowledge Curation

Agents curate quotes relevant to their domain. Sylvia collects writing quotes — craft, voice, revision. Kathryn collects trading maxims — risk, discipline, timing. Eliana collects engineering principles — simplicity, correctness, modularity. The shared quote tracker is a **curated knowledge base** with domain-specific views via tag filtering. Each agent's `search_by_tag` call reveals their slice of the collection. The whole is richer than any agent's view.

## The Shape

```innate
(define-shape quote-tracker
  "A curated collection of quotes with attribution and categorization."

  ;; A quote: text + provenance + classification
  (define-record quote
    (text    :type string   :required t)
    (author  :type string   :required t)
    (source  :type string   :default "")
    (tags    :type (list-of string) :default ()))

  ;; The collection
  (define-state quotes :type (list-of quote) :default ())

  ;; Add a quote with full attribution
  (define-method add-quote (text author &key source tags)
    (push (make-quote :text text :author author
                      :source (or source "")
                      :tags (or tags ()))
          quotes))

  ;; Nondeterministic selection — the resolver chooses
  (define-method random-quote ()
    (when quotes
      (nth (random (length quotes)) quotes)))

  ;; Search by provenance
  (define-method search-by-author (author)
    (filter (lambda (q) (contains-substring-ci (quote-author q) author))
            quotes))

  ;; Search by classification — multi-label query
  (define-method search-by-tag (tag)
    (filter (lambda (q) (member-ci tag (quote-tags q)))
            quotes))

  ;; Enumerate the classification space
  (define-method list-authors ()
    (sort (unique (map #'quote-author quotes)) #'string<))

  (define-method list-tags ()
    (sort (unique (flatten (map #'quote-tags quotes))) #'string<))

  ;; Collection size
  (define-method count ()
    (length quotes)))
```

## Connections

- **G013 (Flash Cards)**: Cards had front/back and category. Quotes have text/author and tags. Cards were single-category; quotes are multi-label. The evolution from single to multi classification.
- **G018 (Bill Splitter)**: `include_y` was binary membership. Tags are set-valued membership — richer, more expressive.
- **G021 (Text Editor)**: The editor managed mutable state (buffer + cursor). The quote tracker manages mutable state (collection). But the editor's state changed character by character; the quote tracker's state changes quote by quote. Granularity of state change.
- **G023 (Notes)**: Notes had title + content + search. Quotes add author (provenance), source (context), and tags (classification). The metadata layer that turns storage into curation.
