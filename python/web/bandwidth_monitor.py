"""Bandwidth Monitor — time-series rate calculation over samples.

First Rosetta Stone project with time series as the primary data structure.
Raw observations are monotonic byte counts; the derived signal is the rate —
bytes per second over a rolling window. Average rate smooths noise; peak
rate preserves spikes that averaging hides. Counter wraparound is handled
by skipping intervals where the count decreases rather than reporting a
nonsense negative rate.
"""

from __future__ import annotations
from collections import deque
from dataclasses import dataclass


@dataclass(frozen=True)
class Sample:
    timestamp_ms: int
    bytes_total: int


class Monitor:
    def __init__(self, capacity: int) -> None:
        if capacity < 2:
            raise ValueError("need at least 2 samples to compute a rate")
        self._samples: deque[Sample] = deque(maxlen=capacity)

    def record(self, timestamp_ms: int, bytes_total: int) -> None:
        self._samples.append(Sample(timestamp_ms, bytes_total))

    def samples(self) -> list[Sample]:
        return list(self._samples)

    def sample_count(self) -> int:
        return len(self._samples)

    def latest(self) -> Sample | None:
        return self._samples[-1] if self._samples else None

    def total_bytes(self) -> int:
        total = 0
        prev: Sample | None = None
        for s in self._samples:
            if prev is not None and s.bytes_total >= prev.bytes_total:
                total += s.bytes_total - prev.bytes_total
            # wraparound: skip this interval
            prev = s
        return total

    def instantaneous_rate(self) -> float:
        if len(self._samples) < 2:
            return 0.0
        a, b = self._samples[-2], self._samples[-1]
        if b.timestamp_ms <= a.timestamp_ms or b.bytes_total < a.bytes_total:
            return 0.0
        dt_s = (b.timestamp_ms - a.timestamp_ms) / 1000.0
        return (b.bytes_total - a.bytes_total) / dt_s

    def average_rate(self, window_ms: int) -> float:
        if not self._samples:
            return 0.0
        latest = self._samples[-1]
        min_ts = max(0, latest.timestamp_ms - window_ms)
        earliest: Sample | None = None
        for s in reversed(self._samples):
            if s.timestamp_ms >= min_ts:
                earliest = s
            else:
                break
        if earliest is None or earliest is latest:
            return 0.0
        if latest.bytes_total < earliest.bytes_total:
            return 0.0
        dt_s = (latest.timestamp_ms - earliest.timestamp_ms) / 1000.0
        if dt_s <= 0:
            return 0.0
        return (latest.bytes_total - earliest.bytes_total) / dt_s

    def peak_rate(self, window_ms: int) -> float:
        if not self._samples:
            return 0.0
        latest = self._samples[-1]
        min_ts = max(0, latest.timestamp_ms - window_ms)
        peak = 0.0
        prev: Sample | None = None
        for s in self._samples:
            if (prev is not None
                    and s.timestamp_ms >= min_ts
                    and s.timestamp_ms > prev.timestamp_ms
                    and s.bytes_total >= prev.bytes_total):
                dt = (s.timestamp_ms - prev.timestamp_ms) / 1000.0
                rate = (s.bytes_total - prev.bytes_total) / dt
                if rate > peak:
                    peak = rate
            prev = s
        return peak


if __name__ == "__main__":
    import random
    m = Monitor(capacity=20)

    bytes_total = 0
    for i in range(15):
        # Baseline 1 KB/s with an occasional spike.
        delta = 1_000 if i != 7 else 50_000
        bytes_total += delta
        m.record(i * 1000, bytes_total)

    print(f"samples: {m.sample_count()}")
    print(f"total bytes:         {m.total_bytes():>8} B")
    print(f"instantaneous:       {m.instantaneous_rate():>8.0f} B/s")
    print(f"average (10s):       {m.average_rate(10_000):>8.0f} B/s")
    print(f"peak    (10s):       {m.peak_rate(10_000):>8.0f} B/s")
    print(f"peak    (all):       {m.peak_rate(1_000_000):>8.0f} B/s")
