"""Screen Saver — idle FSM + bouncing-line physics + dim ramp + activity reset.

Six-state FSM: Active → Dimming → Saver → Locked → Waking → Active.
Physics: each line endpoint has velocity; integration + reflection off edges.
Dim ramp: opacity interpolates 0→255 linearly over dim_ms during Dimming.
Activity events reset idle timer; wake from Dimming/Saver but not Locked.
Integer-scaled positions/velocities (sub-pixel × 100) for cross-lang parity.
"""

from __future__ import annotations
from dataclasses import dataclass, field
from enum import Enum
from typing import Optional


class SaverState(Enum):
    ACTIVE = "active"
    DIMMING = "dimming"
    SAVER = "saver"
    LOCKED = "locked"
    WAKING = "waking"


@dataclass(frozen=True)
class Config:
    idle_ms: int = 60_000
    dim_ms: int = 3_000
    auto_lock_ms: int = 30_000
    wake_ms: int = 500
    canvas_width: int = 1920
    canvas_height: int = 1080


@dataclass
class Endpoint:
    x: int
    y: int
    vx: int
    vy: int


@dataclass(frozen=True)
class SSRgb:
    r: int
    g: int
    b: int


@dataclass
class BouncingLine:
    p1: Endpoint
    p2: Endpoint
    color: SSRgb

    def pixel_endpoints(self) -> tuple[tuple[int, int], tuple[int, int]]:
        return ((self.p1.x // 100, self.p1.y // 100),
                (self.p2.x // 100, self.p2.y // 100))


class SaverEventKind(Enum):
    STATE_CHANGED = "state_changed"
    ACTIVITY_RESET_TIMER = "activity_reset_timer"
    LINE_BOUNCED = "line_bounced"


@dataclass
class SaverEvent:
    kind: SaverEventKind
    at_ms: int
    detail: str = ""


@dataclass
class ScreenSaver:
    config: Config
    state: SaverState = SaverState.ACTIVE
    lines: list[BouncingLine] = field(default_factory=list)
    elapsed_ms: int = 0
    last_activity_ms: int = 0
    state_entered_ms: int = 0
    events: list[SaverEvent] = field(default_factory=list)

    def add_line(self, line: BouncingLine) -> None:
        self.lines.append(line)

    def dim_opacity(self) -> int:
        if self.state != SaverState.DIMMING:
            return 0
        since = max(0, self.elapsed_ms - self.state_entered_ms)
        if self.config.dim_ms == 0:
            return 255
        return min(since, self.config.dim_ms) * 255 // self.config.dim_ms

    def _transition_to(self, new_state: SaverState) -> None:
        if self.state == new_state: return
        self.events.append(SaverEvent(
            SaverEventKind.STATE_CHANGED, self.elapsed_ms,
            f"{self.state.value} -> {new_state.value}"))
        self.state = new_state
        self.state_entered_ms = self.elapsed_ms

    def record_activity(self) -> None:
        self.last_activity_ms = self.elapsed_ms
        self.events.append(SaverEvent(
            SaverEventKind.ACTIVITY_RESET_TIMER, self.elapsed_ms))
        if self.state in (SaverState.DIMMING, SaverState.SAVER):
            self._transition_to(SaverState.WAKING)

    def unlock(self) -> None:
        if self.state == SaverState.LOCKED:
            self._transition_to(SaverState.WAKING)
            self.last_activity_ms = self.elapsed_ms

    def _integrate_physics(self, ms: int) -> None:
        cw = self.config.canvas_width * 100
        ch = self.config.canvas_height * 100
        for idx, line in enumerate(self.lines):
            for ep in (line.p1, line.p2):
                ep.x += ep.vx * ms // 100
                ep.y += ep.vy * ms // 100
                if ep.x < 0:
                    ep.x = -ep.x
                    ep.vx = -ep.vx
                    self.events.append(SaverEvent(
                        SaverEventKind.LINE_BOUNCED, self.elapsed_ms,
                        f"line {idx} axis x"))
                if ep.x > cw:
                    ep.x = 2 * cw - ep.x
                    ep.vx = -ep.vx
                    self.events.append(SaverEvent(
                        SaverEventKind.LINE_BOUNCED, self.elapsed_ms,
                        f"line {idx} axis x"))
                if ep.y < 0:
                    ep.y = -ep.y
                    ep.vy = -ep.vy
                    self.events.append(SaverEvent(
                        SaverEventKind.LINE_BOUNCED, self.elapsed_ms,
                        f"line {idx} axis y"))
                if ep.y > ch:
                    ep.y = 2 * ch - ep.y
                    ep.vy = -ep.vy
                    self.events.append(SaverEvent(
                        SaverEventKind.LINE_BOUNCED, self.elapsed_ms,
                        f"line {idx} axis y"))

    def tick(self, ms: int) -> None:
        self.elapsed_ms += ms
        since_state = self.elapsed_ms - self.state_entered_ms
        if self.state == SaverState.ACTIVE:
            idle = max(0, self.elapsed_ms - self.last_activity_ms)
            if idle >= self.config.idle_ms:
                self._transition_to(SaverState.DIMMING)
        elif self.state == SaverState.DIMMING:
            if since_state >= self.config.dim_ms:
                self._transition_to(SaverState.SAVER)
        elif self.state == SaverState.SAVER:
            if since_state >= self.config.auto_lock_ms:
                self._transition_to(SaverState.LOCKED)
            self._integrate_physics(ms)
        elif self.state == SaverState.WAKING:
            if since_state >= self.config.wake_ms:
                self._transition_to(SaverState.ACTIVE)
                self.last_activity_ms = self.elapsed_ms


def demo() -> None:
    cfg = Config(idle_ms=1000, dim_ms=200, auto_lock_ms=500, wake_ms=100,
                  canvas_width=100, canvas_height=100)
    s = ScreenSaver(config=cfg)
    s.add_line(BouncingLine(
        p1=Endpoint(x=95_00, y=50_00, vx=10_000, vy=0),
        p2=Endpoint(x=50_00, y=50_00, vx=0, vy=0),
        color=SSRgb(r=255, g=255, b=255),
    ))

    print(f"initial: state={s.state.value}")
    s.tick(500)
    print(f"at 500ms: state={s.state.value}")
    s.tick(600)
    print(f"at 1100ms: state={s.state.value} dim_opacity={s.dim_opacity()}")
    s.tick(100)
    print(f"at 1200ms: state={s.state.value} dim_opacity={s.dim_opacity()}")
    s.tick(100)
    print(f"at 1300ms: state={s.state.value}")

    # Advance into Saver, with physics — line at x=9500 moves right, bounces
    s.tick(10)
    print(f"at 1310ms: state={s.state.value} p1.x={s.lines[0].p1.x} p1.vx={s.lines[0].p1.vx}")

    # Keep running until auto-lock
    s.tick(600)
    print(f"at 1910ms: state={s.state.value}")

    # Activity during locked doesn't wake
    s.record_activity()
    print(f"after activity: state={s.state.value}")

    # Explicit unlock wakes
    s.unlock()
    print(f"after unlock: state={s.state.value}")
    s.tick(150)
    print(f"after 150ms in Waking: state={s.state.value}")


if __name__ == "__main__":
    demo()
