"""Versioning Manager — content-addressed version store, minimal Git-like.

FNV-1a 64-bit hash for content addressing (cross-language deterministic).
Blob = bytes stored by hash; Tree = filename→blob hash map; Commit = tree
hash + parent + message + timestamp. Checkout/diff/log operate on the DAG.
"""

from __future__ import annotations
from dataclasses import dataclass, field
from enum import Enum


def fnv1a_hash(data: bytes) -> int:
    h = 0xcbf29ce484222325
    for b in data:
        h ^= b
        h = (h * 0x100000001b3) & 0xFFFFFFFFFFFFFFFF
    return h


@dataclass
class Blob:
    data: bytes


@dataclass
class Tree:
    entries: dict[str, int] = field(default_factory=dict)  # name -> blob hash


@dataclass
class Commit:
    tree_hash: int
    parent: int | None
    message: str
    timestamp_ms: int


class DiffKind(Enum):
    ADDED = "added"
    REMOVED = "removed"
    CHANGED = "changed"


@dataclass
class DiffOp:
    kind: DiffKind
    name: str
    from_hash: int = 0
    to_hash: int = 0


class Repo:
    def __init__(self) -> None:
        self.blobs: dict[int, Blob] = {}
        self.trees: dict[int, Tree] = {}
        self.commits: dict[int, Commit] = {}
        self.head: int | None = None

    def put_blob(self, data: bytes) -> int:
        h = fnv1a_hash(data)
        if h not in self.blobs:
            self.blobs[h] = Blob(data)
        return h

    def put_tree(self, entries: dict[str, int]) -> int:
        tree = Tree(dict(sorted(entries.items())))
        canonical = _canonical_tree_bytes(tree)
        h = fnv1a_hash(canonical)
        if h not in self.trees:
            self.trees[h] = tree
        return h

    def commit(self, files: list[tuple[str, bytes]], message: str,
               timestamp: int) -> int:
        entries = {name: self.put_blob(data) for name, data in files}
        tree_hash = self.put_tree(entries)
        c = Commit(tree_hash=tree_hash, parent=self.head,
                   message=message, timestamp_ms=timestamp)
        canonical = _canonical_commit_bytes(c)
        h = fnv1a_hash(canonical)
        self.commits[h] = c
        self.head = h
        return h

    def checkout(self, commit_hash: int) -> dict[str, bytes] | None:
        c = self.commits.get(commit_hash)
        if c is None:
            return None
        tree = self.trees.get(c.tree_hash)
        if tree is None:
            return None
        out = {}
        for name, bh in tree.entries.items():
            blob = self.blobs.get(bh)
            if blob is None:
                return None
            out[name] = blob.data
        return out

    def diff(self, from_hash: int, to_hash: int) -> list[DiffOp] | None:
        from_c = self.commits.get(from_hash)
        to_c = self.commits.get(to_hash)
        if from_c is None or to_c is None:
            return None
        from_tree = self.trees.get(from_c.tree_hash)
        to_tree = self.trees.get(to_c.tree_hash)
        if from_tree is None or to_tree is None:
            return None
        ops: list[DiffOp] = []
        for name, th in to_tree.entries.items():
            if name not in from_tree.entries:
                ops.append(DiffOp(DiffKind.ADDED, name, to_hash=th))
            elif from_tree.entries[name] != th:
                ops.append(DiffOp(DiffKind.CHANGED, name,
                                   from_hash=from_tree.entries[name], to_hash=th))
        for name, fh in from_tree.entries.items():
            if name not in to_tree.entries:
                ops.append(DiffOp(DiffKind.REMOVED, name, from_hash=fh))

        def key(op: DiffOp) -> str:
            prefix = {DiffKind.ADDED: "A", DiffKind.REMOVED: "R",
                      DiffKind.CHANGED: "C"}[op.kind]
            return prefix + op.name

        ops.sort(key=key)
        return ops

    def log(self) -> list[tuple[int, Commit]]:
        out = []
        cursor = self.head
        while cursor is not None:
            c = self.commits.get(cursor)
            if c is None:
                break
            out.append((cursor, c))
            cursor = c.parent
        return out

    def blob_count(self) -> int:
        return len(self.blobs)

    def commit_count(self) -> int:
        return len(self.commits)


def _canonical_tree_bytes(t: Tree) -> bytes:
    parts = [b"tree\n"]
    for name, h in sorted(t.entries.items()):
        parts.append(f"{name}\t{h:016x}\n".encode())
    return b"".join(parts)


def _canonical_commit_bytes(c: Commit) -> bytes:
    parts = [b"commit\n", f"tree {c.tree_hash:016x}\n".encode()]
    if c.parent is not None:
        parts.append(f"parent {c.parent:016x}\n".encode())
    parts.append(f"ts {c.timestamp_ms}\n".encode())
    parts.append(f"msg {c.message}\n".encode())
    return b"".join(parts)


def demo() -> None:
    r = Repo()
    c1 = r.commit([("a.txt", b"hello"), ("b.txt", b"world")], "initial", 1000)
    c2 = r.commit([("a.txt", b"hello!"), ("b.txt", b"world"), ("c.txt", b"new")],
                  "update a, add c", 2000)
    c3 = r.commit([("a.txt", b"hello!"), ("c.txt", b"new")],
                  "remove b", 3000)

    print("=== log ===")
    for h, c in r.log():
        p = f" <- {c.parent:016x}" if c.parent else ""
        print(f"  {h:016x} tree={c.tree_hash:016x}{p}  {c.message}")

    print(f"\n=== diff c1 -> c2 ===")
    for op in r.diff(c1, c2):
        print(f"  {op.kind.value:7} {op.name}")
    print(f"\n=== diff c2 -> c3 ===")
    for op in r.diff(c2, c3):
        print(f"  {op.kind.value:7} {op.name}")

    print(f"\n=== storage ===")
    print(f"  blobs: {r.blob_count()}  commits: {r.commit_count()}")
    print(f"  HEAD: {r.head:016x}")


if __name__ == "__main__":
    demo()
