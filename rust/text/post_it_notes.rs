//! Post it Notes Program — create, manage, and search notes.

use std::collections::HashMap;
use std::time::SystemTime;

/// A single note with id, title, content, timestamp, and optional color.
#[derive(Debug, Clone)]
pub struct Note {
    pub id: u64,
    pub title: String,
    pub content: String,
    pub created_at: SystemTime,
    pub color: Option<String>,
}

/// A collection of notes with CRUD and search operations.
#[derive(Debug)]
pub struct NoteBoard {
    notes: HashMap<u64, Note>,
    next_id: u64,
}

impl NoteBoard {
    /// Create a new empty note board.
    pub fn new() -> Self {
        NoteBoard {
            notes: HashMap::new(),
            next_id: 1,
        }
    }

    /// Add a note with the given title and content. Returns the assigned id.
    pub fn add_note(&mut self, title: &str, content: &str) -> u64 {
        self.add_note_with_color(title, content, None)
    }

    /// Add a note with an optional color. Returns the assigned id.
    pub fn add_note_with_color(&mut self, title: &str, content: &str, color: Option<&str>) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        self.notes.insert(id, Note {
            id,
            title: title.to_string(),
            content: content.to_string(),
            created_at: SystemTime::now(),
            color: color.map(|c| c.to_string()),
        });
        id
    }

    /// Get a note by id. Returns None if not found.
    pub fn get_note(&self, id: u64) -> Option<&Note> {
        self.notes.get(&id)
    }

    /// Update the content of a note. Returns true if the note existed.
    pub fn update_note(&mut self, id: u64, content: &str) -> bool {
        if let Some(note) = self.notes.get_mut(&id) {
            note.content = content.to_string();
            true
        } else {
            false
        }
    }

    /// Delete a note by id. Returns true if the note existed.
    pub fn delete_note(&mut self, id: u64) -> bool {
        self.notes.remove(&id).is_some()
    }

    /// List all notes sorted by id.
    pub fn list_notes(&self) -> Vec<&Note> {
        let mut notes: Vec<&Note> = self.notes.values().collect();
        notes.sort_by_key(|n| n.id);
        notes
    }

    /// Search notes by title or content (case-insensitive).
    pub fn search_notes(&self, query: &str) -> Vec<&Note> {
        let q = query.to_lowercase();
        self.notes.values()
            .filter(|n| {
                n.title.to_lowercase().contains(&q)
                    || n.content.to_lowercase().contains(&q)
            })
            .collect()
    }

    /// Return the number of notes on the board.
    pub fn len(&self) -> usize {
        self.notes.len()
    }

    /// Return true if the board has no notes.
    pub fn is_empty(&self) -> bool {
        self.notes.is_empty()
    }
}

impl Default for NoteBoard {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_and_get() {
        let mut board = NoteBoard::new();
        let id = board.add_note("Test", "Hello world");
        let note = board.get_note(id).unwrap();
        assert_eq!(note.title, "Test");
        assert_eq!(note.content, "Hello world");
        assert_eq!(note.id, 1);
    }

    #[test]
    fn test_auto_increment_ids() {
        let mut board = NoteBoard::new();
        let id1 = board.add_note("A", "first");
        let id2 = board.add_note("B", "second");
        let id3 = board.add_note("C", "third");
        assert_eq!(id1, 1);
        assert_eq!(id2, 2);
        assert_eq!(id3, 3);
    }

    #[test]
    fn test_update() {
        let mut board = NoteBoard::new();
        let id = board.add_note("Note", "old content");
        assert!(board.update_note(id, "new content"));
        assert_eq!(board.get_note(id).unwrap().content, "new content");
    }

    #[test]
    fn test_update_nonexistent() {
        let mut board = NoteBoard::new();
        assert!(!board.update_note(999, "nope"));
    }

    #[test]
    fn test_delete() {
        let mut board = NoteBoard::new();
        let id = board.add_note("Bye", "going away");
        assert!(board.delete_note(id));
        assert!(board.get_note(id).is_none());
        assert_eq!(board.len(), 0);
    }

    #[test]
    fn test_delete_nonexistent() {
        let mut board = NoteBoard::new();
        assert!(!board.delete_note(42));
    }

    #[test]
    fn test_list_sorted() {
        let mut board = NoteBoard::new();
        board.add_note("C", "third");
        board.add_note("A", "first");
        board.add_note("B", "second");
        let notes = board.list_notes();
        assert_eq!(notes.len(), 3);
        assert_eq!(notes[0].title, "C");
        assert_eq!(notes[1].title, "A");
        assert_eq!(notes[2].title, "B");
    }

    #[test]
    fn test_search() {
        let mut board = NoteBoard::new();
        board.add_note("Groceries", "milk and eggs");
        board.add_note("Meeting", "discuss project timeline");
        board.add_note("Idea", "build a project tracker");

        let results = board.search_notes("project");
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_search_case_insensitive() {
        let mut board = NoteBoard::new();
        board.add_note("HELLO", "World");
        let results = board.search_notes("hello");
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_color() {
        let mut board = NoteBoard::new();
        let id = board.add_note_with_color("Sticky", "note", Some("yellow"));
        let note = board.get_note(id).unwrap();
        assert_eq!(note.color.as_deref(), Some("yellow"));
    }

    #[test]
    fn test_is_empty() {
        let board = NoteBoard::new();
        assert!(board.is_empty());
    }
}
