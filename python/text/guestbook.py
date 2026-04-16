"""Guestbook / Journal — chronological append-only entry log."""

from datetime import datetime, timezone
from dataclasses import dataclass, field
from typing import List, Optional


@dataclass(frozen=True)
class Entry:
    """An immutable journal entry with author, content, and timestamp."""
    author: str
    content: str
    timestamp: datetime = field(default_factory=lambda: datetime.now(timezone.utc))

    def to_text(self) -> str:
        ts = self.timestamp.strftime("%Y-%m-%d %H:%M:%S UTC")
        return f"[{ts}] {self.author}: {self.content}"


class Journal:
    """Chronological append-only journal / guestbook."""

    def __init__(self) -> None:
        self._entries: List[Entry] = []

    def add_entry(self, author: str, content: str) -> Entry:
        """Append a new entry. Returns the created entry."""
        entry = Entry(author=author, content=content)
        self._entries.append(entry)
        return entry

    def get_entries(self) -> List[Entry]:
        """Return all entries, newest first."""
        return list(reversed(self._entries))

    def get_entries_by_author(self, author: str) -> List[Entry]:
        """Return entries by a specific author, newest first."""
        return [e for e in reversed(self._entries) if e.author == author]

    def get_entries_in_range(
        self, start: datetime, end: datetime
    ) -> List[Entry]:
        """Return entries within [start, end], newest first."""
        return [
            e for e in reversed(self._entries)
            if start <= e.timestamp <= end
        ]

    def entry_count(self) -> int:
        """Return total number of entries."""
        return len(self._entries)

    def export_text(self) -> str:
        """Export the full journal as plain text (chronological order)."""
        lines = [e.to_text() for e in self._entries]
        return "\n".join(lines)


if __name__ == "__main__":
    import time

    journal = Journal()

    # Add entries from different authors
    journal.add_entry("Nathan", "Started the Rosetta Stone project today.")
    time.sleep(0.01)
    journal.add_entry("Ghost-A", "Reviewed the palindrome implementations.")
    time.sleep(0.01)
    journal.add_entry("Nathan", "Finished the quote tracker — G024 complete.")
    time.sleep(0.01)
    journal.add_entry("Ghost-B", "Ran benchmarks on the RSS feed parser.")
    time.sleep(0.01)
    journal.add_entry("Nathan", "Starting G025 — the guestbook/journal.")

    print(f"Total entries: {journal.entry_count()}")
    print()

    # All entries newest-first
    print("=== All Entries (newest first) ===")
    for entry in journal.get_entries():
        print(entry.to_text())
    print()

    # Query by author
    print("=== Nathan's Entries ===")
    for entry in journal.get_entries_by_author("Nathan"):
        print(entry.to_text())
    print()

    # Query by date range (all of today)
    now = datetime.now(timezone.utc)
    start_of_day = now.replace(hour=0, minute=0, second=0, microsecond=0)
    print("=== Today's Entries ===")
    for entry in journal.get_entries_in_range(start_of_day, now):
        print(entry.to_text())
    print()

    # Export
    print("=== Full Export ===")
    print(journal.export_text())
