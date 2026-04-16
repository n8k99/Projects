/- Post it Notes Program — create, manage, and search notes. -/

namespace Text

/-- A single note with id, title, content, and optional color. -/
structure Note where
  id        : Nat
  title     : String
  content   : String
  createdAt : Nat := 0
  color     : Option String := none
  deriving Repr, BEq

/-- A collection of notes with CRUD and search operations. Pure functional: operations return new board state. -/
structure NoteBoard where
  notes  : Array Note := #[]
  nextId : Nat := 1
  deriving Repr

/-- Create a new empty note board. -/
def NoteBoard.empty : NoteBoard := {}

/-- Add a note with the given title and content. Returns (id, newBoard). -/
def NoteBoard.addNote (board : NoteBoard) (title content : String) (color : Option String := none)
    : Nat × NoteBoard :=
  let id := board.nextId
  let note : Note := { id := id, title := title, content := content, color := color }
  (id, { notes := board.notes.push note, nextId := id + 1 })

/-- Get a note by id. Returns none if not found. -/
def NoteBoard.getNote (board : NoteBoard) (id : Nat) : Option Note :=
  board.notes.find? (fun n => n.id == id)

/-- Update the content of a note by id. Returns the new board. -/
def NoteBoard.updateNote (board : NoteBoard) (id : Nat) (content : String) : NoteBoard :=
  { board with notes := board.notes.map fun n =>
      if n.id == id then { n with content := content } else n }

/-- Delete a note by id. Returns the new board. -/
def NoteBoard.deleteNote (board : NoteBoard) (id : Nat) : NoteBoard :=
  { board with notes := board.notes.filter fun n => n.id != id }

/-- List all notes (already sorted by insertion order). -/
def NoteBoard.listNotes (board : NoteBoard) : Array Note :=
  board.notes

/-- Search notes by title or content. Simple substring containment check. -/
private def containsSubstr (haystack needle : String) : Bool :=
  let h := haystack.toLower
  let n := needle.toLower
  h.containsSubstr n

/-- Search notes where title or content contains the query (case-insensitive). -/
def NoteBoard.searchNotes (board : NoteBoard) (query : String) : Array Note :=
  board.notes.filter fun n =>
    containsSubstr n.title query || containsSubstr n.content query

/-- Return the number of notes on the board. -/
def NoteBoard.size (board : NoteBoard) : Nat :=
  board.notes.size

/-- Return true if the board has no notes. -/
def NoteBoard.isEmpty (board : NoteBoard) : Bool :=
  board.notes.isEmpty

end Text
