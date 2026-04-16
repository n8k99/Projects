/- Text Editor — buffer model with cursor, edit operations, and undo. -/

namespace Text

/-- Cursor position in the buffer. -/
structure CursorPos where
  line : Nat := 0
  col  : Nat := 0
  deriving Repr, BEq

/-- Direction for cursor movement. -/
inductive Direction where
  | up | down | left | right
  deriving Repr, BEq

/-- A recorded edit action for the undo stack. -/
inductive EditAction where
  | insertChar  (cursor : CursorPos) (ch : Char)
  | deleteChar  (cursor : CursorPos) (ch : Char)
  | deleteCharJoin (cursor : CursorPos) (removedLine : String) (joinCol : Nat)
  | insertLine  (cursor : CursorPos)
  | deleteLine  (cursor : CursorPos) (content : String) (index : Nat)
  | replace     (cursor : CursorPos) (oldLines : Array String)
  deriving Repr

/-- The editor state: buffer lines, cursor, and undo stack. -/
structure EditorState where
  lines     : Array String := #[""]
  cursor    : CursorPos := {}
  undoStack : List EditAction := []
  deriving Repr

/-- Create an editor from text. -/
def EditorState.ofText (text : String) : EditorState :=
  if text.isEmpty then
    { lines := #[""], cursor := {}, undoStack := [] }
  else
    let parts := text.splitOn "\n"
    { lines := parts.toArray, cursor := {}, undoStack := [] }

/-- Clamp cursor to valid bounds. -/
def EditorState.clampCursor (s : EditorState) : EditorState :=
  let maxLine := s.lines.size - 1
  let line := min s.cursor.line maxLine
  let lineLen := (s.lines[line]!).length
  let col := min s.cursor.col lineLen
  { s with cursor := { line, col } }

/-- Get content of line n. -/
def EditorState.getLine (s : EditorState) (n : Nat) : Option String :=
  if n < s.lines.size then some s.lines[n]! else none

/-- Get full buffer content. -/
def EditorState.getText (s : EditorState) : String :=
  "\n".intercalate s.lines.toList

/-- Insert a character at the cursor. -/
def EditorState.insertChar (s : EditorState) (ch : Char) : EditorState :=
  let line := s.lines[s.cursor.line]!
  let before := line.take s.cursor.col
  let after := line.drop s.cursor.col
  let newLine := before ++ ch.toString ++ after
  let newLines := s.lines.set! s.cursor.line newLine
  { lines := newLines
    cursor := { line := s.cursor.line, col := s.cursor.col + 1 }
    undoStack := EditAction.insertChar s.cursor ch :: s.undoStack }

/-- Delete character before cursor (backspace). -/
def EditorState.deleteChar (s : EditorState) : EditorState :=
  if s.cursor.col == 0 && s.cursor.line == 0 then s
  else if s.cursor.col > 0 then
    let line := s.lines[s.cursor.line]!
    let deleted := line.get ⟨s.cursor.col - 1⟩
    let newLine := (line.take (s.cursor.col - 1)) ++ (line.drop s.cursor.col)
    let newLines := s.lines.set! s.cursor.line newLine
    { lines := newLines
      cursor := { line := s.cursor.line, col := s.cursor.col - 1 }
      undoStack := EditAction.deleteChar s.cursor deleted :: s.undoStack }
  else
    let prev := s.cursor.line - 1
    let joinCol := (s.lines[prev]!).length
    let removed := s.lines[s.cursor.line]!
    let joined := s.lines[prev]! ++ removed
    let newLines := (s.lines.set! prev joined).eraseIdx ⟨s.cursor.line⟩
    { lines := newLines
      cursor := { line := prev, col := joinCol }
      undoStack := EditAction.deleteCharJoin s.cursor removed joinCol :: s.undoStack }

/-- Split current line at cursor (Enter). -/
def EditorState.insertLine (s : EditorState) : EditorState :=
  let line := s.lines[s.cursor.line]!
  let before := line.take s.cursor.col
  let after := line.drop s.cursor.col
  let newLines := (s.lines.set! s.cursor.line before).insertAt! (s.cursor.line + 1) after
  { lines := newLines
    cursor := { line := s.cursor.line + 1, col := 0 }
    undoStack := EditAction.insertLine s.cursor :: s.undoStack }

/-- Remove the current line. -/
def EditorState.deleteLine (s : EditorState) : EditorState :=
  let content := s.lines[s.cursor.line]!
  if s.lines.size == 1 then
    { lines := #[""]
      cursor := { line := 0, col := 0 }
      undoStack := EditAction.deleteLine s.cursor content s.cursor.line :: s.undoStack }
  else
    let newLines := s.lines.eraseIdx ⟨s.cursor.line⟩
    let newLine := if s.cursor.line >= newLines.size then newLines.size - 1 else s.cursor.line
    { lines := newLines
      cursor := { line := newLine, col := 0 }
      undoStack := EditAction.deleteLine s.cursor content s.cursor.line :: s.undoStack }

/-- Move cursor in a direction. -/
def EditorState.moveCursor (s : EditorState) (dir : Direction) : EditorState :=
  match dir with
  | .left =>
    if s.cursor.col > 0 then
      { s with cursor := { line := s.cursor.line, col := s.cursor.col - 1 } }
    else if s.cursor.line > 0 then
      let prevLine := s.cursor.line - 1
      { s with cursor := { line := prevLine, col := (s.lines[prevLine]!).length } }
    else s
  | .right =>
    let lineLen := (s.lines[s.cursor.line]!).length
    if s.cursor.col < lineLen then
      { s with cursor := { line := s.cursor.line, col := s.cursor.col + 1 } }
    else if s.cursor.line < s.lines.size - 1 then
      { s with cursor := { line := s.cursor.line + 1, col := 0 } }
    else s
  | .up =>
    if s.cursor.line > 0 then
      let newState := { s with cursor := { line := s.cursor.line - 1, col := s.cursor.col } }
      newState.clampCursor
    else s
  | .down =>
    if s.cursor.line < s.lines.size - 1 then
      let newState := { s with cursor := { line := s.cursor.line + 1, col := s.cursor.col } }
      newState.clampCursor
    else s

/-- Move cursor to specific position. -/
def EditorState.goto (s : EditorState) (line col : Nat) : EditorState :=
  let s' := { s with cursor := { line, col } }
  s'.clampCursor

/-- Undo the last edit operation. -/
def EditorState.undo (s : EditorState) : EditorState :=
  match s.undoStack with
  | [] => s
  | action :: rest =>
    match action with
    | .insertChar cursor ch =>
      let line := s.lines[cursor.line]!
      let newLine := (line.take cursor.col) ++ (line.drop (cursor.col + 1))
      { lines := s.lines.set! cursor.line newLine
        cursor := cursor
        undoStack := rest }
    | .deleteChar cursor ch =>
      let line := s.lines[cursor.line]!
      let newLine := (line.take (cursor.col - 1)) ++ ch.toString ++ (line.drop (cursor.col - 1))
      { lines := s.lines.set! cursor.line newLine
        cursor := cursor
        undoStack := rest }
    | .deleteCharJoin cursor removedLine joinCol =>
      let prevIdx := cursor.line - 1
      let combined := s.lines[prevIdx]!
      let before := combined.take joinCol
      let newLines := (s.lines.set! prevIdx before).insertAt! cursor.line removedLine
      { lines := newLines, cursor := cursor, undoStack := rest }
    | .insertLine cursor =>
      let ln := cursor.line
      if ln + 1 < s.lines.size then
        let joined := s.lines[ln]! ++ s.lines[ln + 1]!
        let newLines := (s.lines.set! ln joined).eraseIdx ⟨ln + 1⟩
        { lines := newLines, cursor := cursor, undoStack := rest }
      else
        { s with cursor := cursor, undoStack := rest }
    | .deleteLine cursor content index =>
      let newLines :=
        if s.lines.size == 1 && s.lines[0]! == "" then
          #[content]
        else
          s.lines.insertAt! index content
      { lines := newLines, cursor := cursor, undoStack := rest }
    | .replace cursor oldLines =>
      { lines := oldLines, cursor := cursor, undoStack := rest }

/-- Find all occurrences of a pattern. Returns list of (line, col). -/
partial def EditorState.find (s : EditorState) (pattern : String) : List (Nat × Nat) :=
  let rec searchLine (line : String) (lineIdx : Nat) (start : Nat) (acc : List (Nat × Nat)) : List (Nat × Nat) :=
    if start + pattern.length > line.length then acc
    else if (line.drop start).take pattern.length == pattern then
      searchLine line lineIdx (start + 1) (acc ++ [(lineIdx, start)])
    else
      searchLine line lineIdx (start + 1) acc
  let rec go (i : Nat) (acc : List (Nat × Nat)) : List (Nat × Nat) :=
    if i >= s.lines.size then acc
    else go (i + 1) (acc ++ searchLine s.lines[i]! i 0 [])
  go 0 []

/-- Replace occurrences of pattern. Returns (new state, count). -/
partial def EditorState.replace (s : EditorState) (pattern replacement : String) (count : Int := -1) : EditorState × Nat :=
  let oldLines := s.lines
  let rec replaceLine (line : String) (remaining : Int) (acc : String) (total : Nat) : String × Nat × Int :=
    if remaining == 0 then (acc ++ line, total, 0)
    else
      match line.findSubstr? pattern with
      | none => (acc ++ line, total, remaining)
      | some idx =>
        let before := line.take idx
        let after := line.drop (idx + pattern.length)
        let newRemaining := if remaining < 0 then remaining else remaining - 1
        replaceLine after newRemaining (acc ++ before ++ replacement) (total + 1)
  let rec go (i : Nat) (lines : Array String) (total : Nat) (remaining : Int) : Array String × Nat :=
    if i >= lines.size then (lines, total)
    else if remaining == 0 then (lines, total)
    else
      let (newLine, n, newRemaining) := replaceLine lines[i]! remaining "" 0
      go (i + 1) (lines.set! i newLine) (total + n) newRemaining
  let (newLines, total) := go 0 s.lines 0 count
  if total > 0 then
    ({ lines := newLines
       cursor := s.cursor
       undoStack := EditAction.replace s.cursor oldLines :: s.undoStack }, total)
  else
    (s, 0)

-- Helper: find substring index
private def String.findSubstr? (s : String) (pattern : String) : Option Nat :=
  let rec go (i : Nat) : Option Nat :=
    if i + pattern.length > s.length then none
    else if (s.drop i).take pattern.length == pattern then some i
    else go (i + 1)
  go 0

end Text
