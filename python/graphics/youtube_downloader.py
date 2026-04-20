"""YouTube Downloader — concurrency-limited queue with resumable downloads.

No threads: bandwidth splits equally across active downloads each tick.
Resumable: bytes_done persists across failures.
Exponential backoff as data: RetryPolicy + backoff_for_attempt(n).
Format selection: MaxQuality / Lowest / BestUnder(cap).
"""

from __future__ import annotations
from collections import deque
from dataclasses import dataclass, field
from enum import Enum
from typing import Union, Optional


@dataclass(frozen=True)
class VideoFormat:
    id: str
    resolution: int
    codec: str
    container: str
    size_bytes: int
    bitrate_bps: int


@dataclass
class Video:
    id: str
    title: str
    formats: list[VideoFormat]


class FormatPolicyKind(Enum):
    MAX_QUALITY = "max_quality"
    LOWEST = "lowest"
    BEST_UNDER = "best_under"


@dataclass(frozen=True)
class FormatPolicy:
    kind: FormatPolicyKind
    cap_bytes: int = 0


def format_policy_max() -> FormatPolicy:
    return FormatPolicy(FormatPolicyKind.MAX_QUALITY)

def format_policy_lowest() -> FormatPolicy:
    return FormatPolicy(FormatPolicyKind.LOWEST)

def format_policy_best_under(cap: int) -> FormatPolicy:
    return FormatPolicy(FormatPolicyKind.BEST_UNDER, cap)


def pick_format(formats: list[VideoFormat], policy: FormatPolicy) -> Optional[VideoFormat]:
    if not formats:
        return None
    if policy.kind == FormatPolicyKind.MAX_QUALITY:
        return max(formats, key=lambda f: f.resolution)
    if policy.kind == FormatPolicyKind.LOWEST:
        return min(formats, key=lambda f: f.resolution)
    if policy.kind == FormatPolicyKind.BEST_UNDER:
        candidates = [f for f in formats if f.size_bytes <= policy.cap_bytes]
        if not candidates:
            return None
        return max(candidates, key=lambda f: f.resolution)
    raise AssertionError(f"unreachable: {policy.kind}")


@dataclass(frozen=True)
class RetryPolicy:
    max_attempts: int
    initial_ms: int
    multiplier: int

    @classmethod
    def default(cls) -> RetryPolicy:
        return cls(max_attempts=3, initial_ms=1000, multiplier=2)

    def backoff_for_attempt(self, n: int) -> int:
        if n == 0:
            return 0
        ms = self.initial_ms
        for _ in range(n - 1):
            ms *= self.multiplier
        return ms


class DownloadStateKind(Enum):
    PENDING = "pending"
    IN_FLIGHT = "in_flight"
    RETRYING = "retrying"
    SUCCEEDED = "succeeded"
    FAILED = "failed"


@dataclass
class DownloadState:
    kind: DownloadStateKind
    ready_at_ms: int = 0  # only meaningful for RETRYING


@dataclass
class Download:
    id: int
    video_id: str
    format: VideoFormat
    bytes_done: int = 0
    bytes_total: int = 0
    attempts: int = 0
    state: DownloadState = field(default_factory=lambda: DownloadState(DownloadStateKind.PENDING))


class EventKind(Enum):
    ENQUEUED = "enqueued"
    STARTED = "started"
    PROGRESS = "progress"
    COMPLETED = "completed"
    FAILED = "failed"
    RETRY_SCHEDULED = "retry_scheduled"
    ABANDONED = "abandoned"


@dataclass
class ManagerEvent:
    kind: EventKind
    id: int
    detail: str = ""


@dataclass
class Manager:
    max_concurrent: int
    bandwidth_bps: int
    retry_policy: RetryPolicy
    downloads: list[Download] = field(default_factory=list)
    pending_queue: deque[int] = field(default_factory=deque)
    elapsed_ms: int = 0
    events: list[ManagerEvent] = field(default_factory=list)
    _next_id: int = 1

    def enqueue(self, video_id: str, format: VideoFormat) -> int:
        id = self._next_id
        self._next_id += 1
        d = Download(
            id=id, video_id=video_id, format=format,
            bytes_done=0, bytes_total=format.size_bytes, attempts=0,
        )
        self.downloads.append(d)
        self.pending_queue.append(id)
        self.events.append(ManagerEvent(EventKind.ENQUEUED, id))
        return id

    def active_count(self) -> int:
        return sum(1 for d in self.downloads if d.state.kind == DownloadStateKind.IN_FLIGHT)

    def download(self, id: int) -> Optional[Download]:
        for d in self.downloads:
            if d.id == id:
                return d
        return None

    def _promote_retries(self) -> None:
        now = self.elapsed_ms
        for d in self.downloads:
            if (d.state.kind == DownloadStateKind.RETRYING
                and d.state.ready_at_ms <= now):
                d.state = DownloadState(DownloadStateKind.PENDING)
                self.pending_queue.append(d.id)

    def _fill_slots(self) -> None:
        while self.active_count() < self.max_concurrent:
            if not self.pending_queue:
                break
            id = self.pending_queue.popleft()
            d = self.download(id)
            if d is None or d.state.kind != DownloadStateKind.PENDING:
                continue
            d.attempts += 1
            d.state = DownloadState(DownloadStateKind.IN_FLIGHT)
            self.events.append(ManagerEvent(EventKind.STARTED, id, f"attempt {d.attempts}"))

    def report_failure(self, id: int) -> None:
        d = self.download(id)
        if d is None or d.state.kind != DownloadStateKind.IN_FLIGHT:
            return
        if d.attempts < self.retry_policy.max_attempts:
            backoff = self.retry_policy.backoff_for_attempt(d.attempts)
            ready = self.elapsed_ms + backoff
            d.state = DownloadState(DownloadStateKind.RETRYING, ready)
            self.events.append(ManagerEvent(EventKind.FAILED, id, f"attempt {d.attempts}"))
            self.events.append(ManagerEvent(EventKind.RETRY_SCHEDULED, id, f"ready {ready}"))
        else:
            d.state = DownloadState(DownloadStateKind.FAILED)
            self.events.append(ManagerEvent(EventKind.FAILED, id, f"attempt {d.attempts}"))
            self.events.append(ManagerEvent(EventKind.ABANDONED, id))

    def tick(self, ms: int) -> None:
        self.elapsed_ms += ms
        self._promote_retries()
        self._fill_slots()

        active = [d for d in self.downloads
                  if d.state.kind == DownloadStateKind.IN_FLIGHT]
        if not active:
            return

        per_stream_bps = self.bandwidth_bps // len(active)
        bytes_this_tick = (per_stream_bps * ms) // 8000  # bps * ms / 8000 = bytes

        for d in active:
            before = d.bytes_done
            d.bytes_done = min(d.bytes_done + bytes_this_tick, d.bytes_total)
            if d.bytes_done > before:
                self.events.append(ManagerEvent(
                    EventKind.PROGRESS, d.id,
                    f"{d.bytes_done}/{d.bytes_total}"))
            if d.bytes_done >= d.bytes_total:
                d.state = DownloadState(DownloadStateKind.SUCCEEDED)
                self.events.append(ManagerEvent(EventKind.COMPLETED, d.id))


def demo() -> None:
    formats = [
        VideoFormat("a", 480, "avc1", "mp4", 50_000_000, 500_000),
        VideoFormat("b", 720, "avc1", "mp4", 100_000_000, 1_000_000),
        VideoFormat("c", 1080, "avc1", "mp4", 200_000_000, 2_000_000),
    ]

    print("format picks:")
    print(f"  max_quality -> {pick_format(formats, format_policy_max()).id}")
    print(f"  lowest -> {pick_format(formats, format_policy_lowest()).id}")
    print(f"  under 120MB -> {pick_format(formats, format_policy_best_under(120_000_000)).id}")

    mgr = Manager(max_concurrent=2, bandwidth_bps=8_000_000,
                  retry_policy=RetryPolicy(max_attempts=3, initial_ms=1000, multiplier=2))
    mgr.enqueue("v1", formats[1])  # 100MB
    mgr.enqueue("v2", formats[0])  # 50MB
    mgr.enqueue("v3", formats[2])  # 200MB

    mgr.tick(1000)  # 1s
    print(f"after 1s: active={mgr.active_count()} pending={len(mgr.pending_queue)}")
    for i in (1, 2, 3):
        d = mgr.download(i)
        print(f"  [{i}] {d.state.kind.value} {d.bytes_done}/{d.bytes_total}")

    # simulate a failure on download 1
    mgr.report_failure(1)
    d1 = mgr.download(1)
    print(f"after failure: [1] {d1.state.kind.value} ready_at={d1.state.ready_at_ms} attempts={d1.attempts}")

    print(f"backoff for retry 1: {mgr.retry_policy.backoff_for_attempt(1)}ms")
    print(f"backoff for retry 2: {mgr.retry_policy.backoff_for_attempt(2)}ms")
    print(f"backoff for retry 3: {mgr.retry_policy.backoff_for_attempt(3)}ms")


if __name__ == "__main__":
    demo()
