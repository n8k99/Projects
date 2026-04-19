"""Bookmark Collector and Sorter — multi-axis organisation over URL-keyed collection.

Two orthogonal ways to organise the same bookmarks: **folders** (hierarchy,
one folder per bookmark) and **tags** (flat set, many-to-many). Real users
use both. URL is the natural key — adding a bookmark with an existing URL
updates the existing entry. Sort order is never stored; every sort is a
**view** computed on query.
"""

from __future__ import annotations
from dataclasses import dataclass, field
from enum import Enum


class SortBy(Enum):
    TITLE = "title"
    URL = "url"
    ADDED_AT = "added_at"
    FOLDER = "folder"


@dataclass
class Bookmark:
    id: int
    url: str
    title: str
    tags: set[str] = field(default_factory=set)
    folder: str = "/"
    added_at_ms: int = 0


class BookmarkCollection:
    def __init__(self) -> None:
        self._bookmarks: list[Bookmark] = []
        self._next_id = 1

    def add(self, url: str, title: str, folder: str = "/",
            tags: set[str] | None = None, added_at_ms: int = 0) -> int:
        tags = set(tags or [])
        for b in self._bookmarks:
            if b.url == url:
                b.title = title
                b.folder = folder
                b.tags = tags
                return b.id
        bid = self._next_id
        self._next_id += 1
        self._bookmarks.append(Bookmark(
            id=bid, url=url, title=title, folder=folder,
            tags=tags, added_at_ms=added_at_ms,
        ))
        return bid

    def remove(self, bid: int) -> bool:
        for i, b in enumerate(self._bookmarks):
            if b.id == bid:
                del self._bookmarks[i]
                return True
        return False

    def count(self) -> int:
        return len(self._bookmarks)

    def get(self, bid: int) -> Bookmark | None:
        return next((b for b in self._bookmarks if b.id == bid), None)

    def set_tags(self, bid: int, tags: set[str]) -> bool:
        b = self.get(bid)
        if b is None:
            return False
        b.tags = set(tags)
        return True

    def move_to_folder(self, bid: int, folder: str) -> bool:
        b = self.get(bid)
        if b is None:
            return False
        b.folder = folder
        return True

    def all(self) -> list[Bookmark]:
        return list(self._bookmarks)

    def search(self, query: str) -> list[Bookmark]:
        q = query.lower()
        return [b for b in self._bookmarks
                if q in b.title.lower() or q in b.url.lower()
                or any(q in t.lower() for t in b.tags)]

    def by_tag(self, tag: str) -> list[Bookmark]:
        return [b for b in self._bookmarks if tag in b.tags]

    def by_folder(self, folder: str, recursive: bool = False) -> list[Bookmark]:
        if recursive:
            prefix = folder if folder.endswith("/") else folder + "/"
            return [b for b in self._bookmarks
                    if b.folder == folder or b.folder.startswith(prefix)]
        return [b for b in self._bookmarks if b.folder == folder]

    def sort_by(self, criterion: SortBy) -> list[Bookmark]:
        key_fns = {
            SortBy.TITLE:    lambda b: b.title,
            SortBy.URL:      lambda b: b.url,
            SortBy.ADDED_AT: lambda b: b.added_at_ms,
            SortBy.FOLDER:   lambda b: b.folder,
        }
        # Python's sorted() is stable.
        return sorted(self._bookmarks, key=key_fns[criterion])

    def all_tags(self) -> list[str]:
        tags: set[str] = set()
        for b in self._bookmarks:
            tags |= b.tags
        return sorted(tags)

    def all_folders(self) -> list[str]:
        return sorted({b.folder for b in self._bookmarks})


if __name__ == "__main__":
    c = BookmarkCollection()
    c.add("https://rust-lang.org", "Rust Language", "/dev/languages",
          {"language", "systems", "memory-safe"}, added_at_ms=1000)
    c.add("https://python.org", "Python", "/dev/languages",
          {"language", "dynamic"}, added_at_ms=2000)
    c.add("https://kernel.org", "Linux", "/dev/systems",
          {"systems", "unix"}, added_at_ms=3000)
    c.add("https://news.ycombinator.com", "Hacker News", "/news",
          {"news", "tech"}, added_at_ms=1500)

    print(f"count: {c.count()}")
    print(f"all tags: {c.all_tags()}")
    print()
    print("sort by title:")
    for b in c.sort_by(SortBy.TITLE):
        print(f"  {b.title} ({b.folder})")
    print()
    print(f"by folder /dev (recursive): {len(c.by_folder('/dev', recursive=True))}")
    print(f"by folder /dev (exact):     {len(c.by_folder('/dev'))}")
    print(f"by tag 'language':          {len(c.by_tag('language'))}")
    print(f"search 'py':                {[b.title for b in c.search('py')]}")

    # Dedup on re-add with same URL.
    c.add("https://rust-lang.org", "The Rust Programming Language",
          "/dev", {"language", "rust"}, added_at_ms=4000)
    print(f"after re-add: count={c.count()} (still 4)")
    print(f"updated title: {c.get(1).title if c.get(1) else 'missing'}")
