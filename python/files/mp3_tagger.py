"""Mp3 Tagger — file metadata as a queryable, indexed tag store.

Tags (title, artist, album, year, genre, track) as Optional fields.
Indexed on artist/album/year for O(1) equality. Title is scan-only.
Missing vs empty distinction preserved throughout.
"""

from __future__ import annotations
from dataclasses import dataclass, field
from enum import Enum


@dataclass
class Tag:
    title: str | None = None
    artist: str | None = None
    album: str | None = None
    year: int | None = None
    genre: str | None = None
    track: int | None = None

    def is_empty(self) -> bool:
        return all(v is None for v in (
            self.title, self.artist, self.album, self.year, self.genre, self.track
        ))

    def missing_fields(self) -> list[str]:
        out = []
        if self.title is None: out.append("title")
        if self.artist is None: out.append("artist")
        if self.album is None: out.append("album")
        if self.year is None: out.append("year")
        if self.genre is None: out.append("genre")
        if self.track is None: out.append("track")
        return out


class QueryKind(Enum):
    BY_ARTIST = "by_artist"
    BY_ALBUM = "by_album"
    BY_YEAR = "by_year"
    YEAR_RANGE = "year_range"
    TITLE_CONTAINS = "title_contains"
    MISSING_FIELD = "missing_field"


@dataclass
class Query:
    kind: QueryKind
    artist: str = ""
    album: str = ""
    year: int = 0
    year_from: int = 0
    year_to: int = 0
    needle: str = ""
    field_name: str = ""


class TagStore:
    def __init__(self) -> None:
        self.tags: dict[str, Tag] = {}
        self._artist_idx: dict[str, list[str]] = {}
        self._album_idx: dict[str, list[str]] = {}
        self._year_idx: dict[int, list[str]] = {}

    def __len__(self) -> int:
        return len(self.tags)

    def get(self, path: str) -> Tag | None:
        return self.tags.get(path)

    def insert(self, path: str, tag: Tag) -> None:
        if path in self.tags:
            self._remove_from_indexes(path)
        if tag.artist is not None:
            self._artist_idx.setdefault(tag.artist, []).append(path)
        if tag.album is not None:
            self._album_idx.setdefault(tag.album, []).append(path)
        if tag.year is not None:
            self._year_idx.setdefault(tag.year, []).append(path)
        self.tags[path] = tag

    def remove(self, path: str) -> Tag | None:
        self._remove_from_indexes(path)
        return self.tags.pop(path, None)

    def _remove_from_indexes(self, path: str) -> None:
        tag = self.tags.get(path)
        if tag is None:
            return
        if tag.artist is not None and tag.artist in self._artist_idx:
            self._artist_idx[tag.artist] = [p for p in self._artist_idx[tag.artist] if p != path]
        if tag.album is not None and tag.album in self._album_idx:
            self._album_idx[tag.album] = [p for p in self._album_idx[tag.album] if p != path]
        if tag.year is not None and tag.year in self._year_idx:
            self._year_idx[tag.year] = [p for p in self._year_idx[tag.year] if p != path]

    def query(self, q: Query) -> list[str]:
        if q.kind == QueryKind.BY_ARTIST:
            paths = list(self._artist_idx.get(q.artist, []))
        elif q.kind == QueryKind.BY_ALBUM:
            paths = list(self._album_idx.get(q.album, []))
        elif q.kind == QueryKind.BY_YEAR:
            paths = list(self._year_idx.get(q.year, []))
        elif q.kind == QueryKind.YEAR_RANGE:
            paths = []
            for y, ps in self._year_idx.items():
                if q.year_from <= y <= q.year_to:
                    paths.extend(ps)
        elif q.kind == QueryKind.TITLE_CONTAINS:
            needle = q.needle.lower()
            paths = [
                p for p, t in self.tags.items()
                if t.title and needle in t.title.lower()
            ]
        elif q.kind == QueryKind.MISSING_FIELD:
            paths = [
                p for p, t in self.tags.items()
                if _field_missing(t, q.field_name)
            ]
        else:
            paths = []
        return sorted(paths)


def _field_missing(t: Tag, name: str) -> bool:
    return getattr(t, name, "present") is None


def demo() -> None:
    s = TagStore()
    s.insert("a.mp3", Tag(title="Aeon", artist="Björk", album="Vespertine",
                          year=2001, genre="Electronic", track=1))
    s.insert("b.mp3", Tag(title="Pagan Poetry", artist="Björk", album="Vespertine",
                          year=2001, genre="Electronic", track=3))
    s.insert("c.mp3", Tag(title="Unplayed"))
    s.insert("d.mp3", Tag(title="Hyperballad", artist="Björk", album="Post",
                          year=1995, track=2))

    print("=== by artist: Björk ===")
    for p in s.query(Query(QueryKind.BY_ARTIST, artist="Björk")):
        print(f"  {p}")
    print("\n=== year range 2000-2010 ===")
    for p in s.query(Query(QueryKind.YEAR_RANGE, year_from=2000, year_to=2010)):
        print(f"  {p}")
    print("\n=== title contains 'aeon' ===")
    for p in s.query(Query(QueryKind.TITLE_CONTAINS, needle="aeon")):
        print(f"  {p}")
    print("\n=== missing genre ===")
    for p in s.query(Query(QueryKind.MISSING_FIELD, field_name="genre")):
        print(f"  {p}")


if __name__ == "__main__":
    demo()
