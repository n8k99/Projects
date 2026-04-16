package text

import (
	"strings"
)

// CursorPos represents a position in the buffer.
type CursorPos struct {
	Line int
	Col  int
}

// actionKind identifies the type of edit for undo.
type actionKind int

const (
	actInsertChar actionKind = iota
	actDeleteChar
	actDeleteCharJoin
	actInsertLine
	actDeleteLine
	actReplace
)

// editorAction records a single edit for the undo stack.
type editorAction struct {
	kind        actionKind
	cursor      CursorPos
	char        rune
	removedLine string
	joinCol     int
	content     string
	index       int
	oldLines    []string
}

// TextEditor is a programmatic text buffer with cursor, edit operations, and undo.
type TextEditor struct {
	Lines     []string
	Cursor    CursorPos
	undoStack []editorAction
}

// NewTextEditor creates a new editor, optionally initialized with text.
func NewTextEditor(text string) *TextEditor {
	var lines []string
	if text == "" {
		lines = []string{""}
	} else {
		lines = strings.Split(text, "\n")
	}
	return &TextEditor{
		Lines:     lines,
		Cursor:    CursorPos{0, 0},
		undoStack: nil,
	}
}

func (e *TextEditor) clampCursor() {
	if e.Cursor.Line < 0 {
		e.Cursor.Line = 0
	}
	if e.Cursor.Line >= len(e.Lines) {
		e.Cursor.Line = len(e.Lines) - 1
	}
	lineLen := len([]rune(e.Lines[e.Cursor.Line]))
	if e.Cursor.Col < 0 {
		e.Cursor.Col = 0
	}
	if e.Cursor.Col > lineLen {
		e.Cursor.Col = lineLen
	}
}

// InsertChar inserts a character at the cursor position.
func (e *TextEditor) InsertChar(ch rune) {
	e.undoStack = append(e.undoStack, editorAction{
		kind:   actInsertChar,
		cursor: e.Cursor,
		char:   ch,
	})
	runes := []rune(e.Lines[e.Cursor.Line])
	newRunes := make([]rune, 0, len(runes)+1)
	newRunes = append(newRunes, runes[:e.Cursor.Col]...)
	newRunes = append(newRunes, ch)
	newRunes = append(newRunes, runes[e.Cursor.Col:]...)
	e.Lines[e.Cursor.Line] = string(newRunes)
	e.Cursor.Col++
}

// DeleteChar performs backspace at the cursor.
func (e *TextEditor) DeleteChar() {
	if e.Cursor.Col == 0 && e.Cursor.Line == 0 {
		return
	}
	if e.Cursor.Col > 0 {
		runes := []rune(e.Lines[e.Cursor.Line])
		deleted := runes[e.Cursor.Col-1]
		e.undoStack = append(e.undoStack, editorAction{
			kind:   actDeleteChar,
			cursor: e.Cursor,
			char:   deleted,
		})
		newRunes := append(runes[:e.Cursor.Col-1], runes[e.Cursor.Col:]...)
		e.Lines[e.Cursor.Line] = string(newRunes)
		e.Cursor.Col--
	} else {
		prev := e.Cursor.Line - 1
		joinCol := len([]rune(e.Lines[prev]))
		removed := e.Lines[e.Cursor.Line]
		e.undoStack = append(e.undoStack, editorAction{
			kind:        actDeleteCharJoin,
			cursor:      e.Cursor,
			removedLine: removed,
			joinCol:     joinCol,
		})
		e.Lines[prev] += removed
		e.Lines = append(e.Lines[:e.Cursor.Line], e.Lines[e.Cursor.Line+1:]...)
		e.Cursor.Line = prev
		e.Cursor.Col = joinCol
	}
}

// InsertLine splits the current line at the cursor (Enter).
func (e *TextEditor) InsertLine() {
	e.undoStack = append(e.undoStack, editorAction{
		kind:   actInsertLine,
		cursor: e.Cursor,
	})
	runes := []rune(e.Lines[e.Cursor.Line])
	before := string(runes[:e.Cursor.Col])
	after := string(runes[e.Cursor.Col:])
	e.Lines[e.Cursor.Line] = before
	// Insert after
	newLines := make([]string, 0, len(e.Lines)+1)
	newLines = append(newLines, e.Lines[:e.Cursor.Line+1]...)
	newLines = append(newLines, after)
	newLines = append(newLines, e.Lines[e.Cursor.Line+1:]...)
	e.Lines = newLines
	e.Cursor.Line++
	e.Cursor.Col = 0
}

// DeleteLine removes the current line.
func (e *TextEditor) DeleteLine() {
	e.undoStack = append(e.undoStack, editorAction{
		kind:    actDeleteLine,
		cursor:  e.Cursor,
		content: e.Lines[e.Cursor.Line],
		index:   e.Cursor.Line,
	})
	if len(e.Lines) == 1 {
		e.Lines[0] = ""
		e.Cursor.Col = 0
	} else {
		e.Lines = append(e.Lines[:e.Cursor.Line], e.Lines[e.Cursor.Line+1:]...)
		if e.Cursor.Line >= len(e.Lines) {
			e.Cursor.Line = len(e.Lines) - 1
		}
		e.Cursor.Col = 0
	}
}

// MoveCursor moves the cursor: "up", "down", "left", "right".
func (e *TextEditor) MoveCursor(direction string) {
	switch direction {
	case "left":
		if e.Cursor.Col > 0 {
			e.Cursor.Col--
		} else if e.Cursor.Line > 0 {
			e.Cursor.Line--
			e.Cursor.Col = len([]rune(e.Lines[e.Cursor.Line]))
		}
	case "right":
		lineLen := len([]rune(e.Lines[e.Cursor.Line]))
		if e.Cursor.Col < lineLen {
			e.Cursor.Col++
		} else if e.Cursor.Line < len(e.Lines)-1 {
			e.Cursor.Line++
			e.Cursor.Col = 0
		}
	case "up":
		if e.Cursor.Line > 0 {
			e.Cursor.Line--
			e.clampCursor()
		}
	case "down":
		if e.Cursor.Line < len(e.Lines)-1 {
			e.Cursor.Line++
			e.clampCursor()
		}
	}
}

// Goto moves the cursor to a specific position.
func (e *TextEditor) Goto(line, col int) {
	e.Cursor.Line = line
	e.Cursor.Col = col
	e.clampCursor()
}

// GetLine returns the content of line n.
func (e *TextEditor) GetLine(n int) string {
	if n >= 0 && n < len(e.Lines) {
		return e.Lines[n]
	}
	return ""
}

// GetText returns the full buffer content.
func (e *TextEditor) GetText() string {
	return strings.Join(e.Lines, "\n")
}

// Undo reverts the last edit operation.
func (e *TextEditor) Undo() {
	if len(e.undoStack) == 0 {
		return
	}
	action := e.undoStack[len(e.undoStack)-1]
	e.undoStack = e.undoStack[:len(e.undoStack)-1]

	switch action.kind {
	case actInsertChar:
		runes := []rune(e.Lines[action.cursor.Line])
		runes = append(runes[:action.cursor.Col], runes[action.cursor.Col+1:]...)
		e.Lines[action.cursor.Line] = string(runes)
	case actDeleteChar:
		runes := []rune(e.Lines[action.cursor.Line])
		col := action.cursor.Col - 1
		newRunes := make([]rune, 0, len(runes)+1)
		newRunes = append(newRunes, runes[:col]...)
		newRunes = append(newRunes, action.char)
		newRunes = append(newRunes, runes[col:]...)
		e.Lines[action.cursor.Line] = string(newRunes)
	case actDeleteCharJoin:
		prevLine := action.cursor.Line - 1
		combined := e.Lines[prevLine]
		runesCombined := []rune(combined)
		before := string(runesCombined[:action.joinCol])
		e.Lines[prevLine] = before
		newLines := make([]string, 0, len(e.Lines)+1)
		newLines = append(newLines, e.Lines[:action.cursor.Line]...)
		newLines = append(newLines, action.removedLine)
		newLines = append(newLines, e.Lines[action.cursor.Line:]...)
		e.Lines = newLines
	case actInsertLine:
		ln := action.cursor.Line
		if ln+1 < len(e.Lines) {
			e.Lines[ln] += e.Lines[ln+1]
			e.Lines = append(e.Lines[:ln+1], e.Lines[ln+2:]...)
		}
	case actDeleteLine:
		if len(e.Lines) == 1 && e.Lines[0] == "" {
			e.Lines[0] = action.content
		} else {
			newLines := make([]string, 0, len(e.Lines)+1)
			newLines = append(newLines, e.Lines[:action.index]...)
			newLines = append(newLines, action.content)
			newLines = append(newLines, e.Lines[action.index:]...)
			e.Lines = newLines
		}
	case actReplace:
		e.Lines = action.oldLines
	}
	e.Cursor = action.cursor
}

// Find returns all positions where pattern occurs as (line, col) pairs.
func (e *TextEditor) Find(pattern string) []CursorPos {
	var results []CursorPos
	for i, line := range e.Lines {
		start := 0
		for {
			idx := strings.Index(line[start:], pattern)
			if idx == -1 {
				break
			}
			// Convert byte offset to rune offset
			runeCol := len([]rune(line[:start+idx]))
			results = append(results, CursorPos{Line: i, Col: runeCol})
			start += idx + 1
		}
	}
	return results
}

// Replace replaces occurrences of pattern with replacement.
// count=-1 replaces all. Returns number of replacements made.
func (e *TextEditor) Replace(pattern, replacement string, count int) int {
	oldLines := make([]string, len(e.Lines))
	copy(oldLines, e.Lines)

	total := 0
	for i := range e.Lines {
		if count >= 0 && total >= count {
			break
		}
		if count < 0 {
			n := strings.Count(e.Lines[i], pattern)
			e.Lines[i] = strings.ReplaceAll(e.Lines[i], pattern, replacement)
			total += n
		} else {
			remaining := count - total
			line := e.Lines[i]
			var result strings.Builder
			replaced := 0
			for replaced < remaining {
				idx := strings.Index(line, pattern)
				if idx == -1 {
					break
				}
				result.WriteString(line[:idx])
				result.WriteString(replacement)
				line = line[idx+len(pattern):]
				replaced++
			}
			result.WriteString(line)
			e.Lines[i] = result.String()
			total += replaced
		}
	}

	if total > 0 {
		e.undoStack = append(e.undoStack, editorAction{
			kind:     actReplace,
			cursor:   e.Cursor,
			oldLines: oldLines,
		})
	}
	return total
}
