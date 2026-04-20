"""File Copy Utility — recursive tree copy with policy + filter + progress events.

Three overwrite policies (Skip / Overwrite / RenameWithSuffix).
Filter accepts include patterns and exclude patterns (substring match).
Progress is a list of CopyEvents instead of callbacks — testable.
"""

from __future__ import annotations
from dataclasses import dataclass, field
from enum import Enum


class ItemKind(Enum):
    FILE = "file"
    DIR = "dir"


@dataclass
class Item:
    kind: ItemKind
    size: int = 0


class Overwrite(Enum):
    SKIP = "skip"
    OVERWRITE = "overwrite"
    RENAME_WITH_SUFFIX = "rename_with_suffix"


@dataclass
class Filter:
    include: list[str] = field(default_factory=list)
    exclude: list[str] = field(default_factory=list)

    @classmethod
    def allow_all(cls) -> Filter:
        return cls()

    def accepts(self, path: str) -> bool:
        if self.include and not any(pat in path for pat in self.include):
            return False
        if any(pat in path for pat in self.exclude):
            return False
        return True


class EventKind(Enum):
    COPIED = "copied"
    SKIPPED = "skipped"
    RENAMED = "renamed"
    CREATED_DIR = "created_dir"


@dataclass
class CopyEvent:
    kind: EventKind
    source: str
    dest: str
    bytes: int = 0


@dataclass
class Fs:
    items: dict[str, Item] = field(default_factory=dict)

    @classmethod
    def from_items(cls, entries: list[tuple[str, Item]]) -> Fs:
        return cls(items={p: i for p, i in entries})

    def paths(self) -> list[str]:
        return sorted(self.items.keys())

    def copy_tree(self, src_root: str, dest_root: str,
                  policy: Overwrite, filter: Filter) -> list[CopyEvent]:
        src_paths = sorted(
            p for p in self.items
            if p == src_root or p.startswith(src_root + "/")
        )
        if not src_paths:
            raise ValueError(f"source not found: {src_root}")

        events = []
        for src_path in src_paths:
            item = self.items[src_path]
            if not filter.accepts(src_path):
                continue
            rel = "" if src_path == src_root else src_path[len(src_root) + 1:]
            dest_path = dest_root if rel == "" else f"{dest_root}/{rel}"

            final_dest = dest_path
            if dest_path in self.items:
                if policy == Overwrite.SKIP:
                    events.append(CopyEvent(EventKind.SKIPPED, src_path, dest_path))
                    continue
                elif policy == Overwrite.RENAME_WITH_SUFFIX:
                    renamed = _next_available(self.items, dest_path)
                    events.append(CopyEvent(EventKind.RENAMED, src_path, renamed))
                    final_dest = renamed

            if item.kind == ItemKind.DIR:
                self.items[final_dest] = Item(ItemKind.DIR)
                events.append(CopyEvent(EventKind.CREATED_DIR, src_path, final_dest))
            else:
                self.items[final_dest] = Item(ItemKind.FILE, size=item.size)
                events.append(CopyEvent(EventKind.COPIED, src_path, final_dest, bytes=item.size))
        return events

    def total_size_copied(self, events: list[CopyEvent]) -> int:
        return sum(e.bytes for e in events if e.kind == EventKind.COPIED)


def _next_available(items: dict[str, Item], base: str) -> str:
    n = 1
    while True:
        candidate = f"{base}.{n}"
        if candidate not in items:
            return candidate
        n += 1


def demo() -> None:
    fs = Fs.from_items([
        ("src", Item(ItemKind.DIR)),
        ("src/a.txt", Item(ItemKind.FILE, size=100)),
        ("src/b.txt", Item(ItemKind.FILE, size=200)),
        ("src/sub", Item(ItemKind.DIR)),
        ("src/sub/c.txt", Item(ItemKind.FILE, size=50)),
    ])
    events = fs.copy_tree("src", "dest", Overwrite.OVERWRITE, Filter.allow_all())
    print(f"=== copy: src → dest ===")
    for e in events:
        print(f"  {e.kind.value:12} {e.source:20} -> {e.dest:20} ({e.bytes}B)")
    print(f"\ntotal bytes copied: {fs.total_size_copied(events)}")
    print(f"all paths: {fs.paths()}")


if __name__ == "__main__":
    demo()
