# G025 — Guestbook / Journal

> Chronological append-only entry log.

## Insight

A journal is an **append-only log** — the first immutable data structure in the Rosetta Stone. G023's notes could be edited and deleted. Journal entries cannot. Once written, they are permanent. This is the event log pattern from G021's undo stack, but without the undo. The log IS the truth. This is how the vault's daily notes work — you append to today, you don't edit yesterday.

Chronological ordering makes the journal a **temporal sequence** — entries are ordered by when they were written, not by content. Time is the primary index. This connects to G011's alarm clock and the vault's temporal chain: the journal's timeline IS a temporal chain at the entry level.

Author attribution on entries is **provenance** at the entry level, extending G024's quote attribution. In the noosphere, every journal entry has an author — an agent who wrote it. The daily note's "What I Did Today" section is a journal where Nathan is the author. The ghost-tagged entries are journal entries where the ghosts are authors.

Append-only + chronological + attributed = **audit trail**. This is the pattern for compliance, debugging, and accountability. You can always answer "who wrote what, when." The vault's git history IS an audit trail.

## Schema

```innate
(def-record Entry
  "An immutable journal entry."
  :author    String
  :content   String
  :timestamp Timestamp)   ;; auto-assigned on creation

(def-record Journal
  "Chronological append-only log of entries."
  :entries (List Entry))  ;; oldest first internally
```

## Protocol

```innate
(def-protocol Journaling
  "Append-only chronological entry log."

  (add-entry [journal author content] -> [journal entry]
    "Append a new entry with auto-assigned timestamp.
     Returns the updated journal and the created entry.
     This is the ONLY mutation — and it only appends.")

  (get-entries [journal] -> (List Entry)
    "All entries, newest first.")

  (get-entries-by-author [journal author] -> (List Entry)
    "Entries by a specific author, newest first.")

  (get-entries-in-range [journal start end] -> (List Entry)
    "Entries within [start, end] timestamps, newest first.")

  (entry-count [journal] -> Nat
    "Total number of entries.")

  (export-text [journal] -> String
    "Full journal as plain text, chronological order."))
```

## Implementation

```innate
(defn make-journal []
  "Create an empty journal."
  {:entries []})

(defn add-entry [journal author content]
  "Append a new entry. Returns [updated-journal, entry]."
  (let [entry {:author author
               :content content
               :timestamp (now)}]
    [(update journal :entries #(append % [entry]))
     entry]))

(defn get-entries [journal]
  "All entries, newest first."
  (reverse (:entries journal)))

(defn get-entries-by-author [journal author]
  "Filter by author, newest first."
  (->> (get-entries journal)
       (filter #(= (:author %) author))))

(defn get-entries-in-range [journal start end]
  "Filter by timestamp range, newest first."
  (->> (get-entries journal)
       (filter #(and (>= (:timestamp %) start)
                     (<= (:timestamp %) end)))))

(defn entry-count [journal]
  "Total entries."
  (length (:entries journal)))

(defn format-entry [entry]
  "[timestamp] author: content"
  (str "[" (format-time (:timestamp entry)) "] "
       (:author entry) ": " (:content entry)))

(defn export-text [journal]
  "Full journal as plain text, chronological."
  (join "\n" (map format-entry (:entries journal))))
```

## Key Properties

```innate
;; Append-only: no remove, no edit, no update-in-place
;; The entries list only grows. This is enforced by the protocol —
;; there is no delete-entry or edit-entry operation.

;; Temporal ordering is implicit in the append sequence.
;; Entries are stored in insertion order, which IS chronological order
;; because timestamps are auto-assigned at creation time.

;; Immutability of entries: once created, an Entry's fields never change.
;; The record is frozen at creation. This is structural immutability,
;; not just API-level protection.
```

## Connections

- **G021 Undo Stack**: Both are logs. The undo stack is a log you can rewind. The journal is a log you cannot. The journal is the simpler, purer form.
- **G011 Alarm Clock**: Both are temporal. The alarm triggers at a future time. The journal records at the present time. Time as index vs. time as trigger.
- **G023 Post-it Notes**: Notes are mutable (edit, delete). Journal entries are immutable. The journal is what notes become when you want a permanent record.
- **G024 Quote Tracker**: Both have attribution (author/source). The quote tracker collects wisdom from others. The journal records your own observations. Both build provenance chains.
- **Vault Daily Notes**: A daily note IS a journal partitioned by day. The daily note template structures what the journal leaves freeform.
