"""Zip File Maker — archive multiple named files with per-entry compression.

RLE compression: consecutive runs become (count, byte) pairs. Max run 255.
add_file picks the smaller of Store vs RLE per entry.
Round-trip: extract(add_file(data)) == data.
"""

from __future__ import annotations
from dataclasses import dataclass, field
from enum import Enum


class Method(Enum):
    STORE = "store"
    RLE = "rle"


@dataclass
class Entry:
    name: str
    method: Method
    size_original: int
    data: bytes  # raw if STORE, encoded if RLE


@dataclass
class Archive:
    entries: list[Entry] = field(default_factory=list)

    def add_file(self, name: str, data: bytes) -> None:
        rle = rle_encode(data)
        if len(rle) < len(data):
            self.entries.append(Entry(name, Method.RLE, len(data), rle))
        else:
            self.entries.append(Entry(name, Method.STORE, len(data), data))

    def add_stored(self, name: str, data: bytes) -> None:
        self.entries.append(Entry(name, Method.STORE, len(data), data))

    def compressed_size(self) -> int:
        return sum(len(e.data) for e in self.entries)

    def compression_ratio(self) -> float:
        orig = sum(e.size_original for e in self.entries)
        return 1.0 if orig == 0 else self.compressed_size() / orig

    def extract(self, name: str) -> bytes | None:
        for e in self.entries:
            if e.name == name:
                return e.data if e.method == Method.STORE else rle_decode(e.data)
        return None

    def names(self) -> list[str]:
        return [e.name for e in self.entries]


def rle_encode(data: bytes) -> bytes:
    out = bytearray()
    i = 0
    while i < len(data):
        byte = data[i]
        run = 1
        while i + run < len(data) and data[i + run] == byte and run < 255:
            run += 1
        out.append(run)
        out.append(byte)
        i += run
    return bytes(out)


def rle_decode(encoded: bytes) -> bytes:
    out = bytearray()
    i = 0
    while i + 1 < len(encoded):
        count = encoded[i]
        byte = encoded[i + 1]
        out.extend(bytes([byte]) * count)
        i += 2
    return bytes(out)


def demo() -> None:
    a = Archive()
    a.add_file("repetitive.txt", b"aaaaaaaaaa")
    a.add_file("random.bin", bytes([1, 2, 3, 4, 5]))
    a.add_file("big.txt", b"x" * 1000)
    a.add_file("mixed.txt", b"the quick brown fox aaaaaaa xyz")

    print(f"names: {a.names()}")
    for e in a.entries:
        print(f"  {e.name:18}  method={e.method.value:5}  "
              f"orig={e.size_original:4}  stored={len(e.data):4}")
    print(f"\ntotal compressed: {a.compressed_size()} bytes")
    print(f"ratio: {a.compression_ratio():.4f}")

    # Round-trip check
    assert a.extract("repetitive.txt") == b"aaaaaaaaaa"
    assert a.extract("big.txt") == b"x" * 1000
    assert a.extract("mixed.txt") == b"the quick brown fox aaaaaaa xyz"
    print("\nround-trip: all entries extract cleanly")


if __name__ == "__main__":
    demo()
