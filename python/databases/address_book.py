"""Address Book — contacts with normalised-identifier deduplication and merge.

Canonical email (lowercase, gmail-dot-strip, plus-tag-strip), canonical
phone (digit-only, US +1 normalised). Duplicate detection via shared
canonical identifier (union-find). Merge = union of fields + longest
name + earliest timestamp.
"""

from __future__ import annotations
from dataclasses import dataclass, field


@dataclass
class Contact:
    id: int
    name: str
    created_ms: int
    emails: list[str] = field(default_factory=list)
    phones: list[str] = field(default_factory=list)
    tags: list[str] = field(default_factory=list)
    address: str = ""
    notes: str = ""

    def canonical_emails(self) -> list[str]:
        return [canonical_email(e) for e in self.emails]

    def canonical_phones(self) -> list[str]:
        return [canonical_phone(p) for p in self.phones]


def canonical_email(email: str) -> str:
    lower = email.strip().lower()
    if "@" not in lower:
        return lower
    local, _, domain = lower.partition("@")
    if "+" in local:
        local = local.split("+", 1)[0]
    if domain in ("gmail.com", "googlemail.com"):
        local = local.replace(".", "")
        domain = "gmail.com"
    return f"{local}@{domain}"


def canonical_phone(phone: str) -> str:
    digits = "".join(c for c in phone if c.isdigit())
    if len(digits) == 11 and digits.startswith("1"):
        return digits[1:]
    return digits


class AddressBook:
    def __init__(self) -> None:
        self.contacts: dict[int, Contact] = {}
        self._next_id = 1

    def add(self, contact: Contact) -> int:
        if contact.id >= self._next_id:
            self._next_id = contact.id + 1
        self.contacts[contact.id] = contact
        return contact.id

    def new_id(self) -> int:
        i = self._next_id
        self._next_id += 1
        return i

    def get(self, id: int) -> Contact | None:
        return self.contacts.get(id)

    def find_duplicates(self) -> list[list[int]]:
        by_email: dict[str, set[int]] = {}
        by_phone: dict[str, set[int]] = {}
        for c in self.contacts.values():
            for e in c.canonical_emails():
                if e:
                    by_email.setdefault(e, set()).add(c.id)
            for p in c.canonical_phones():
                if p:
                    by_phone.setdefault(p, set()).add(c.id)

        parent = {id: id for id in self.contacts}

        def find(x):
            while parent[x] != x:
                parent[x] = parent[parent[x]]
                x = parent[x]
            return x

        def union(a, b):
            ra, rb = find(a), find(b)
            if ra != rb:
                parent[ra] = rb

        for group in list(by_email.values()) + list(by_phone.values()):
            ids = list(group)
            for i in range(1, len(ids)):
                union(ids[0], ids[i])

        groups: dict[int, list[int]] = {}
        for id in sorted(self.contacts):
            r = find(id)
            groups.setdefault(r, []).append(id)

        out = [sorted(g) for g in groups.values() if len(g) >= 2]
        out.sort(key=lambda g: g[0])
        return out

    def merge(self, ids: list[int]) -> int:
        if len(ids) < 2:
            raise ValueError("need at least 2 ids to merge")
        sources = []
        for id in ids:
            c = self.contacts.get(id)
            if c is None:
                raise ValueError(f"id {id} not found")
            sources.append(c)

        merged_id = self.new_id()
        earliest = min(c.created_ms for c in sources)
        non_empty_names = [c.name for c in sources if c.name]
        longest = max(non_empty_names, key=len) if non_empty_names else ""

        emails: set[str] = set()
        phones: set[str] = set()
        tags: set[str] = set()
        addrs = []
        notes = []
        for c in sources:
            emails.update(e for e in c.emails if e)
            phones.update(p for p in c.phones if p)
            tags.update(t for t in c.tags if t)
            if c.address:
                addrs.append(c.address)
            if c.notes:
                notes.append(c.notes)

        merged = Contact(
            id=merged_id, name=longest, created_ms=earliest,
            emails=sorted(emails), phones=sorted(phones), tags=sorted(tags),
            address=" | ".join(addrs),
            notes="\n---\n".join(notes),
        )
        for id in ids:
            del self.contacts[id]
        self.contacts[merged_id] = merged
        return merged_id

    def search_by_name(self, needle: str) -> list[int]:
        lc = needle.lower()
        return sorted(c.id for c in self.contacts.values() if lc in c.name.lower())

    def search_by_email(self, email: str) -> list[int]:
        canonical = canonical_email(email)
        return sorted(
            c.id for c in self.contacts.values()
            if any(e == canonical for e in c.canonical_emails())
        )


def demo() -> None:
    ab = AddressBook()
    ab.add(Contact(1, "Alice", 100, emails=["alice@gmail.com"], phones=["555-1111"]))
    ab.add(Contact(2, "Alice J. Smith", 200,
                   emails=["a.l.i.c.e@gmail.com", "alice.smith@example.com"],
                   phones=["(555) 2222"]))
    ab.add(Contact(3, "Bob", 300, emails=["bob@gmail.com"], phones=["555-3333"]))

    print("=== duplicates ===")
    for group in ab.find_duplicates():
        names = [ab.get(id).name for id in group]
        print(f"  group {group}: {names}")

    print("\n=== merging 1 and 2 ===")
    merged_id = ab.merge([1, 2])
    merged = ab.get(merged_id)
    print(f"  new id: {merged_id}")
    print(f"  name: {merged.name}")
    print(f"  emails: {merged.emails}")
    print(f"  phones: {merged.phones}")
    print(f"  earliest created_ms: {merged.created_ms}")

    print("\n=== remaining contacts ===")
    for id in sorted(ab.contacts):
        print(f"  {id}: {ab.get(id).name}")


if __name__ == "__main__":
    demo()
