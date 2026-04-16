package text

import (
	"strings"
	"time"
)

// Note represents a single note with id, title, content, timestamp, and optional color.
type Note struct {
	ID        int
	Title     string
	Content   string
	CreatedAt time.Time
	Color     string // empty string means no color
}

// NoteBoard is a collection of notes with CRUD and search operations.
type NoteBoard struct {
	notes  map[int]*Note
	nextID int
}

// NewNoteBoard creates a new empty note board.
func NewNoteBoard() *NoteBoard {
	return &NoteBoard{
		notes:  make(map[int]*Note),
		nextID: 1,
	}
}

// AddNote creates a new note with the given title and content. Returns the assigned id.
func (nb *NoteBoard) AddNote(title, content string) int {
	return nb.AddNoteWithColor(title, content, "")
}

// AddNoteWithColor creates a new note with an optional color. Returns the assigned id.
func (nb *NoteBoard) AddNoteWithColor(title, content, color string) int {
	id := nb.nextID
	nb.nextID++
	nb.notes[id] = &Note{
		ID:        id,
		Title:     title,
		Content:   content,
		CreatedAt: time.Now(),
		Color:     color,
	}
	return id
}

// GetNote retrieves a note by id. Returns the note and true, or nil and false if not found.
func (nb *NoteBoard) GetNote(id int) (*Note, bool) {
	note, ok := nb.notes[id]
	return note, ok
}

// UpdateNote updates the content of a note. Returns true if the note existed.
func (nb *NoteBoard) UpdateNote(id int, content string) bool {
	note, ok := nb.notes[id]
	if !ok {
		return false
	}
	note.Content = content
	return true
}

// DeleteNote removes a note by id. Returns true if the note existed.
func (nb *NoteBoard) DeleteNote(id int) bool {
	_, ok := nb.notes[id]
	if ok {
		delete(nb.notes, id)
	}
	return ok
}

// ListNotes returns all notes sorted by id.
func (nb *NoteBoard) ListNotes() []*Note {
	result := make([]*Note, 0, len(nb.notes))
	// Collect and sort by id
	for _, note := range nb.notes {
		result = append(result, note)
	}
	// Simple insertion sort — small collections
	for i := 1; i < len(result); i++ {
		key := result[i]
		j := i - 1
		for j >= 0 && result[j].ID > key.ID {
			result[j+1] = result[j]
			j--
		}
		result[j+1] = key
	}
	return result
}

// SearchNotes searches notes by title or content (case-insensitive).
func (nb *NoteBoard) SearchNotes(query string) []*Note {
	q := strings.ToLower(query)
	var result []*Note
	for _, note := range nb.notes {
		if strings.Contains(strings.ToLower(note.Title), q) ||
			strings.Contains(strings.ToLower(note.Content), q) {
			result = append(result, note)
		}
	}
	return result
}

// Len returns the number of notes on the board.
func (nb *NoteBoard) Len() int {
	return len(nb.notes)
}
