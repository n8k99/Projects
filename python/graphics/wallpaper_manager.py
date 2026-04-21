"""Wallpaper Manager — multi-monitor rotation + resolution best-fit + recency.

Multi-slot assignment (N monitors, independent state each).
Resolution fit score: aspect-ratio match + pixel-proximity.
Recency-weighted selection: avoid last N shown, fall back when exhausted.
Schedule policies as data: Interval / FixedList / TagWindow.
"""

from __future__ import annotations
from dataclasses import dataclass, field
from enum import Enum
from typing import Optional, Union


@dataclass(frozen=True)
class Resolution:
    width: int
    height: int

    def aspect_ratio(self) -> float:
        return self.width / self.height

    def pixel_count(self) -> int:
        return self.width * self.height


@dataclass
class Monitor:
    id: int
    resolution: Resolution


@dataclass
class WallpaperVariant:
    resolution: Resolution
    path: str


@dataclass
class Wallpaper:
    id: int
    name: str
    tags: list[str]
    variants: list[WallpaperVariant]


def resolution_fit_score(monitor: Resolution, variant: Resolution) -> int:
    if monitor == variant:
        return 1000
    m_ar = monitor.aspect_ratio()
    v_ar = variant.aspect_ratio()
    ar_diff = abs(m_ar - v_ar)
    ar_score = int((0.5 - min(ar_diff, 0.5)) / 0.5 * 500)

    m_px = monitor.pixel_count()
    v_px = variant.pixel_count()
    ratio = v_px / m_px if m_px else 0
    if ratio >= 1.0:
        import math
        denom = max(1.0, min(2.0, max(1.0, math.log2(ratio))))
        px_score = int(300 / denom)
    else:
        px_score = int(max(0.0, (ratio - 0.5)) * 600)

    return ar_score + px_score


def best_variant_for_monitor(wallpaper: Wallpaper, monitor: Resolution) -> Optional[WallpaperVariant]:
    if not wallpaper.variants:
        return None
    return max(wallpaper.variants, key=lambda v: resolution_fit_score(monitor, v.resolution))


class ScheduleKind(Enum):
    INTERVAL = "interval"
    FIXED_LIST = "fixed_list"
    TAG_WINDOW = "tag_window"


@dataclass(frozen=True)
class Schedule:
    kind: ScheduleKind
    interval_ms: int = 0
    rotation_times_ms: tuple[int, ...] = ()
    tag: str = ""
    start_ms: int = 0
    end_ms: int = 0
    inner_ms: int = 0

    @classmethod
    def interval(cls, ms: int) -> Schedule:
        return cls(ScheduleKind.INTERVAL, interval_ms=ms)

    @classmethod
    def fixed_list(cls, times: list[int]) -> Schedule:
        return cls(ScheduleKind.FIXED_LIST, rotation_times_ms=tuple(times))

    @classmethod
    def tag_window(cls, tag: str, start_ms: int, end_ms: int, inner_ms: int) -> Schedule:
        return cls(ScheduleKind.TAG_WINDOW, tag=tag, start_ms=start_ms, end_ms=end_ms, inner_ms=inner_ms)


class EventKind(Enum):
    ROTATED = "rotated"
    SKIPPED_NO_ELIGIBLE = "skipped_no_eligible"


@dataclass
class ManagerEvent:
    kind: EventKind
    monitor_id: int
    at_ms: int
    wallpaper_id: Optional[int] = None


@dataclass
class MonitorState:
    current_wallpaper_id: Optional[int] = None
    last_rotation_ms: int = 0
    history: list[int] = field(default_factory=list)


LCG_MUL = 6364136223846793005
LCG_INC = 1442695040888963407
U64 = 0xFFFFFFFFFFFFFFFF


@dataclass
class Manager:
    schedule: Schedule
    recency_window: int
    lcg_state: int
    monitors: list[Monitor] = field(default_factory=list)
    wallpapers: list[Wallpaper] = field(default_factory=list)
    assignments: dict[int, MonitorState] = field(default_factory=dict)
    elapsed_ms: int = 0
    events: list[ManagerEvent] = field(default_factory=list)

    @classmethod
    def new(cls, schedule: Schedule, recency_window: int, seed: int) -> Manager:
        return cls(schedule=schedule, recency_window=recency_window,
                    lcg_state=(seed + 1) & U64)

    def add_monitor(self, m: Monitor) -> None:
        self.monitors.append(m)
        self.assignments[m.id] = MonitorState()

    def add_wallpaper(self, w: Wallpaper) -> None:
        self.wallpapers.append(w)

    def _lcg_next(self) -> int:
        self.lcg_state = (self.lcg_state * LCG_MUL + LCG_INC) & U64
        return self.lcg_state

    def pick_next_for_monitor(self, monitor_id: int,
                               tag_filter: Optional[str] = None) -> Optional[int]:
        state = self.assignments[monitor_id]
        recent = list(reversed(state.history))[:self.recency_window]

        def matches_tag(w: Wallpaper) -> bool:
            return tag_filter is None or tag_filter in w.tags

        eligible = [w.id for w in self.wallpapers
                     if w.id not in recent and matches_tag(w)]
        if not eligible:
            fallback = [w.id for w in self.wallpapers if matches_tag(w)]
            if not fallback:
                return None
            r = self._lcg_next()
            return fallback[r % len(fallback)]

        r = self._lcg_next()
        return eligible[r % len(eligible)]

    def rotate_monitor(self, monitor_id: int, tag_filter: Optional[str] = None) -> bool:
        pick = self.pick_next_for_monitor(monitor_id, tag_filter)
        if pick is None:
            self.events.append(ManagerEvent(
                EventKind.SKIPPED_NO_ELIGIBLE, monitor_id, self.elapsed_ms))
            return False
        state = self.assignments[monitor_id]
        state.current_wallpaper_id = pick
        state.last_rotation_ms = self.elapsed_ms
        state.history.append(pick)
        self.events.append(ManagerEvent(
            EventKind.ROTATED, monitor_id, self.elapsed_ms, wallpaper_id=pick))
        return True

    def _should_rotate(self, monitor_id: int) -> bool:
        state = self.assignments[monitor_id]
        s = self.schedule
        if s.kind == ScheduleKind.INTERVAL:
            return (state.current_wallpaper_id is None
                    or self.elapsed_ms - state.last_rotation_ms >= s.interval_ms)
        if s.kind == ScheduleKind.FIXED_LIST:
            return any(state.last_rotation_ms < t <= self.elapsed_ms
                        for t in s.rotation_times_ms)
        if s.kind == ScheduleKind.TAG_WINDOW:
            return (state.current_wallpaper_id is None
                    or self.elapsed_ms - state.last_rotation_ms >= s.inner_ms)
        return False

    def _active_tag(self) -> Optional[str]:
        s = self.schedule
        if s.kind != ScheduleKind.TAG_WINDOW:
            return None
        day_ms = 24 * 60 * 60 * 1000
        in_day = self.elapsed_ms % day_ms
        if s.start_ms <= in_day < s.end_ms:
            return s.tag
        return None

    def tick(self, ms: int) -> None:
        self.elapsed_ms += ms
        for m in self.monitors:
            if self._should_rotate(m.id):
                self.rotate_monitor(m.id, self._active_tag())

    def current_variant_for_monitor(self, monitor_id: int) -> Optional[WallpaperVariant]:
        state = self.assignments.get(monitor_id)
        if not state or state.current_wallpaper_id is None:
            return None
        monitor = next((m for m in self.monitors if m.id == monitor_id), None)
        wallpaper = next((w for w in self.wallpapers if w.id == state.current_wallpaper_id), None)
        if monitor is None or wallpaper is None:
            return None
        return best_variant_for_monitor(wallpaper, monitor.resolution)


def demo() -> None:
    mgr = Manager.new(Schedule.interval(1000), recency_window=2, seed=42)
    mgr.add_monitor(Monitor(1, Resolution(1920, 1080)))
    mgr.add_monitor(Monitor(2, Resolution(2560, 1440)))

    for i, tags in [(10, ["nature"]), (11, ["abstract"]), (12, ["nature"]),
                     (13, ["abstract"]), (14, ["nature"])]:
        mgr.add_wallpaper(Wallpaper(
            i, f"wp{i}", tags,
            [WallpaperVariant(Resolution(1920, 1080), f"/wp/{i}/1080.png"),
             WallpaperVariant(Resolution(2560, 1440), f"/wp/{i}/1440.png")]))

    mgr.tick(1)  # initial rotation
    print(f"t=1ms: m1={mgr.assignments[1].current_wallpaper_id} m2={mgr.assignments[2].current_wallpaper_id}")
    mgr.tick(1100)
    print(f"t=1101ms: m1={mgr.assignments[1].current_wallpaper_id} m2={mgr.assignments[2].current_wallpaper_id}")
    mgr.tick(1100)
    print(f"t=2201ms: m1={mgr.assignments[1].current_wallpaper_id} m2={mgr.assignments[2].current_wallpaper_id}")

    # Best-fit variant
    v = mgr.current_variant_for_monitor(2)
    print(f"m2 variant: {v.resolution.width}x{v.resolution.height}")

    # Resolution scoring
    m1080 = Resolution(1920, 1080)
    print(f"score 1920x1080 vs 1920x1080: {resolution_fit_score(m1080, Resolution(1920, 1080))}")
    print(f"score 1920x1080 vs 1280x720:  {resolution_fit_score(m1080, Resolution(1280, 720))}")
    print(f"score 1920x1080 vs 2000x1500: {resolution_fit_score(m1080, Resolution(2000, 1500))}")

    print(f"history m1: {mgr.assignments[1].history}")
    print(f"history m2: {mgr.assignments[2].history}")


if __name__ == "__main__":
    demo()
