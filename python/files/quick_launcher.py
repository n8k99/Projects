"""Quick Launcher — ranked launchable items with frecency and file round-trip.

Second Files project. The persistent state of the launcher (what you've run,
how often, how recently) is itself a file round-trip: to_text/from_text
reproduce the exact same launcher. Matching has two layers — text match
(name-exact > name-prefix > name-substring > keyword) and frecency
(log(1+count) / (1+age_hours)). Text match dominates; frecency breaks ties.
"""

from __future__ import annotations
from dataclasses import dataclass, field
import math


@dataclass
class Entry:
    id: int
    name: str
    action: str
    keywords: list[str] = field(default_factory=list)
    launch_count: int = 0
    last_launched_ms: int = 0


@dataclass
class Match:
    entry_id: int
    score: float


class LaunchError(Exception):
    pass


class ParseError(Exception):
    pass


class Launcher:
    def __init__(self, frecency_weight: float = 0.1) -> None:
        self.entries: dict[int, Entry] = {}
        self.frecency_weight = frecency_weight

    def add(self, entry: Entry) -> None:
        self.entries[entry.id] = entry

    def get(self, entry_id: int) -> Entry | None:
        return self.entries.get(entry_id)

    def __len__(self) -> int:
        return len(self.entries)

    def query(self, text: str, now_ms: int) -> list[Match]:
        needle = text.strip().lower()
        matches: list[Match] = []
        for e in self.entries.values():
            tm = _text_match(needle, e)
            if tm == 0.0:
                continue
            fr = _frecency(e, now_ms)
            matches.append(Match(e.id, tm + self.frecency_weight * fr))
        matches.sort(key=lambda m: (-m.score, m.entry_id))
        return matches

    def record_launch(self, entry_id: int, now_ms: int) -> None:
        e = self.entries.get(entry_id)
        if e is None:
            raise LaunchError(f"unknown entry {entry_id}")
        e.launch_count += 1
        e.last_launched_ms = now_ms

    def to_text(self) -> str:
        out = ["LAUNCHER"]
        for entry_id in sorted(self.entries):
            e = self.entries[entry_id]
            out.append("---")
            out.append(f"id: {e.id}")
            out.append(f"name: {e.name}")
            out.append(f"action: {e.action}")
            for k in e.keywords:
                out.append(f"keyword: {k}")
            out.append(f"count: {e.launch_count}")
            out.append(f"last: {e.last_launched_ms}")
        return "\n".join(out) + "\n"

    @classmethod
    def from_text(cls, text: str) -> Launcher:
        lines = text.splitlines()
        if not lines or lines[0] != "LAUNCHER":
            raise ParseError("missing LAUNCHER header")
        launcher = cls()
        i = 1
        while i < len(lines):
            if lines[i] != "---":
                raise ParseError(f"expected --- separator at line {i+1}")
            i += 1
            entry, consumed = _parse_entry(lines[i:])
            launcher.add(entry)
            i += consumed
        return launcher


def _text_match(needle: str, entry: Entry) -> float:
    if not needle:
        return 0.5
    name_lc = entry.name.lower()
    if name_lc == needle:
        return 1.0
    if name_lc.startswith(needle):
        return 0.8
    if needle in name_lc:
        return 0.6
    for k in entry.keywords:
        k_lc = k.lower()
        if k_lc == needle:
            return 0.5
        if needle in k_lc:
            return 0.4
    return 0.0


def _frecency(entry: Entry, now_ms: int) -> float:
    if entry.launch_count == 0:
        return 0.0
    age_ms = max(0, now_ms - entry.last_launched_ms)
    age_hours = age_ms / 3_600_000.0
    return math.log1p(entry.launch_count) / (1.0 + age_hours)


def _parse_entry(lines: list[str]) -> tuple[Entry, int]:
    id_ = None
    name = None
    action = None
    keywords: list[str] = []
    count = 0
    last = 0
    consumed = 0
    for line in lines:
        if line == "---":
            break
        consumed += 1
        if ": " not in line:
            raise ParseError(f"no `: ` in {line!r}")
        key, _, val = line.partition(": ")
        if key == "id":
            id_ = int(val)
        elif key == "name":
            name = val
        elif key == "action":
            action = val
        elif key == "keyword":
            keywords.append(val)
        elif key == "count":
            count = int(val)
        elif key == "last":
            last = int(val)
    if id_ is None or name is None or action is None:
        raise ParseError("missing id/name/action")
    return (Entry(id_, name, action, keywords, count, last), consumed)


def demo() -> None:
    l = Launcher()
    l.add(Entry(1, "Firefox", "firefox", ["browser", "web"]))
    l.add(Entry(2, "Files", "nautilus", ["folder"]))
    l.add(Entry(3, "Terminal", "kitty", ["shell", "cli"]))
    l.add(Entry(4, "File Manager", "thunar", ["folder"]))

    print("=== Query 'file' (fresh, no launches yet) ===")
    for m in l.query("file", 0):
        e = l.get(m.entry_id)
        print(f"  {e.name:15} score={m.score:.3f}")

    print("\n=== record_launch(2) 5x at t=1_000_000, query 'file' at t=2_000_000 ===")
    for _ in range(5):
        l.record_launch(2, 1_000_000)
    for m in l.query("file", 2_000_000):
        e = l.get(m.entry_id)
        print(f"  {e.name:15} score={m.score:.3f}")

    print("\n=== Round-trip check ===")
    text = l.to_text()
    parsed = Launcher.from_text(text)
    print(f"  original entries: {len(l)}, parsed: {len(parsed)}")
    print(f"  id=2 count restored: {parsed.get(2).launch_count}")
    print(f"  id=2 timestamp restored: {parsed.get(2).last_launched_ms}")


if __name__ == "__main__":
    demo()
