"""Log File Maker — structured append-only log with level filtering and rotation.

Append-only, size-bounded active log. Threshold drops entries below severity.
Overflow moves oldest to archive. Parse/serialise tab-delimited for round-trip.
"""

from __future__ import annotations
from dataclasses import dataclass
from enum import IntEnum


class Level(IntEnum):
    DEBUG = 0
    INFO = 1
    WARN = 2
    ERROR = 3

    @classmethod
    def parse(cls, s: str) -> Level | None:
        return {
            "DEBUG": cls.DEBUG, "INFO": cls.INFO,
            "WARN": cls.WARN, "ERROR": cls.ERROR,
        }.get(s)

    def as_str(self) -> str:
        return {0: "DEBUG", 1: "INFO", 2: "WARN", 3: "ERROR"}[self.value]


@dataclass
class Entry:
    timestamp_ms: int
    level: Level
    category: str
    message: str

    def __str__(self) -> str:
        return f"{self.timestamp_ms}\t{self.level.as_str()}\t{self.category}\t{self.message}"


class Logger:
    def __init__(self, level_threshold: Level, max_entries: int) -> None:
        self.level_threshold = level_threshold
        self.max_entries = max_entries
        self.active: list[Entry] = []
        self.archive: list[Entry] = []

    def log(self, ts: int, level: Level, category: str, message: str) -> None:
        if level < self.level_threshold:
            return
        self.active.append(Entry(ts, level, category, message))
        while len(self.active) > self.max_entries:
            self.archive.append(self.active.pop(0))

    def query_level(self, min_level: Level) -> list[Entry]:
        return [e for e in self.active if e.level >= min_level]

    def query_category(self, category: str) -> list[Entry]:
        return [e for e in self.active if e.category == category]

    def query_message_contains(self, needle: str) -> list[Entry]:
        lc = needle.lower()
        return [e for e in self.active if lc in e.message.lower()]

    def query_time_range(self, from_ms: int, to_ms: int) -> list[Entry]:
        return [e for e in self.active if from_ms <= e.timestamp_ms <= to_ms]

    def to_text(self) -> str:
        if not self.active:
            return ""
        return "\n".join(str(e) for e in self.active) + "\n"

    @classmethod
    def parse_entries(cls, text: str) -> list[Entry] | None:
        out = []
        for line in text.split("\n"):
            if line == "":
                continue
            parts = line.split("\t", 3)
            if len(parts) != 4:
                return None
            try:
                ts = int(parts[0])
            except ValueError:
                return None
            level = Level.parse(parts[1])
            if level is None:
                return None
            out.append(Entry(ts, level, parts[2], parts[3]))
        return out

    def total_archived(self) -> int:
        return len(self.archive)

    def active_len(self) -> int:
        return len(self.active)


def demo() -> None:
    l = Logger(Level.DEBUG, max_entries=100)
    l.log(100, Level.INFO, "boot", "starting up")
    l.log(200, Level.DEBUG, "boot", "loaded config")
    l.log(300, Level.WARN, "net", "slow response")
    l.log(400, Level.ERROR, "db", "connection lost")
    l.log(500, Level.INFO, "db", "reconnected")

    print("=== all entries ===")
    print(l.to_text())

    print("=== >= WARN ===")
    for e in l.query_level(Level.WARN):
        print(f"  {e}")

    print("\n=== category db ===")
    for e in l.query_category("db"):
        print(f"  {e}")

    print("\n=== parse round-trip ===")
    parsed = Logger.parse_entries(l.to_text())
    print(f"  {len(parsed)} entries round-tripped")

    print("\n=== rotation demo ===")
    l2 = Logger(Level.DEBUG, max_entries=3)
    for i in range(6):
        l2.log(i * 10, Level.INFO, "x", f"m{i}")
    print(f"  active: {[e.message for e in l2.active]}, archived: {l2.total_archived()}")


if __name__ == "__main__":
    demo()
