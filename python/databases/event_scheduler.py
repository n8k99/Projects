"""Event Scheduler and Calendar — intervals, recurrence, overlap detection.

Half-open [start_ms, end_ms) intervals. Recurrence: None/Daily/Weekly/Monthly.
Overlap: a_start < b_end AND b_start < a_end. Recurrence expansion skips
efficient to avoid iterating all dates.
"""

from __future__ import annotations
from dataclasses import dataclass, field
from enum import Enum


MS_PER_DAY = 86_400_000


class Recurrence(Enum):
    NONE = "none"
    DAILY = "daily"
    WEEKLY = "weekly"
    MONTHLY = "monthly"


@dataclass
class Event:
    id: int
    title: str
    start_ms: int
    end_ms: int
    recurrence: Recurrence = Recurrence.NONE
    series_end_ms: int | None = None

    def overlaps(self, other_start: int, other_end: int) -> bool:
        return self.start_ms < other_end and other_start < self.end_ms

    def duration_ms(self) -> int:
        return self.end_ms - self.start_ms


@dataclass
class Occurrence:
    event_id: int
    title: str
    start_ms: int
    end_ms: int


def _expand(event: Event, from_ms: int, to_ms: int) -> list[Occurrence]:
    out: list[Occurrence] = []
    duration = event.duration_ms()
    cutoff = event.series_end_ms if event.series_end_ms is not None else 2**62

    if event.recurrence == Recurrence.NONE:
        if event.overlaps(from_ms, to_ms):
            out.append(Occurrence(event.id, event.title, event.start_ms, event.end_ms))
        return out

    step = {
        Recurrence.DAILY: MS_PER_DAY,
        Recurrence.WEEKLY: 7 * MS_PER_DAY,
        Recurrence.MONTHLY: 30 * MS_PER_DAY,
    }[event.recurrence]

    start = event.start_ms
    if start < from_ms:
        gap = from_ms - start
        skips = max(0, gap - duration) // step
        start += skips * step
    while start < to_ms and start <= cutoff:
        end = start + duration
        if end > from_ms:
            out.append(Occurrence(event.id, event.title, start, end))
        start += step
    return out


@dataclass
class Calendar:
    events: dict[int, Event] = field(default_factory=dict)

    def add(self, event: Event) -> None:
        self.events[event.id] = event

    def get(self, id: int) -> Event | None:
        return self.events.get(id)

    def events_between(self, from_ms: int, to_ms: int) -> list[Occurrence]:
        out: list[Occurrence] = []
        for event in self.events.values():
            out.extend(_expand(event, from_ms, to_ms))
        out.sort(key=lambda o: (o.start_ms, o.event_id))
        return out

    def find_conflicts(self, start_ms: int, end_ms: int) -> list[int]:
        out: list[int] = []
        for event in self.events.values():
            for occ in _expand(event, start_ms, end_ms):
                if occ.start_ms < end_ms and start_ms < occ.end_ms:
                    out.append(event.id)
                    break
        return sorted(out)


def demo() -> None:
    cal = Calendar()
    day = lambda n: n * MS_PER_DAY
    cal.add(Event(1, "Standup", day(10), day(10) + 1800_000,
                  Recurrence.DAILY, day(30)))
    cal.add(Event(2, "Weekly Review", day(14) + 15 * 3600_000,
                  day(14) + 16 * 3600_000, Recurrence.WEEKLY))
    cal.add(Event(3, "One-off", day(12), day(12) + 2 * 3600_000))

    print("=== events between day 10 and day 16 ===")
    for o in cal.events_between(day(10), day(16)):
        print(f"  #{o.event_id} {o.title:15} start={o.start_ms // MS_PER_DAY}d end={(o.end_ms) // MS_PER_DAY}d")

    print("\n=== conflicts for a new event on day 12 midday ===")
    conflicts = cal.find_conflicts(day(12), day(12) + 12 * 3600_000)
    print(f"  {conflicts}")


if __name__ == "__main__":
    demo()
