/-
  Guestbook / Journal — chronological append-only entry log.
  Pure functional implementation.
-/

namespace Text

/-- An immutable journal entry with author, content, and timestamp. -/
structure Entry where
  author    : String
  content   : String
  timestamp : Nat  -- epoch seconds
  deriving Repr, BEq

/-- Format an entry as "[timestamp] author: content". -/
def Entry.toText (e : Entry) : String :=
  s!"[{e.timestamp}] {e.author}: {e.content}"

instance : ToString Entry where
  toString := Entry.toText

/-- A chronological append-only journal. Entries stored oldest-first. -/
structure Journal where
  entries : List Entry := []
  deriving Repr

/-- Create an empty journal. -/
def Journal.empty : Journal := { entries := [] }

/-- Append a new entry to the journal. Returns the updated journal and entry. -/
def Journal.addEntry (j : Journal) (author content : String) (timestamp : Nat) : Journal × Entry :=
  let entry : Entry := { author, content, timestamp }
  ({ entries := j.entries ++ [entry] }, entry)

/-- Return all entries, newest first. -/
def Journal.getEntries (j : Journal) : List Entry :=
  j.entries.reverse

/-- Return entries by a specific author, newest first. -/
def Journal.getEntriesByAuthor (j : Journal) (author : String) : List Entry :=
  j.entries.reverse.filter (fun e => e.author == author)

/-- Return entries within [startTime, endTime], newest first. -/
def Journal.getEntriesInRange (j : Journal) (startTime endTime : Nat) : List Entry :=
  j.entries.reverse.filter (fun e => e.timestamp >= startTime && e.timestamp <= endTime)

/-- Return total number of entries. -/
def Journal.entryCount (j : Journal) : Nat :=
  j.entries.length

/-- Export the full journal as plain text in chronological order. -/
def Journal.exportText (j : Journal) : String :=
  "\n".intercalate (j.entries.map Entry.toText)

-- ── Demo ────────────────────────────────────────────────────────────

#eval do
  let j := Journal.empty
  let (j, _) := j.addEntry "Nathan" "Started the Rosetta Stone project." 1713200000
  let (j, _) := j.addEntry "Ghost-A" "Reviewed palindrome implementations." 1713200100
  let (j, _) := j.addEntry "Nathan" "Finished G024 quote tracker." 1713200200
  let (j, _) := j.addEntry "Ghost-B" "Ran RSS feed benchmarks." 1713200300
  let (j, _) := j.addEntry "Nathan" "Starting G025 guestbook." 1713200400

  IO.println s!"Total entries: {j.entryCount}"
  IO.println ""
  IO.println "=== All Entries (newest first) ==="
  for e in j.getEntries do
    IO.println e.toText
  IO.println ""
  IO.println "=== Nathan's Entries ==="
  for e in j.getEntriesByAuthor "Nathan" do
    IO.println e.toText
  IO.println ""
  IO.println "=== Entries in range [1713200100, 1713200300] ==="
  for e in j.getEntriesInRange 1713200100 1713200300 do
    IO.println e.toText
  IO.println ""
  IO.println "=== Export ==="
  IO.println j.exportText

end Text
