---
id: G023
title: Post it Notes Program
domain: text
type: rosetta-stone
status: active
depends_on: [G021, G012]
concepts:
  - collection with CRUD
  - identity
  - query/search
  - data store
  - shared memory
  - content addressability
---

# G023 — Post it Notes Program

A simple note-taking system — create, read, update, delete notes with titles and content. The first data store in the Rosetta Stone.

## Insight: The First Data Store

G012's city table was read-only lookup. G021's text editor was a single document. The note board is a **collection of documents** with create, read, update, delete operations. This is the database pattern at its simplest. Every vault, every table, every collection is a note board with different column names. The CRUD operations are the universal interface to mutable collections.

## Insight: Identity Makes Content Addressable

Each note has an ID — the first time the Rosetta Stone assigns **identity** to content. Strings and numbers don't have identity; notes do. Two notes with the same content are still different notes because they have different IDs. Identity is what makes content addressable and mutable. Without an ID, you can't say "update that note" — you can only say "update all notes that look like this." Identity is the difference between a value and an entity.

## Insight: Search is the Simplest Query

Search across notes is **query** — filtering a collection by a predicate. This is the simplest database query: `SELECT * FROM notes WHERE content LIKE '%query%'`. The resolver needs query primitives because agents need to find things in collections. Search is what makes a collection useful beyond enumeration.

## Insight: The Note Board is the Vault in Miniature

Notes = vault_notes table. Add = create file. Search = grep/search. Delete = archive. The Rosetta Stone keeps building things that already exist in Nathan's infrastructure. The note board is the vault's document model stripped to its essentials. Understanding the note board is understanding the vault.

## Insight: Shared Memory for Agent Teams

Choreographic case: shared knowledge base where agents create and search notes. Sarah logs tasks as notes. Lena searches for relevant context. Eliana reviews and prunes. The note board is the **shared memory** of the agent team — a collection that multiple agents read from and write to, each with different access patterns. The CRUD interface becomes the protocol for collaborative knowledge management.

## InnateScript Sketch

```dpn
(def-struct note
  :id       (auto-id)
  :title    string
  :content  string
  :created  (timestamp)
  :color    (optional string))

(def-struct note-board
  :notes    (collection note :key :id))

;; CRUD — the universal collection interface
(def add-note [board title content &opt color]
  (let [id (next-id board)]
    (collection/insert (:notes board)
      (note :id id :title title :content content
            :created (now) :color color))
    id))

(def get-note [board id]
  (collection/get (:notes board) id))

(def update-note [board id content]
  (collection/update (:notes board) id
    (fn [note] (assoc note :content content))))

(def delete-note [board id]
  (collection/remove (:notes board) id))

(def list-notes [board]
  (collection/values (:notes board)))

(def search-notes [board query]
  (collection/filter (:notes board)
    (fn [note]
      (or (contains? (lower (:title note)) (lower query))
          (contains? (lower (:content note)) (lower query))))))

;; The collection primitives: insert, get, update, remove, values, filter.
;; These six operations define the universal data store interface.
;; Every table in the DB, every directory in the vault, every
;; agent's memory — they all implement this interface.

;; Choreographic form: shared board
(def-choreography shared-notes [board]
  (agent sarah
    (add-note board "Sprint task" "Implement auth flow")
    (add-note board "Bug report" "Login redirect fails on mobile"))
  (agent lena
    (let [relevant (search-notes board "auth")]
      (for-each relevant
        (fn [note] (analyze-context note)))))
  (agent eliana
    (let [all (list-notes board)]
      (for-each all
        (fn [note]
          (when (stale? note)
            (delete-note board (:id note))))))))
```

## Resolver Implications

The note board introduces **collection semantics** to the resolver:

- `collection/insert` — add an entity to a keyed collection
- `collection/get` — retrieve by identity (not by value)
- `collection/update` — mutate an entity in place (or return new collection in pure mode)
- `collection/remove` — delete by identity
- `collection/filter` — query with predicate

These six primitives are the foundation for every data store the resolver will ever manage. The vault, the database, the agent memory — all are collections with these operations.

The `auto-id` type is new: a value the system assigns, not the user. This is the resolver's first encounter with **system-generated identity** — the collection itself decides what ID each item gets. The resolver needs to track ID counters as part of collection state.
