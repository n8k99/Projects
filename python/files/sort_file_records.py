"""Sort File Records Utility — parse delimited records, multi-key sort.

Records are tables: comma-delimited lines become typed rows. Sort spec is
a list of (field, order, kind); multi-key sort composes from primitives.
Round-trip: parse → serialise preserves exact bytes when unsorted.
"""

from __future__ import annotations
from dataclasses import dataclass
from enum import Enum


class Order(Enum):
    ASC = "asc"
    DESC = "desc"


class Kind(Enum):
    STR = "str"
    INT = "int"


@dataclass
class SortKey:
    field: int
    order: Order = Order.ASC
    kind: Kind = Kind.STR


def _field_from_str(s: str) -> int | str:
    try:
        return int(s)
    except ValueError:
        return s


def _field_as_str(f: int | str) -> str:
    return str(f)


@dataclass
class Record:
    fields: list[int | str]

    @classmethod
    def parse(cls, line: str) -> Record:
        return cls([_field_from_str(p.strip()) for p in line.split(",")])

    def serialise(self) -> str:
        return ",".join(_field_as_str(f) for f in self.fields)

    def get(self, i: int) -> int | str | None:
        return self.fields[i] if i < len(self.fields) else None


@dataclass
class Table:
    header: list[str] | None
    rows: list[Record]

    @classmethod
    def parse(cls, text: str, has_header: bool) -> Table:
        lines = [l for l in text.split("\n") if l != ""]
        header = None
        if has_header and lines:
            header = [h.strip() for h in lines[0].split(",")]
            lines = lines[1:]
        return cls(header, [Record.parse(l) for l in lines])

    def serialise(self) -> str:
        out = []
        if self.header is not None:
            out.append(",".join(self.header))
        for r in self.rows:
            out.append(r.serialise())
        return "\n".join(out) + "\n"

    def sort_by(self, spec: list[SortKey]) -> None:
        def key(record: Record) -> tuple:
            parts = []
            for sk in spec:
                v = record.get(sk.field)
                # missing -> sorts first in asc, last in desc
                missing_sentinel = 0 if sk.order == Order.ASC else 1
                if v is None:
                    parts.append((missing_sentinel, None))
                else:
                    converted = (
                        int(v) if sk.kind == Kind.INT and isinstance(v, str)
                        else (v if sk.kind == Kind.INT else str(v))
                    )
                    parts.append((1 - missing_sentinel, converted))
            return tuple(parts)

        # Python sort is stable — apply keys in reverse order to honour priority.
        for sk in reversed(spec):
            self.rows.sort(
                key=lambda r, sk=sk: _sort_key_single(r, sk),
                reverse=(sk.order == Order.DESC),
            )

    def field_index(self, name: str) -> int | None:
        if self.header is None:
            return None
        name_l = name.lower()
        for i, h in enumerate(self.header):
            if h.lower() == name_l:
                return i
        return None


def _sort_key_single(r: Record, sk: SortKey):
    v = r.get(sk.field)
    if v is None:
        # Missing field: want it to sort first always. Use a two-tuple
        # where present=1, missing=0; for desc, invert sentinel so missing
        # still ends up as sorted-first after reverse.
        if sk.order == Order.ASC:
            return (0, 0)
        else:
            return (1, 0)  # after reverse → effectively first
    if sk.kind == Kind.INT:
        v2 = int(v) if isinstance(v, str) else v
    else:
        v2 = str(v)
    return (1, v2) if sk.order == Order.ASC else (0, v2)


def demo() -> None:
    sample = "name,age,score\nAlice,30,85\nBob,25,92\nCarol,30,78\nDave,25,88\n"
    t = Table.parse(sample, has_header=True)
    print(f"header: {t.header}")
    print(f"rows before sort: {[r.serialise() for r in t.rows]}")

    print("\n=== sort by age asc, score desc ===")
    t.sort_by([
        SortKey(field=1, order=Order.ASC, kind=Kind.INT),
        SortKey(field=2, order=Order.DESC, kind=Kind.INT),
    ])
    for r in t.rows:
        print(f"  {r.serialise()}")

    print("\n=== serialise round trip ===")
    t2 = Table.parse(sample, has_header=True)
    print(f"  unchanged: {t2.serialise() == sample}")


if __name__ == "__main__":
    demo()
