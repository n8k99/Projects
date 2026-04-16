"""Quote Tracker — collect, categorize, and retrieve quotes."""

from dataclasses import dataclass, field
import random


@dataclass
class Quote:
    """A single quote with attribution and categorization."""
    text: str
    author: str
    source: str = ""
    tags: list[str] = field(default_factory=list)

    def __str__(self) -> str:
        parts = [f'"{self.text}" — {self.author}']
        if self.source:
            parts.append(f"  ({self.source})")
        if self.tags:
            parts.append(f"  [{', '.join(self.tags)}]")
        return "".join(parts)


class QuoteTracker:
    """A curated collection of quotes with attribution, tags, and search."""

    def __init__(self):
        self._quotes: list[Quote] = []

    def add_quote(self, text: str, author: str, source: str = "", tags: list[str] | None = None) -> Quote:
        """Add a quote to the collection."""
        quote = Quote(text=text, author=author, source=source, tags=tags or [])
        self._quotes.append(quote)
        return quote

    def random_quote(self) -> Quote | None:
        """Return a random quote, or None if the collection is empty."""
        if not self._quotes:
            return None
        return random.choice(self._quotes)

    def search_by_author(self, author: str) -> list[Quote]:
        """Return all quotes by the given author (case-insensitive substring match)."""
        author_lower = author.lower()
        return [q for q in self._quotes if author_lower in q.author.lower()]

    def search_by_tag(self, tag: str) -> list[Quote]:
        """Return all quotes with the given tag (case-insensitive)."""
        tag_lower = tag.lower()
        return [q for q in self._quotes if any(t.lower() == tag_lower for t in q.tags)]

    def list_authors(self) -> list[str]:
        """Return sorted list of unique authors."""
        return sorted(set(q.author for q in self._quotes))

    def list_tags(self) -> list[str]:
        """Return sorted list of unique tags."""
        all_tags: set[str] = set()
        for q in self._quotes:
            all_tags.update(q.tags)
        return sorted(all_tags)

    def count(self) -> int:
        """Return total number of quotes."""
        return len(self._quotes)


if __name__ == "__main__":
    tracker = QuoteTracker()

    tracker.add_quote(
        "The only way to do great work is to love what you do.",
        "Steve Jobs", "Stanford Commencement, 2005", ["work", "passion"]
    )
    tracker.add_quote(
        "Talk is cheap. Show me the code.",
        "Linus Torvalds", "", ["programming", "work"]
    )
    tracker.add_quote(
        "The best way to predict the future is to invent it.",
        "Alan Kay", "1971", ["innovation", "programming"]
    )
    tracker.add_quote(
        "Simplicity is the ultimate sophistication.",
        "Leonardo da Vinci", "", ["design", "philosophy"]
    )
    tracker.add_quote(
        "Programs must be written for people to read, and only incidentally for machines to execute.",
        "Hal Abelson", "SICP", ["programming", "design"]
    )
    tracker.add_quote(
        "In the middle of difficulty lies opportunity.",
        "Albert Einstein", "", ["philosophy", "resilience"]
    )

    print(f"Collection: {tracker.count()} quotes")
    print(f"Authors: {', '.join(tracker.list_authors())}")
    print(f"Tags: {', '.join(tracker.list_tags())}")

    print("\n--- Random quote ---")
    print(tracker.random_quote())

    print("\n--- Search by author: 'alan' ---")
    for q in tracker.search_by_author("alan"):
        print(q)

    print("\n--- Search by tag: 'programming' ---")
    for q in tracker.search_by_tag("programming"):
        print(q)

    print("\n--- Search by tag: 'philosophy' ---")
    for q in tracker.search_by_tag("philosophy"):
        print(q)
