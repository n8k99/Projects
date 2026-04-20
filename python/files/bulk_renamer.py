"""Bulk Renamer and Organizer — pattern-driven rename + preview + undo log.

Dry-run first (preview), then apply. Apply returns undo log = reverse ops.
Collisions caught in preview; apply is atomic.
"""

from __future__ import annotations
from dataclasses import dataclass
from enum import Enum


class RuleKind(Enum):
    REPLACE = "replace"
    PREFIX = "prefix"
    SUFFIX_BEFORE_EXT = "suffix_before_ext"
    SEQUENCE = "sequence"


@dataclass
class Rule:
    kind: RuleKind
    find: str = ""
    replace: str = ""
    prefix: str = ""
    suffix: str = ""
    template: str = ""


@dataclass
class RenameOp:
    from_: str
    to: str


class RenameError(Exception):
    pass


class FromMissing(RenameError):
    pass


class ToExists(RenameError):
    pass


class Collision(RenameError):
    pass


class Directory:
    def __init__(self, names: list[str] | None = None) -> None:
        self.entries: set[str] = set(names or [])

    def names(self) -> list[str]:
        return sorted(self.entries)

    def __len__(self) -> int:
        return len(self.entries)

    def preview(self, rule: Rule) -> list[RenameOp]:
        ops = []
        seen_targets: dict[str, str] = {}
        for i, name in enumerate(sorted(self.entries)):
            new_name = apply_rule(name, rule, i)
            if new_name == name:
                continue
            if new_name in seen_targets:
                raise Collision(f"{seen_targets[new_name]} and {name} both → {new_name}")
            seen_targets[new_name] = name
            ops.append(RenameOp(name, new_name))
        rename_sources = {o.from_ for o in ops}
        for op in ops:
            if op.to in self.entries and op.to not in rename_sources:
                raise ToExists(op.to)
        return ops

    def apply(self, ops: list[RenameOp]) -> list[RenameOp]:
        for op in ops:
            if op.from_ not in self.entries:
                raise FromMissing(op.from_)
        for op in ops:
            self.entries.discard(op.from_)
        for op in ops:
            self.entries.add(op.to)
        return [RenameOp(o.to, o.from_) for o in ops]

    def undo(self, log: list[RenameOp]) -> None:
        self.apply(log)


def apply_rule(name: str, rule: Rule, index: int) -> str:
    if rule.kind == RuleKind.REPLACE:
        pos = name.find(rule.find)
        if pos < 0:
            return name
        return name[:pos] + rule.replace + name[pos + len(rule.find):]
    if rule.kind == RuleKind.PREFIX:
        return rule.prefix + name
    if rule.kind == RuleKind.SUFFIX_BEFORE_EXT:
        dot = name.rfind(".")
        if dot < 0:
            return name + rule.suffix
        return name[:dot] + rule.suffix + name[dot:]
    if rule.kind == RuleKind.SEQUENCE:
        return rule.template.replace("{n}", str(index + 1))
    return name


def demo() -> None:
    d = Directory(["img_01.png", "img_02.png", "img_03.png", "notes.txt"])
    print(f"before: {d.names()}")

    rule = Rule(RuleKind.REPLACE, find="img_", replace="photo_")
    ops = d.preview(rule)
    print(f"preview: {[(o.from_, o.to) for o in ops]}")
    undo_log = d.apply(ops)
    print(f"after: {d.names()}")
    d.undo(undo_log)
    print(f"undone: {d.names()}")

    print("\n=== sequence rename ===")
    d2 = Directory(["track_a.wav", "track_b.wav", "track_c.wav"])
    rule = Rule(RuleKind.SEQUENCE, template="song_{n}.mp3")
    ops = d2.preview(rule)
    print(f"preview: {[(o.from_, o.to) for o in ops]}")
    d2.apply(ops)
    print(f"after: {d2.names()}")


if __name__ == "__main__":
    demo()
