"""File Explorer — navigable virtual filesystem with path resolution.

Third Files project. Core move: **path resolution** turns user input
(absolute, relative, with `.` and `..`) into a canonical segment list.
Root escape is impossible. Virtual filesystem — pure, no disk I/O.
"""

from __future__ import annotations
from dataclasses import dataclass, field
from enum import Enum


class Kind(Enum):
    FILE = "file"
    DIR = "dir"


@dataclass
class Node:
    kind: Kind
    size: int = 0
    mtime_ms: int = 0
    children: dict[str, Node] = field(default_factory=dict)

    @staticmethod
    def dir() -> Node:
        return Node(Kind.DIR, children={})

    @staticmethod
    def file(size: int, mtime_ms: int) -> Node:
        return Node(Kind.FILE, size=size, mtime_ms=mtime_ms)


@dataclass
class Listing:
    name: str
    kind: Kind
    size: int
    mtime_ms: int


class FsError(Exception):
    pass


class NotFound(FsError):
    pass


class NotADir(FsError):
    pass


class AlreadyExists(FsError):
    pass


class Explorer:
    def __init__(self) -> None:
        self.root = Node.dir()
        self._cwd: list[str] = []

    @property
    def cwd(self) -> str:
        return "/" + "/".join(self._cwd)

    def resolve(self, path: str) -> list[str] | None:
        if path == "":
            return None
        segs: list[str] = [] if path.startswith("/") else list(self._cwd)
        for seg in path.split("/"):
            if seg == "" or seg == ".":
                continue
            if seg == "..":
                if segs:
                    segs.pop()
                continue
            segs.append(seg)
        return segs

    def _node_at(self, segs: list[str]) -> Node | None:
        cur = self.root
        for s in segs:
            if cur.kind != Kind.DIR or s not in cur.children:
                return None
            cur = cur.children[s]
        return cur

    def cd(self, path: str) -> None:
        segs = self.resolve(path)
        if segs is None:
            raise NotFound(path)
        node = self._node_at(segs)
        if node is None:
            raise NotFound(path)
        if node.kind != Kind.DIR:
            raise NotADir(path)
        self._cwd = segs

    def ls(self, path: str = "") -> list[Listing]:
        segs = list(self._cwd) if path == "" else self.resolve(path)
        if segs is None:
            raise NotFound(path)
        node = self._node_at(segs)
        if node is None:
            raise NotFound(path)
        if node.kind != Kind.DIR:
            raise NotADir(path)
        listings = [
            Listing(name, child.kind,
                    child.size if child.kind == Kind.FILE else 0,
                    child.mtime_ms if child.kind == Kind.FILE else 0)
            for name, child in node.children.items()
        ]
        # Dirs first, then alphabetical.
        listings.sort(key=lambda l: (l.kind != Kind.DIR, l.name))
        return listings

    def mkdir(self, path: str) -> None:
        segs = self.resolve(path)
        if segs is None or not segs:
            raise AlreadyExists(path or "/")
        parent = self._node_at(segs[:-1])
        if parent is None or parent.kind != Kind.DIR:
            raise NotFound(path)
        if segs[-1] in parent.children:
            raise AlreadyExists(path)
        parent.children[segs[-1]] = Node.dir()

    def touch(self, path: str, size: int, mtime_ms: int) -> None:
        segs = self.resolve(path)
        if segs is None or not segs:
            raise AlreadyExists(path or "/")
        parent = self._node_at(segs[:-1])
        if parent is None or parent.kind != Kind.DIR:
            raise NotFound(path)
        parent.children[segs[-1]] = Node.file(size, mtime_ms)

    def stat(self, path: str) -> Listing:
        segs = self.resolve(path)
        if segs is None:
            raise NotFound(path)
        node = self._node_at(segs)
        if node is None:
            raise NotFound(path)
        name = segs[-1] if segs else "/"
        if node.kind == Kind.DIR:
            return Listing(name, Kind.DIR, 0, 0)
        return Listing(name, Kind.FILE, node.size, node.mtime_ms)

    def total_size(self, path: str) -> int:
        segs = self.resolve(path)
        if segs is None:
            raise NotFound(path)
        node = self._node_at(segs)
        if node is None:
            raise NotFound(path)
        return _size_of(node)


def _size_of(node: Node) -> int:
    if node.kind == Kind.FILE:
        return node.size
    return sum(_size_of(c) for c in node.children.values())


def demo() -> None:
    e = Explorer()
    e.mkdir("/home")
    e.mkdir("/home/nathan")
    e.mkdir("/home/nathan/docs")
    e.touch("/home/nathan/docs/a.txt", 100, 1000)
    e.touch("/home/nathan/docs/b.txt", 200, 2000)
    e.touch("/home/nathan/notes.md", 50, 500)
    e.mkdir("/tmp")

    print(f"=== cwd: {e.cwd} ===")
    e.cd("/home/nathan")
    print(f"cd /home/nathan → cwd: {e.cwd}")
    e.cd("docs")
    print(f"cd docs → cwd: {e.cwd}")
    e.cd("../..")
    print(f"cd ../.. → cwd: {e.cwd}")

    print("\n=== ls /home/nathan ===")
    for l in e.ls("/home/nathan"):
        kind = "d" if l.kind == Kind.DIR else "f"
        print(f"  {kind} {l.name:12} {l.size:5}  mtime={l.mtime_ms}")

    print(f"\ntotal size of /home = {e.total_size('/home')}")
    print(f"resolve /a/./b/../c = {e.resolve('/a/./b/../c')}")


if __name__ == "__main__":
    demo()
