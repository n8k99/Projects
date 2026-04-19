"""Download Manager — N concurrent workers sharing one job queue.

Fan-out: one producer (the manager) adds jobs to a shared queue; N worker
threads pull jobs off the queue and run them in parallel. Fan-in: each
worker appends its outcome to a shared result list; the manager reads the
list after all workers finish.

The queue is the rendezvous point. Workers don't know about each other —
they only know the queue. A slow worker pulls less, a fast worker pulls
more. Load balancing is emergent from the pull model, not designed.

``run_with`` takes a ``simulate`` function so tests don't need network.
The manager is a pattern; the transport is a parameter.
"""

from __future__ import annotations
import queue
import threading
from dataclasses import dataclass
from typing import Callable


@dataclass
class Download:
    url: str
    expected_bytes: int


@dataclass(frozen=True)
class DownloadResult:
    url: str
    # One of: ("completed", bytes), ("cancelled", None), ("failed", error_str)
    status: str
    bytes: int | None = None
    error: str | None = None


class DownloadManager:
    def __init__(self, worker_count: int) -> None:
        if worker_count < 1:
            raise ValueError("worker_count must be >= 1")
        self._worker_count = worker_count
        # queue.Queue is already thread-safe; we pair it with a cancel flag
        # and a results list guarded by its own lock.
        self._queue: queue.Queue[Download] = queue.Queue()
        self._results: list[DownloadResult] = []
        self._results_lock = threading.Lock()
        self._cancel = threading.Event()

    def enqueue(self, d: Download) -> None:
        self._queue.put(d)

    def cancel(self) -> None:
        self._cancel.set()

    def queued_count(self) -> int:
        return self._queue.qsize()

    def result_count(self) -> int:
        with self._results_lock:
            return len(self._results)

    def run_with(self, simulate: Callable[[Download], int]) -> list[DownloadResult]:
        """Spawn `worker_count` workers pulling from the queue until empty or
        cancelled. `simulate(download) -> bytes_downloaded` or raises on
        failure. Returns results in completion order."""
        def worker() -> None:
            while True:
                if self._cancel.is_set():
                    return
                try:
                    d = self._queue.get_nowait()
                except queue.Empty:
                    return
                if self._cancel.is_set():
                    result = DownloadResult(url=d.url, status="cancelled")
                else:
                    try:
                        bytes_done = simulate(d)
                        result = DownloadResult(url=d.url, status="completed",
                                                bytes=bytes_done)
                    except Exception as e:  # isolate per-job failure
                        result = DownloadResult(url=d.url, status="failed",
                                                error=str(e))
                with self._results_lock:
                    self._results.append(result)

        threads = [threading.Thread(target=worker, daemon=True)
                   for _ in range(self._worker_count)]
        for t in threads: t.start()
        for t in threads: t.join()

        with self._results_lock:
            return list(self._results)


if __name__ == "__main__":
    import random
    import time

    m = DownloadManager(worker_count=4)
    for i in range(12):
        m.enqueue(Download(url=f"https://example.org/file-{i}.bin",
                           expected_bytes=100_000 * (i + 1)))

    def fake_download(d: Download) -> int:
        # Pretend each download takes 30–80 ms and ~1 in 10 fails.
        time.sleep(random.uniform(0.03, 0.08))
        if random.random() < 0.1:
            raise RuntimeError("simulated network error")
        return d.expected_bytes

    started = time.monotonic()
    results = m.run_with(fake_download)
    elapsed = time.monotonic() - started

    completed = [r for r in results if r.status == "completed"]
    failed    = [r for r in results if r.status == "failed"]
    total_bytes = sum(r.bytes for r in completed)  # type: ignore[misc]

    print(f"{len(results)} jobs in {elapsed:.2f}s across 4 workers")
    print(f"  completed: {len(completed)}, failed: {len(failed)}")
    print(f"  total bytes: {total_bytes:,}")
    if failed:
        print(f"  failures: {[(r.url, r.error) for r in failed]}")
