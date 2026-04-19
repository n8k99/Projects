"""Password Safe — encrypted-at-rest credential store with session FSM.

**Not a real cryptographic password manager.** Uses a byte-sum hash + XOR
keystream to *demonstrate* encryption-at-rest structure without pretending
to be secure. A production version would use Argon2id + AES-GCM.

What G077 *does* teach:
  - Encryption-at-rest: stored bytes are unreadable without a key derived
    from the master password.
  - Lock/unlock FSM (extends G073's protocol-state pattern).
  - Auto-lock on inactivity (time-based state transition).
  - Password generation from a declarative policy.
  - Password strength scoring as a separate, testable concern.
"""

from __future__ import annotations
from dataclasses import dataclass, field
from enum import Enum


class Strength(Enum):
    WEAK = "weak"
    FAIR = "fair"
    STRONG = "strong"
    VERY_STRONG = "very_strong"


@dataclass
class GeneratePolicy:
    length: int = 16
    lower: bool = True
    upper: bool = True
    digits: bool = True
    symbols: bool = True


@dataclass
class _Entry:
    id: int
    service: str
    username: str
    encrypted_password: bytes
    notes: str
    created_at_ms: int
    modified_at_ms: int


@dataclass
class EntrySummary:
    """Entry projection for search results — never includes the password."""
    id: int
    service: str
    username: str
    notes: str
    created_at_ms: int
    modified_at_ms: int


def _verifier_hash(master: str, salt: bytes) -> bytes:
    out = bytearray()
    for r in range(32):
        h = r
        for b in master.encode():
            h = (h * 131 + b) & 0xFFFFFFFFFFFFFFFF
        for b in salt:
            h = (h * 131 + b) & 0xFFFFFFFFFFFFFFFF
        out.append(h & 0xFF)
    return bytes(out)


def _derive_key(master: str, salt: bytes) -> bytes:
    out = bytearray()
    for r in range(128, 192):
        h = r
        for b in master.encode():
            h = (h * 197 + b) & 0xFFFFFFFFFFFFFFFF
        for b in salt:
            h = (h * 197 + b) & 0xFFFFFFFFFFFFFFFF
        out.append(h & 0xFF)
    return bytes(out)


def _xor(data: bytes, key: bytes) -> bytes:
    return bytes(b ^ key[i % len(key)] for i, b in enumerate(data))


class Safe:
    def __init__(self, master: str, auto_lock_after_ms: int) -> None:
        self._salt = bytes(range(16))
        self._verifier = _verifier_hash(master, self._salt)
        self._entries: list[_Entry] = []
        self._next_id = 1
        self._key: bytes | None = None
        self._last_activity_ms = 0
        self._auto_lock_after_ms = auto_lock_after_ms

    def is_locked(self) -> bool:
        return self._key is None

    def unlock(self, master: str, now_ms: int) -> bool:
        if _verifier_hash(master, self._salt) != self._verifier:
            return False
        self._key = _derive_key(master, self._salt)
        self._last_activity_ms = now_ms
        return True

    def lock(self) -> None:
        self._key = None

    def check_auto_lock(self, now_ms: int) -> None:
        if self._key is not None and now_ms >= self._last_activity_ms + self._auto_lock_after_ms:
            self.lock()

    def _touch(self, now_ms: int) -> bytes | None:
        if self._key is None:
            return None
        self._last_activity_ms = now_ms
        return self._key

    def add_entry(self, service: str, username: str, password: str,
                  notes: str, now_ms: int) -> int | None:
        key = self._touch(now_ms)
        if key is None:
            return None
        eid = self._next_id
        self._next_id += 1
        self._entries.append(_Entry(
            id=eid, service=service, username=username,
            encrypted_password=_xor(password.encode(), key),
            notes=notes, created_at_ms=now_ms, modified_at_ms=now_ms,
        ))
        return eid

    def get_password(self, eid: int, now_ms: int) -> str | None:
        key = self._touch(now_ms)
        if key is None:
            return None
        entry = next((e for e in self._entries if e.id == eid), None)
        if entry is None:
            return None
        return _xor(entry.encrypted_password, key).decode()

    def get_summary(self, eid: int) -> EntrySummary | None:
        entry = next((e for e in self._entries if e.id == eid), None)
        if entry is None:
            return None
        return EntrySummary(
            id=entry.id, service=entry.service, username=entry.username,
            notes=entry.notes,
            created_at_ms=entry.created_at_ms, modified_at_ms=entry.modified_at_ms,
        )

    def remove_entry(self, eid: int, now_ms: int) -> bool:
        if self._touch(now_ms) is None:
            return False
        for i, e in enumerate(self._entries):
            if e.id == eid:
                del self._entries[i]
                return True
        return False

    def search(self, query: str, now_ms: int) -> list[EntrySummary]:
        if self._touch(now_ms) is None:
            return []
        q = query.lower()
        return [EntrySummary(
                    id=e.id, service=e.service, username=e.username,
                    notes=e.notes,
                    created_at_ms=e.created_at_ms, modified_at_ms=e.modified_at_ms,
                ) for e in self._entries
                if q in e.service.lower() or q in e.username.lower()
                   or q in e.notes.lower()]

    def entry_count(self) -> int:
        return len(self._entries)


def generate_password(policy: GeneratePolicy, seed: int) -> str:
    alphabet = ""
    if policy.lower:   alphabet += "abcdefghijklmnopqrstuvwxyz"
    if policy.upper:   alphabet += "ABCDEFGHIJKLMNOPQRSTUVWXYZ"
    if policy.digits:  alphabet += "0123456789"
    if policy.symbols: alphabet += "!@#$%^&*()-_=+[]{}"
    if not alphabet:
        return ""
    state = (seed * 6364136223846793005 + 1442695040888963407) & 0xFFFFFFFFFFFFFFFF
    out = []
    for _ in range(policy.length):
        state = (state * 6364136223846793005 + 1442695040888963407) & 0xFFFFFFFFFFFFFFFF
        out.append(alphabet[(state >> 32) % len(alphabet)])
    return "".join(out)


def password_strength(pw: str) -> Strength:
    length = len(pw)
    classes = 0
    if any(c.islower() for c in pw): classes += 1
    if any(c.isupper() for c in pw): classes += 1
    if any(c.isdigit() for c in pw): classes += 1
    if any(not c.isalnum() for c in pw): classes += 1

    if length < 8:
        return Strength.WEAK
    if length < 12 and classes < 3:
        return Strength.WEAK
    if length >= 20:
        return Strength.VERY_STRONG
    if length >= 14 and classes >= 3:
        return Strength.STRONG
    if classes >= 4:
        return Strength.STRONG
    return Strength.FAIR


if __name__ == "__main__":
    safe = Safe("my-master-passphrase", auto_lock_after_ms=30_000)
    print(f"starts locked: {safe.is_locked()}")

    ok = safe.unlock("wrong", now_ms=1000)
    print(f"wrong master unlocks: {ok}")

    ok = safe.unlock("my-master-passphrase", now_ms=1000)
    print(f"correct master unlocks: {ok}")

    gen = generate_password(GeneratePolicy(length=20), seed=42)
    print(f"generated: {gen} (strength={password_strength(gen).value})")

    id1 = safe.add_entry("github.com", "alice@example.com",
                         "s3cret-p@ssword!", "main dev account", 1000)
    safe.add_entry("news.ycombinator.com", "alice",
                   gen, "throwaway discussion account", 1500)

    print()
    print(f"entries: {safe.entry_count()}")
    for s in safe.search("git", 2000):
        print(f"  {s.service} / {s.username}  ({s.notes})")

    pw = safe.get_password(id1, 2500)
    print()
    print(f"retrieved GitHub password: {pw}")

    safe.check_auto_lock(40_000)
    print(f"after 40s idle, locked: {safe.is_locked()}")
    pw = safe.get_password(id1, 41_000)
    print(f"can still read? {pw}")
