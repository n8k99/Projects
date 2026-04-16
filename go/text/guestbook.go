package text

import (
	"fmt"
	"strings"
	"time"
)

// Entry represents an immutable journal entry.
type Entry struct {
	Author    string
	Content   string
	Timestamp time.Time
}

// String formats an entry as "[timestamp] author: content".
func (e Entry) String() string {
	return fmt.Sprintf("[%s] %s: %s",
		e.Timestamp.UTC().Format("2006-01-02 15:04:05 UTC"),
		e.Author,
		e.Content,
	)
}

// Journal is a chronological append-only guestbook.
type Journal struct {
	entries []Entry
}

// NewJournal creates an empty journal.
func NewJournal() *Journal {
	return &Journal{entries: make([]Entry, 0)}
}

// AddEntry appends a new entry with the current timestamp.
func (j *Journal) AddEntry(author, content string) Entry {
	entry := Entry{
		Author:    author,
		Content:   content,
		Timestamp: time.Now().UTC(),
	}
	j.entries = append(j.entries, entry)
	return entry
}

// AddEntryAt appends a new entry with an explicit timestamp (for testing).
func (j *Journal) AddEntryAt(author, content string, ts time.Time) Entry {
	entry := Entry{
		Author:    author,
		Content:   content,
		Timestamp: ts,
	}
	j.entries = append(j.entries, entry)
	return entry
}

// GetEntries returns all entries, newest first.
func (j *Journal) GetEntries() []Entry {
	result := make([]Entry, len(j.entries))
	for i, e := range j.entries {
		result[len(j.entries)-1-i] = e
	}
	return result
}

// GetEntriesByAuthor returns entries by a specific author, newest first.
func (j *Journal) GetEntriesByAuthor(author string) []Entry {
	var result []Entry
	for i := len(j.entries) - 1; i >= 0; i-- {
		if j.entries[i].Author == author {
			result = append(result, j.entries[i])
		}
	}
	return result
}

// GetEntriesInRange returns entries within [start, end], newest first.
func (j *Journal) GetEntriesInRange(start, end time.Time) []Entry {
	var result []Entry
	for i := len(j.entries) - 1; i >= 0; i-- {
		ts := j.entries[i].Timestamp
		if (ts.Equal(start) || ts.After(start)) && (ts.Equal(end) || ts.Before(end)) {
			result = append(result, j.entries[i])
		}
	}
	return result
}

// EntryCount returns the total number of entries.
func (j *Journal) EntryCount() int {
	return len(j.entries)
}

// ExportText exports the full journal as plain text in chronological order.
func (j *Journal) ExportText() string {
	lines := make([]string, len(j.entries))
	for i, e := range j.entries {
		lines[i] = e.String()
	}
	return strings.Join(lines, "\n")
}
