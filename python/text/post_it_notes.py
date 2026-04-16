"""Post it Notes Program — create, manage, and search notes."""

from dataclasses import dataclass, field
from datetime import datetime
from typing import Optional


@dataclass
class Note:
    """A single note with id, title, content, timestamp, and optional color."""
    id: int
    title: str
    content: str
    created_at: datetime
    color: Optional[str] = None

    def __repr__(self) -> str:
        color_str = f", color={self.color!r}" if self.color else ""
        return f"Note(id={self.id}, title={self.title!r}{color_str})"


class NoteBoard:
    """A collection of notes with CRUD and search operations."""

    def __init__(self):
        self._notes: dict[int, Note] = {}
        self._next_id: int = 1

    def add_note(self, title: str, content: str, color: Optional[str] = None) -> int:
        """Create a new note and return its id."""
        note_id = self._next_id
        self._next_id += 1
        self._notes[note_id] = Note(
            id=note_id,
            title=title,
            content=content,
            created_at=datetime.now(),
            color=color,
        )
        return note_id

    def get_note(self, note_id: int) -> Note:
        """Retrieve a note by id. Raises KeyError if not found."""
        if note_id not in self._notes:
            raise KeyError(f"Note {note_id} not found")
        return self._notes[note_id]

    def update_note(self, note_id: int, content: str) -> None:
        """Update the content of an existing note."""
        if note_id not in self._notes:
            raise KeyError(f"Note {note_id} not found")
        self._notes[note_id].content = content

    def delete_note(self, note_id: int) -> None:
        """Delete a note by id. Raises KeyError if not found."""
        if note_id not in self._notes:
            raise KeyError(f"Note {note_id} not found")
        del self._notes[note_id]

    def list_notes(self) -> list[Note]:
        """Return all notes sorted by id."""
        return sorted(self._notes.values(), key=lambda n: n.id)

    def search_notes(self, query: str) -> list[Note]:
        """Search notes by title or content (case-insensitive)."""
        q = query.lower()
        return [
            n for n in self._notes.values()
            if q in n.title.lower() or q in n.content.lower()
        ]

    def __len__(self) -> int:
        return len(self._notes)


if __name__ == "__main__":
    board = NoteBoard()

    # Create 3 notes
    id1 = board.add_note("Groceries", "Milk, eggs, bread", color="yellow")
    id2 = board.add_note("Meeting Notes", "Discuss project timeline with team")
    id3 = board.add_note("Idea", "Build a note-taking app with search", color="blue")

    print("=== All notes ===")
    for note in board.list_notes():
        print(f"  [{note.id}] {note.title}: {note.content}")

    # Search
    print("\n=== Search 'project' ===")
    for note in board.search_notes("project"):
        print(f"  [{note.id}] {note.title}: {note.content}")

    # Update
    board.update_note(id2, "Discuss project timeline — moved to Friday")
    print(f"\n=== Updated note {id2} ===")
    print(f"  {board.get_note(id2).content}")

    # Delete
    board.delete_note(id1)
    print(f"\n=== After deleting note {id1} ===")
    for note in board.list_notes():
        print(f"  [{note.id}] {note.title}: {note.content}")
