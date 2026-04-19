"""File Downloader — resumable transfer with atomic file creation.

Where G066 was about a *pool* of downloads (fan-out), G072 is about the
correctness of a *single* download:

  1. **Atomic rename at end.** Bytes go to ``<dest>.partial`` during
     transfer; ``os.replace(.partial, dest)`` promotes the file in one
     syscall. Observers never see a half-written file at ``dest``.

  2. **Resumable from .partial.** If the process dies, the ``.partial``
     persists. The next call checks its size and resumes from that byte
     offset. No work is re-done.

  3. **Hash-verified before promotion.** Optional expected-hash parameter;
     mismatch refuses the rename and keeps ``.partial`` for inspection.

The transport is a callable ``read_from(offset, size) -> bytes``; tests
use an in-memory fake, production uses ``urllib`` with a Range header.
"""

from __future__ import annotations
import os
from dataclasses import dataclass
from pathlib import Path
from typing import Callable


def byte_sum_hash(b: bytes) -> int:
    h = 0
    for byte in b:
        h = (h * 31 + byte) & 0xFFFFFFFFFFFFFFFF
    return (h + len(b)) & 0xFFFFFFFFFFFFFFFF


@dataclass
class DownloadOpts:
    resume: bool = True
    chunk_size: int = 8192
    expected_hash: int | None = None


@dataclass
class Outcome:
    bytes_written: int
    final_path: Path
    verified: bool


class HashMismatch(Exception):
    def __init__(self, expected: int, actual: int) -> None:
        super().__init__(f"hash mismatch: expected {expected:x}, got {actual:x}")
        self.expected = expected
        self.actual = actual


def _partial_path(dest: Path) -> Path:
    return dest.parent / (dest.name + ".partial")


class _Hasher:
    """Streamable version of byte_sum_hash."""
    __slots__ = ("h", "len")

    def __init__(self) -> None:
        self.h = 0
        self.len = 0

    def update(self, b: bytes) -> None:
        h = self.h
        for byte in b:
            h = (h * 31 + byte) & 0xFFFFFFFFFFFFFFFF
        self.h = h
        self.len += len(b)

    def finalize(self) -> int:
        return (self.h + self.len) & 0xFFFFFFFFFFFFFFFF


def download(
    read_from: Callable[[int, int], bytes],
    dest: Path,
    opts: DownloadOpts = DownloadOpts(),
    on_progress: Callable[[int], None] = lambda _: None,
) -> Outcome:
    """Download into ``dest`` via ``.partial`` + atomic rename.

    ``read_from(offset, size)`` returns up to ``size`` bytes starting at
    ``offset``; returning ``b""`` signals EOF. Raising an exception aborts
    the download but leaves ``.partial`` intact for later resume.
    """
    partial = _partial_path(dest)
    hasher = _Hasher()
    start_offset = 0

    if opts.resume and partial.exists():
        with partial.open("rb") as f:
            existing = f.read()
        hasher.update(existing)
        start_offset = len(existing)
        handle = partial.open("ab")
    else:
        handle = partial.open("wb")

    total = start_offset
    try:
        while True:
            chunk = read_from(total, opts.chunk_size)
            if not chunk:
                break
            handle.write(chunk)
            hasher.update(chunk)
            total += len(chunk)
            on_progress(total)
        handle.flush()
        os.fsync(handle.fileno())
    finally:
        handle.close()

    actual = hasher.finalize()
    if opts.expected_hash is not None and opts.expected_hash != actual:
        raise HashMismatch(opts.expected_hash, actual)

    os.replace(partial, dest)
    return Outcome(bytes_written=total, final_path=dest,
                   verified=opts.expected_hash is not None)


if __name__ == "__main__":
    import tempfile

    # Simulate a 10 KB download with a mid-stream interruption, then resume.
    content = bytes(i % 256 for i in range(10_000))
    expected = byte_sum_hash(content)

    with tempfile.TemporaryDirectory() as td:
        dest = Path(td) / "payload.bin"

        # First attempt: fail after 3000 bytes.
        def flaky_transport(offset: int, size: int) -> bytes:
            end = offset + size
            if offset >= 3000 and offset < 6000:
                # Return what we can up to 3000 then raise.
                if offset < 3000:
                    end = min(end, 3000)
                    return content[offset:end]
                raise RuntimeError("simulated network failure at offset " + str(offset))
            return content[offset:min(end, len(content))]

        try:
            download(flaky_transport, dest)
        except RuntimeError as e:
            print(f"first attempt failed as expected: {e}")
            partial = _partial_path(dest)
            print(f"  .partial preserved with {partial.stat().st_size} bytes")

        # Second attempt with the real transport, hash verification.
        def real_transport(offset: int, size: int) -> bytes:
            return content[offset:offset + size]

        outcome = download(
            real_transport, dest,
            DownloadOpts(expected_hash=expected),
            on_progress=lambda b: None,
        )
        print(f"resume succeeded: {outcome.bytes_written} bytes, verified={outcome.verified}")
        assert dest.stat().st_size == len(content)
        assert dest.read_bytes() == content
        print("final file matches original content")
