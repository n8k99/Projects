"""CD Burning App — multi-session disc model + bin-packing + ISO filenames.

Multi-session overhead: each session costs leadin+leadout.
FFD bin-packing: largest file first, first fit.
ISO 9660 Level 1 names: uppercase, alnum+underscore, 8.3, ~N collisions.
Finalize FSM: Blank -> SessionOpen -> SessionClosed -> Finalized.
"""

from __future__ import annotations
from dataclasses import dataclass, field
from enum import Enum


LEADIN_BYTES = 13_000_000
LEADOUT_BYTES = 22_000_000


@dataclass
class BurnFile:
    name: str
    size_bytes: int


class SessionState(Enum):
    OPEN = "open"
    CLOSED = "closed"


@dataclass
class Session:
    tracks: list[BurnFile] = field(default_factory=list)
    state: SessionState = SessionState.OPEN

    def total_track_bytes(self) -> int:
        return sum(f.size_bytes for f in self.tracks)

    def overhead_bytes(self) -> int:
        return LEADIN_BYTES + LEADOUT_BYTES

    def total_bytes(self) -> int:
        return self.total_track_bytes() + self.overhead_bytes()


class DiscState(Enum):
    BLANK = "blank"
    SESSION_OPEN = "session_open"
    SESSION_CLOSED = "session_closed"
    FINALIZED = "finalized"


class BurnError(Exception):
    pass


class DiscFinalized(BurnError):
    pass


class NoOpenSession(BurnError):
    pass


class SessionAlreadyOpen(BurnError):
    pass


class NotEnoughSpace(BurnError):
    def __init__(self, needed: int, available: int):
        super().__init__(f"needed={needed} available={available}")
        self.needed = needed
        self.available = available


@dataclass
class Disc:
    capacity_bytes: int
    sessions: list[Session] = field(default_factory=list)
    state: DiscState = DiscState.BLANK

    def used_bytes(self) -> int:
        return sum(s.total_bytes() for s in self.sessions)

    def remaining_bytes(self) -> int:
        return max(0, self.capacity_bytes - self.used_bytes())

    def usable_for_next_file(self) -> int:
        if self.state == DiscState.FINALIZED:
            return 0
        if self.state == DiscState.SESSION_OPEN:
            return self.remaining_bytes()
        # Blank / SessionClosed: new session costs overhead
        return max(0, self.remaining_bytes() - (LEADIN_BYTES + LEADOUT_BYTES))

    def open_session(self) -> None:
        if self.state == DiscState.FINALIZED:
            raise DiscFinalized()
        if self.state == DiscState.SESSION_OPEN:
            raise SessionAlreadyOpen()
        self.sessions.append(Session())
        self.state = DiscState.SESSION_OPEN

    def add_file(self, file: BurnFile) -> None:
        if self.state == DiscState.FINALIZED:
            raise DiscFinalized()
        if self.state != DiscState.SESSION_OPEN:
            raise NoOpenSession()
        available = self.remaining_bytes()
        if file.size_bytes > available:
            raise NotEnoughSpace(file.size_bytes, available)
        self.sessions[-1].tracks.append(file)

    def close_session(self) -> None:
        if self.state == DiscState.FINALIZED:
            raise DiscFinalized()
        if self.state != DiscState.SESSION_OPEN:
            raise NoOpenSession()
        self.sessions[-1].state = SessionState.CLOSED
        self.state = DiscState.SESSION_CLOSED

    def finalize(self) -> None:
        if self.state == DiscState.FINALIZED:
            raise DiscFinalized()
        if self.state == DiscState.SESSION_OPEN:
            self.close_session()
        self.state = DiscState.FINALIZED


@dataclass
class PackResult:
    discs: list[list[BurnFile]]
    overflow: list[BurnFile]


def pack_files_ffd(files: list[BurnFile], disc_capacity: int, num_discs: int) -> PackResult:
    sorted_files = sorted(files, key=lambda f: -f.size_bytes)
    usable = max(0, disc_capacity - (LEADIN_BYTES + LEADOUT_BYTES))
    discs: list[list[BurnFile]] = [[] for _ in range(num_discs)]
    remaining = [usable] * num_discs
    overflow: list[BurnFile] = []
    for f in sorted_files:
        placed = False
        for i in range(num_discs):
            if f.size_bytes <= remaining[i]:
                remaining[i] -= f.size_bytes
                discs[i].append(f)
                placed = True
                break
        if not placed:
            overflow.append(f)
    return PackResult(discs, overflow)


def _clean_chars(s: str) -> str:
    return "".join(c.upper() for c in s if c.isascii() and (c.isalnum() or c == "_"))


def _split_ext(name: str) -> tuple[str, str]:
    i = name.rfind(".")
    if i > 0:
        return name[:i], name[i+1:]
    return name, ""


def canonicalize_iso_name(name: str, existing: set[str]) -> str:
    stem, ext = _split_ext(name)
    stem_clean = _clean_chars(stem)
    ext_trunc = _clean_chars(ext)[:3]
    stem_base = stem_clean[:8]
    base = f"{stem_base}.{ext_trunc}" if ext_trunc else stem_base
    if base not in existing:
        return base
    for n in range(1, 1000):
        suffix = f"~{n}"
        room = max(0, 8 - len(suffix))
        short = stem_clean[:room]
        candidate = f"{short}{suffix}.{ext_trunc}" if ext_trunc else f"{short}{suffix}"
        if candidate not in existing:
            return candidate
    return base


def demo() -> None:
    disc = Disc(capacity_bytes=700_000_000)
    disc.open_session()
    disc.add_file(BurnFile("album1.flac", 100_000_000))
    disc.add_file(BurnFile("album2.flac", 150_000_000))
    print(f"after 2 files: used={disc.used_bytes()} remaining={disc.remaining_bytes()}")

    disc.close_session()
    print(f"state: {disc.state.value}")

    disc.open_session()
    disc.add_file(BurnFile("bonus.mp3", 50_000_000))
    print(f"after session 2: sessions={len(disc.sessions)} used={disc.used_bytes()}")

    disc.finalize()
    print(f"final state: {disc.state.value}")

    files = [BurnFile("big.iso", 500_000_000),
             BurnFile("mid.mp4", 200_000_000),
             BurnFile("small.txt", 1_000_000)]
    result = pack_files_ffd(files, 700_000_000, 2)
    for i, pack in enumerate(result.discs):
        names = [f.name for f in pack]
        print(f"disc {i}: {names}")
    print(f"overflow: {[f.name for f in result.overflow]}")

    existing: set[str] = set()
    for name in ["Report.Document.Txt", "report.txt", "report.txt", "my file@2026!.txt"]:
        c = canonicalize_iso_name(name, existing)
        existing.add(c)
        print(f"  {name} -> {c}")


if __name__ == "__main__":
    demo()
