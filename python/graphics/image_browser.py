"""Image Browser — 2D grid navigation + LRU cache + filter/sort query.

Grid cursor with directional moves + partial-row clamping.
LRU cache: access refreshes recency; LRU evicted on insertion.
Query: list of filter predicates (AND) + SortBy + Order as data.
Breadcrumbs: push/pop directory stack; path = '/'.join(stack).
"""

from __future__ import annotations
from collections import OrderedDict
from dataclasses import dataclass, field
from enum import Enum
from typing import Callable, Optional


@dataclass
class ImageEntry:
    id: int
    name: str
    path: str
    width: int
    height: int
    size_bytes: int
    tags: list[str]
    created_ms: int


class FilterKind(Enum):
    NAME_CONTAINS = "name_contains"
    TAG_HAS = "tag_has"
    SIZE_UNDER = "size_under"
    SIZE_OVER = "size_over"
    WIDTH_AT_LEAST = "width_at_least"
    CREATED_AFTER = "created_after"


@dataclass(frozen=True)
class FilterPredicate:
    kind: FilterKind
    s: str = ""
    n: int = 0

    def matches(self, e: ImageEntry) -> bool:
        k = self.kind
        if k == FilterKind.NAME_CONTAINS: return self.s in e.name
        if k == FilterKind.TAG_HAS: return self.s in e.tags
        if k == FilterKind.SIZE_UNDER: return e.size_bytes < self.n
        if k == FilterKind.SIZE_OVER: return e.size_bytes > self.n
        if k == FilterKind.WIDTH_AT_LEAST: return e.width >= self.n
        if k == FilterKind.CREATED_AFTER: return e.created_ms > self.n
        raise AssertionError("unreachable")


def filter_name_contains(s: str) -> FilterPredicate:
    return FilterPredicate(FilterKind.NAME_CONTAINS, s=s)
def filter_tag_has(s: str) -> FilterPredicate:
    return FilterPredicate(FilterKind.TAG_HAS, s=s)
def filter_size_over(n: int) -> FilterPredicate:
    return FilterPredicate(FilterKind.SIZE_OVER, n=n)
def filter_size_under(n: int) -> FilterPredicate:
    return FilterPredicate(FilterKind.SIZE_UNDER, n=n)


class SortBy(Enum):
    NAME = "name"
    SIZE = "size"
    CREATED = "created"
    WIDTH = "width"


class Order(Enum):
    ASC = "asc"
    DESC = "desc"


@dataclass
class Query:
    filters: list[FilterPredicate] = field(default_factory=list)
    sort_by: SortBy = SortBy.NAME
    order: Order = Order.ASC


def apply_query(entries: list[ImageEntry], query: Query) -> list[ImageEntry]:
    filtered = [e for e in entries if all(p.matches(e) for p in query.filters)]
    key = {
        SortBy.NAME: lambda e: e.name,
        SortBy.SIZE: lambda e: e.size_bytes,
        SortBy.CREATED: lambda e: e.created_ms,
        SortBy.WIDTH: lambda e: e.width,
    }[query.sort_by]
    filtered.sort(key=key, reverse=(query.order == Order.DESC))
    return filtered


class GridMove(Enum):
    LEFT = "left"
    RIGHT = "right"
    UP = "up"
    DOWN = "down"
    HOME = "home"
    END = "end"


@dataclass
class Grid:
    cols: int
    total: int
    selected: int = 0

    def rows(self) -> int:
        if self.cols == 0 or self.total == 0: return 0
        return (self.total + self.cols - 1) // self.cols

    def selected_row_col(self) -> tuple[int, int]:
        return (self.selected // self.cols, self.selected % self.cols)

    def move(self, mv: GridMove) -> None:
        if self.total == 0: return
        row, col = self.selected_row_col()
        if mv == GridMove.LEFT:
            if self.selected > 0:
                self.selected -= 1
        elif mv == GridMove.RIGHT:
            if self.selected + 1 < self.total:
                self.selected += 1
        elif mv == GridMove.UP:
            if row > 0:
                self.selected = (row - 1) * self.cols + col
        elif mv == GridMove.DOWN:
            nxt = (row + 1) * self.cols + col
            if nxt < self.total:
                self.selected = nxt
            elif row + 1 < self.rows():
                self.selected = self.total - 1
        elif mv == GridMove.HOME:
            self.selected = 0
        elif mv == GridMove.END:
            self.selected = self.total - 1


class LruCache:
    def __init__(self, capacity: int) -> None:
        self.capacity = capacity
        self._data: OrderedDict[int, bytes] = OrderedDict()

    def get(self, k: int) -> Optional[bytes]:
        if k not in self._data: return None
        self._data.move_to_end(k)
        return self._data[k]

    def put(self, k: int, v: bytes) -> Optional[int]:
        """Returns evicted key if any."""
        if k in self._data:
            self._data.move_to_end(k)
            self._data[k] = v
            return None
        if len(self._data) >= self.capacity:
            evicted, _ = self._data.popitem(last=False)
            self._data[k] = v
            return evicted
        self._data[k] = v
        return None

    def __len__(self) -> int: return len(self._data)
    def contains(self, k: int) -> bool: return k in self._data


@dataclass
class Browser:
    grid_cols: int
    thumbnail_cache_capacity: int
    breadcrumbs: list[str] = field(default_factory=list)
    all_entries: list[ImageEntry] = field(default_factory=list)
    query: Query = field(default_factory=Query)
    grid: Grid = field(init=False)
    thumbnail_cache: LruCache = field(init=False)

    def __post_init__(self) -> None:
        self.grid = Grid(self.grid_cols, 0)
        self.thumbnail_cache = LruCache(self.thumbnail_cache_capacity)

    def set_entries(self, entries: list[ImageEntry]) -> None:
        self.all_entries = entries
        filtered = apply_query(self.all_entries, self.query)
        self.grid = Grid(self.grid_cols, len(filtered))

    def set_query(self, query: Query) -> None:
        self.query = query
        filtered = apply_query(self.all_entries, self.query)
        self.grid = Grid(self.grid_cols, len(filtered))

    def filtered_entries(self) -> list[ImageEntry]:
        return apply_query(self.all_entries, self.query)

    def selected_entry(self) -> Optional[ImageEntry]:
        filtered = self.filtered_entries()
        if 0 <= self.grid.selected < len(filtered):
            return filtered[self.grid.selected]
        return None

    def navigate_into(self, subdir: str) -> None:
        self.breadcrumbs.append(subdir)

    def navigate_up(self) -> bool:
        if self.breadcrumbs:
            self.breadcrumbs.pop()
            return True
        return False

    def current_path(self) -> str:
        return "/" + "/".join(self.breadcrumbs)

    def move_cursor(self, mv: GridMove) -> None:
        self.grid.move(mv)

    def get_or_load_thumbnail(self, id: int,
                               producer: Callable[[], bytes]) -> tuple[bytes, bool]:
        cached = self.thumbnail_cache.get(id)
        if cached is not None:
            return (cached, False)
        v = producer()
        self.thumbnail_cache.put(id, v)
        return (v, True)


def demo() -> None:
    br = Browser(grid_cols=3, thumbnail_cache_capacity=3)
    entries = [
        ImageEntry(1, "alpha.jpg", "/img/alpha.jpg", 1920, 1080, 200_000, ["nature"], 1000),
        ImageEntry(2, "beta.jpg", "/img/beta.jpg", 800, 600, 50_000, ["abstract"], 2000),
        ImageEntry(3, "gamma.png", "/img/gamma.png", 1024, 768, 150_000, ["nature"], 3000),
        ImageEntry(4, "delta.jpg", "/img/delta.jpg", 2560, 1440, 500_000, ["nature"], 4000),
        ImageEntry(5, "epsilon.png", "/img/epsilon.png", 1920, 1080, 100_000, ["abstract"], 5000),
    ]
    br.set_entries(entries)

    print(f"all: {len(br.filtered_entries())}; grid total: {br.grid.total}")

    br.set_query(Query(
        filters=[filter_name_contains(".jpg"), filter_tag_has("nature")],
        sort_by=SortBy.SIZE, order=Order.ASC,
    ))
    names = [e.name for e in br.filtered_entries()]
    print(f"nature jpg by size asc: {names}")
    print(f"grid total: {br.grid.total}")

    # Move cursor across the filtered 2-item grid
    print(f"selected: {br.selected_entry().name}")
    br.move_cursor(GridMove.RIGHT)
    print(f"after right: {br.selected_entry().name}")
    br.move_cursor(GridMove.RIGHT)  # at end, no-op
    print(f"after right at end: {br.selected_entry().name}")

    # Breadcrumbs
    br.navigate_into("photos")
    br.navigate_into("2026")
    print(f"path: {br.current_path()}")
    br.navigate_up()
    print(f"after up: {br.current_path()}")

    # LRU cache
    calls = [0]
    def gen() -> bytes:
        calls[0] += 1
        return b"bytes"
    (_, miss1) = br.get_or_load_thumbnail(1, gen)
    (_, miss2) = br.get_or_load_thumbnail(1, gen)
    (_, miss3) = br.get_or_load_thumbnail(2, gen)
    print(f"cache miss: {miss1} hit: {not miss2} miss2: {miss3}")
    print(f"producer calls: {calls[0]}")

    # LRU eviction
    lru: LruCache = LruCache(capacity=2)
    lru.put(1, b"a")
    lru.put(2, b"b")
    lru.get(1)  # refresh 1
    evicted = lru.put(3, b"c")
    print(f"LRU evicted key: {evicted}")


if __name__ == "__main__":
    demo()
