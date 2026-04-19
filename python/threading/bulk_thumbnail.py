"""Bulk Thumbnail Creator — CPU-bound fan-out with content-addressed dedup.

G066's pattern was fan-out with isolated per-job results appended to a list.
G068 is fan-out with a **shared index** as the output: the result IS the
aggregation, not a side-effect of it. Workers compute content hashes and
check the index before generating a thumbnail — so the same content
processed twice (by two different source files, or in two requests)
produces one thumbnail, and the second worker's entry simply records the
already-computed result.

Two sources of concurrency cost:
  1. Queue pop (inherited from G066).
  2. Index check-and-insert (new to G068). The reservation pattern releases
     the lock before running the expensive generator: claim under lock,
     generate unlocked, install under lock. Concurrent workers with the same
     hash see the reservation and treat it as "already present."

The hasher and thumbnailer are parameters so the pattern works for any
content-addressed derivation: image → thumbnail, source code → AST,
document → summary, ledger entry → balance.

Note: Python's GIL serialises CPU-bound work on a single core. Real
parallel speedup for CPU-bound tasks comes from multiprocessing, not
threading. G068's Python version still demonstrates the dedup pattern
correctly; speedup is tested for I/O-bound simulated thumbnailers.
"""

from __future__ import annotations
import queue
import threading
from dataclasses import dataclass, field
from typing import Callable


@dataclass
class ThumbnailRequest:
    source_id: str
    content: bytes


@dataclass
class ThumbnailIndexSnapshot:
    thumbnails: dict[int, str] = field(default_factory=dict)
    mappings: dict[str, int] = field(default_factory=dict)
    generated: int = 0
    deduplicated: int = 0


class BulkThumbnailCreator:
    def __init__(self, worker_count: int) -> None:
        if worker_count < 1:
            raise ValueError("worker_count must be >= 1")
        self._worker_count = worker_count
        self._queue: queue.Queue[ThumbnailRequest] = queue.Queue()
        self._thumbnails: dict[int, str] = {}
        self._mappings: dict[str, int] = {}
        self._generated = 0
        self._deduplicated = 0
        self._index_lock = threading.Lock()

    def enqueue(self, req: ThumbnailRequest) -> None:
        self._queue.put(req)

    def run_with(
        self,
        hasher: Callable[[bytes], int],
        thumbnailer: Callable[[int, bytes], str],
    ) -> ThumbnailIndexSnapshot:
        def worker() -> None:
            while True:
                try:
                    req = self._queue.get_nowait()
                except queue.Empty:
                    return
                hash_ = hasher(req.content)

                # Claim under lock; generate unlocked.
                with self._index_lock:
                    if hash_ in self._thumbnails:
                        is_claimant = False
                    else:
                        self._thumbnails[hash_] = ""  # reservation
                        is_claimant = True

                if is_claimant:
                    thumb = thumbnailer(hash_, req.content)
                    with self._index_lock:
                        self._thumbnails[hash_] = thumb
                        self._generated += 1
                else:
                    with self._index_lock:
                        self._deduplicated += 1

                with self._index_lock:
                    self._mappings[req.source_id] = hash_

        threads = [threading.Thread(target=worker, daemon=True)
                   for _ in range(self._worker_count)]
        for t in threads: t.start()
        for t in threads: t.join()

        with self._index_lock:
            return ThumbnailIndexSnapshot(
                thumbnails=dict(self._thumbnails),
                mappings=dict(self._mappings),
                generated=self._generated,
                deduplicated=self._deduplicated,
            )


if __name__ == "__main__":
    import hashlib
    import time

    # 50 source files; only 10 unique contents (each content appears 5 times).
    creator = BulkThumbnailCreator(worker_count=4)
    for kind in range(10):
        for copy in range(5):
            creator.enqueue(ThumbnailRequest(
                source_id=f"photo_{kind:02}_{copy}.jpg",
                content=f"pretend pixels kind {kind}".encode(),
            ))

    def sha1_prefix(content: bytes) -> int:
        h = hashlib.sha1(content).hexdigest()
        return int(h[:15], 16)  # truncate for readability

    def make_thumbnail(hash_: int, content: bytes) -> str:
        # Simulate a non-trivial resize. Sleep briefly to make the dedup
        # savings visible.
        time.sleep(0.01)
        return f"thumb_{hash_:016x}_size_{len(content)}"

    start = time.monotonic()
    snap = creator.run_with(sha1_prefix, make_thumbnail)
    elapsed = time.monotonic() - start

    print(f"{len(snap.mappings)} requests processed in {elapsed:.2f}s")
    print(f"  unique thumbnails: {len(snap.thumbnails)} (expected 10)")
    print(f"  generated: {snap.generated}, deduplicated: {snap.deduplicated}")
    print(f"  thumbnail func called {snap.generated} times (vs {len(snap.mappings)} requests)")
