"""Progress Bar for Downloads — the first concurrent producer-consumer.

One thread (the worker) advances a byte counter as it downloads chunks.
Another thread (the observer) reads the counter to render a display.
The two never block each other — Python's GIL makes primitive attribute
writes atomic, and a ``threading.Lock`` protects the compound transitions
that change multiple fields at once.

This is the first Rosetta Stone project where "who computes" and "who
displays" are deliberately distinct actors. Cancellation is cooperative:
the main thread sets a flag; the worker checks it between chunks and
exits cleanly. Completion, cancellation, and failure are STATES
(a small FSM observed from another thread), not boolean flags.
"""

from __future__ import annotations
import threading
import time
from dataclasses import dataclass
from enum import Enum


class Status(Enum):
    RUNNING = "running"
    COMPLETED = "completed"
    CANCELLED = "cancelled"
    FAILED = "failed"


@dataclass(frozen=True)
class ProgressSnapshot:
    bytes_done: int
    total_bytes: int
    status: Status
    elapsed_seconds: float

    def percent(self) -> float:
        if self.total_bytes == 0:
            return 100.0
        return (self.bytes_done / self.total_bytes) * 100.0

    def bytes_per_second(self) -> float:
        if self.elapsed_seconds <= 0.0:
            return 0.0
        return self.bytes_done / self.elapsed_seconds

    def eta_seconds(self) -> float | None:
        rate = self.bytes_per_second()
        if rate <= 0.0 or self.bytes_done >= self.total_bytes:
            return None
        return (self.total_bytes - self.bytes_done) / rate

    def render_bar(self, width: int) -> str:
        filled = min(width, round((self.percent() / 100.0) * width))
        empty = width - filled
        return f"[{'#' * filled}{'-' * empty}] {self.percent():>6.2f}%"


class ProgressTracker:
    """Thread-safe counter + status FSM, shareable across threads."""

    def __init__(self, total_bytes: int) -> None:
        self._lock = threading.Lock()
        self._bytes_done = 0
        self._total_bytes = total_bytes
        self._status = Status.RUNNING
        self._start_time = time.monotonic()

    def advance(self, delta: int) -> None:
        with self._lock:
            self._bytes_done += delta

    def cancel(self) -> None:
        with self._lock:
            # Only Running → Cancelled. Terminal states are sticky.
            if self._status is Status.RUNNING:
                self._status = Status.CANCELLED

    def complete(self) -> None:
        with self._lock:
            if self._status is Status.RUNNING:
                self._status = Status.COMPLETED

    def fail(self) -> None:
        with self._lock:
            if self._status is Status.RUNNING:
                self._status = Status.FAILED

    def is_cancelled(self) -> bool:
        with self._lock:
            return self._status is Status.CANCELLED

    def snapshot(self) -> ProgressSnapshot:
        with self._lock:
            return ProgressSnapshot(
                bytes_done=self._bytes_done,
                total_bytes=self._total_bytes,
                status=self._status,
                elapsed_seconds=time.monotonic() - self._start_time,
            )


if __name__ == "__main__":
    # Simulate a 1 MB download in 10 KB chunks on a worker thread while
    # the main thread polls the progress bar every 50 ms.
    TOTAL = 1_000_000
    CHUNK = 10_000
    tracker = ProgressTracker(TOTAL)

    def worker() -> None:
        for _ in range(TOTAL // CHUNK):
            if tracker.is_cancelled():
                return
            tracker.advance(CHUNK)
            time.sleep(0.002)
        tracker.complete()

    t = threading.Thread(target=worker, daemon=True)
    t.start()
    while t.is_alive():
        snap = tracker.snapshot()
        print(f"\r{snap.render_bar(40)}  "
              f"{snap.bytes_done:>7} / {snap.total_bytes}  "
              f"rate={snap.bytes_per_second():>8.0f} B/s", end="", flush=True)
        time.sleep(0.05)
    t.join()
    final = tracker.snapshot()
    print(f"\r{final.render_bar(40)}  status={final.status.value}  "
          f"elapsed={final.elapsed_seconds:.2f}s")
