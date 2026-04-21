"""Traffic Light — coordinated super-state FSM + preemption + adaptive timing.

Five-phase cycle: NSGreen → NSYellow → AllRed → EWGreen → EWYellow → AllRed.
Light colors derived from phase; invalid combinations (both green) unrepresentable.
Emergency preemption saves and restores phase.
Adaptive green extension: queued vehicles extend green up to a cap.
Pedestrian walk gated to AllRed entry; tracks Walk → Flashing → Inactive.
"""

from __future__ import annotations
from dataclasses import dataclass, field
from enum import Enum
from typing import Optional


class Phase(Enum):
    NS_GREEN = "ns_green"
    NS_YELLOW = "ns_yellow"
    ALL_RED = "all_red"
    EW_GREEN = "ew_green"
    EW_YELLOW = "ew_yellow"
    EMERGENCY_ALL_RED = "emergency_all_red"


class LightColor(Enum):
    RED = "red"
    YELLOW = "yellow"
    GREEN = "green"


class PedSignal(Enum):
    DONT_WALK = "dont_walk"
    WALK = "walk"
    FLASHING = "flashing"


class Direction(Enum):
    NS = "ns"
    EW = "ew"


def phase_ns_light(p: Phase) -> LightColor:
    if p == Phase.NS_GREEN: return LightColor.GREEN
    if p == Phase.NS_YELLOW: return LightColor.YELLOW
    return LightColor.RED


def phase_ew_light(p: Phase) -> LightColor:
    if p == Phase.EW_GREEN: return LightColor.GREEN
    if p == Phase.EW_YELLOW: return LightColor.YELLOW
    return LightColor.RED


@dataclass(frozen=True)
class Config:
    ns_green_ms: int = 20_000
    ns_yellow_ms: int = 3_000
    ew_green_ms: int = 20_000
    ew_yellow_ms: int = 3_000
    all_red_ms: int = 2_000
    max_green_extension_ms: int = 10_000
    ped_walk_ms: int = 8_000
    ped_flashing_ms: int = 5_000


class EventKind(Enum):
    PHASE_CHANGED = "phase_changed"
    GREEN_EXTENDED = "green_extended"
    PED_REQUESTED = "ped_requested"
    PED_WALK_STARTED = "ped_walk_started"
    PED_WALK_ENDED = "ped_walk_ended"
    EMERGENCY_STARTED = "emergency_started"
    EMERGENCY_ENDED = "emergency_ended"


@dataclass
class IntersectionEvent:
    kind: EventKind
    at_ms: int
    detail: str = ""


class PedState(Enum):
    INACTIVE = "inactive"
    WALK = "walk"
    FLASHING = "flashing"


@dataclass
class Intersection:
    config: Config
    phase: Phase = Phase.NS_GREEN
    phase_elapsed_ms: int = 0
    phase_duration_ms: int = 0
    cycle_side: Direction = Direction.EW
    ns_queue: int = 0
    ew_queue: int = 0
    ped_requested: bool = False
    _ped_state: PedState = PedState.INACTIVE
    _ped_elapsed_ms: int = 0
    _saved_phase: Optional[tuple[Phase, int, Direction]] = None
    total_elapsed_ms: int = 0
    events: list[IntersectionEvent] = field(default_factory=list)

    def __post_init__(self) -> None:
        self.phase_duration_ms = self._duration_for(self.phase)

    def _duration_for(self, p: Phase) -> int:
        c = self.config
        if p == Phase.NS_GREEN: return c.ns_green_ms
        if p == Phase.NS_YELLOW: return c.ns_yellow_ms
        if p == Phase.EW_GREEN: return c.ew_green_ms
        if p == Phase.EW_YELLOW: return c.ew_yellow_ms
        if p == Phase.ALL_RED: return c.all_red_ms
        if p == Phase.EMERGENCY_ALL_RED: return 10**18
        return 0

    def ns_light(self) -> LightColor: return phase_ns_light(self.phase)
    def ew_light(self) -> LightColor: return phase_ew_light(self.phase)

    def ped_signal(self) -> PedSignal:
        if self._ped_state == PedState.INACTIVE: return PedSignal.DONT_WALK
        if self._ped_state == PedState.WALK: return PedSignal.WALK
        return PedSignal.FLASHING

    def set_ns_queue(self, n: int) -> None: self.ns_queue = n
    def set_ew_queue(self, n: int) -> None: self.ew_queue = n

    def request_ped_crossing(self) -> None:
        if self._ped_state != PedState.INACTIVE: return
        if not self.ped_requested:
            self.ped_requested = True
            self.events.append(IntersectionEvent(
                EventKind.PED_REQUESTED, self.total_elapsed_ms))

    def begin_emergency(self) -> None:
        if self.phase == Phase.EMERGENCY_ALL_RED: return
        self._saved_phase = (self.phase, self.phase_elapsed_ms, self.cycle_side)
        at = self.total_elapsed_ms
        from_phase = self.phase
        self.phase = Phase.EMERGENCY_ALL_RED
        self.phase_elapsed_ms = 0
        self.phase_duration_ms = 10**18
        self._ped_state = PedState.INACTIVE
        self.events.append(IntersectionEvent(EventKind.EMERGENCY_STARTED, at))
        self.events.append(IntersectionEvent(
            EventKind.PHASE_CHANGED, at,
            f"{from_phase.value} -> emergency_all_red"))

    def end_emergency(self) -> None:
        if self.phase != Phase.EMERGENCY_ALL_RED: return
        at = self.total_elapsed_ms
        from_phase = self.phase
        if self._saved_phase is not None:
            p, elapsed, side = self._saved_phase
            self._saved_phase = None
            self.phase = p
            self.phase_elapsed_ms = elapsed
            self.phase_duration_ms = self._duration_for(p)
            self.cycle_side = side
        else:
            self.phase = Phase.ALL_RED
            self.phase_elapsed_ms = 0
            self.phase_duration_ms = self.config.all_red_ms
        self.events.append(IntersectionEvent(EventKind.EMERGENCY_ENDED, at))
        self.events.append(IntersectionEvent(
            EventKind.PHASE_CHANGED, at,
            f"{from_phase.value} -> {self.phase.value}"))

    def _consider_extension(self) -> int:
        direction: Optional[Direction]
        if self.phase == Phase.NS_GREEN: direction = Direction.NS
        elif self.phase == Phase.EW_GREEN: direction = Direction.EW
        else: direction = None
        if direction is None: return 0
        queue = self.ns_queue if direction == Direction.NS else self.ew_queue
        if queue == 0: return 0

        base = self.config.ns_green_ms if direction == Direction.NS else self.config.ew_green_ms
        already = max(0, self.phase_duration_ms - base)
        remaining_cap = max(0, self.config.max_green_extension_ms - already)
        if remaining_cap == 0: return 0
        grant = min(queue * 2000, remaining_cap)
        self.phase_duration_ms += grant
        if grant > 0:
            self.events.append(IntersectionEvent(
                EventKind.GREEN_EXTENDED, self.total_elapsed_ms,
                f"{direction.value} +{grant}ms"))
        return grant

    def _next_phase(self) -> tuple[Phase, Direction]:
        p = self.phase
        if p == Phase.NS_GREEN: return (Phase.NS_YELLOW, self.cycle_side)
        if p == Phase.NS_YELLOW: return (Phase.ALL_RED, Direction.EW)
        if p == Phase.ALL_RED:
            if self.cycle_side == Direction.EW:
                return (Phase.EW_GREEN, Direction.NS)
            return (Phase.NS_GREEN, Direction.EW)
        if p == Phase.EW_GREEN: return (Phase.EW_YELLOW, self.cycle_side)
        if p == Phase.EW_YELLOW: return (Phase.ALL_RED, Direction.NS)
        return (Phase.EMERGENCY_ALL_RED, self.cycle_side)

    def _transition_phase(self) -> bool:
        new_phase, new_side = self._next_phase()
        from_phase = self.phase
        self.phase = new_phase
        self.cycle_side = new_side
        self.phase_elapsed_ms = 0
        self.phase_duration_ms = self._duration_for(new_phase)
        self.events.append(IntersectionEvent(
            EventKind.PHASE_CHANGED, self.total_elapsed_ms,
            f"{from_phase.value} -> {new_phase.value}"))
        if new_phase == Phase.ALL_RED and self.ped_requested:
            self.ped_requested = False
            self._ped_state = PedState.WALK
            self._ped_elapsed_ms = 0
            self.events.append(IntersectionEvent(
                EventKind.PED_WALK_STARTED, self.total_elapsed_ms))
            return True
        return False

    def _advance_pedestrian(self, ms: int) -> None:
        if self._ped_state == PedState.INACTIVE: return
        self._ped_elapsed_ms += ms
        if (self._ped_state == PedState.WALK
            and self._ped_elapsed_ms >= self.config.ped_walk_ms):
            self._ped_state = PedState.FLASHING
            self._ped_elapsed_ms = 0
        elif (self._ped_state == PedState.FLASHING
              and self._ped_elapsed_ms >= self.config.ped_flashing_ms):
            self._ped_state = PedState.INACTIVE
            self._ped_elapsed_ms = 0
            self.events.append(IntersectionEvent(
                EventKind.PED_WALK_ENDED, self.total_elapsed_ms))

    def tick(self, ms: int) -> None:
        self.total_elapsed_ms += ms
        if self.phase == Phase.EMERGENCY_ALL_RED: return

        self.phase_elapsed_ms += ms
        if (self.phase in (Phase.NS_GREEN, Phase.EW_GREEN)
            and self.phase_elapsed_ms >= self.phase_duration_ms):
            self._consider_extension()

        ped_just_started_remaining: Optional[int] = None
        while (self.phase_elapsed_ms >= self.phase_duration_ms
               and self.phase != Phase.EMERGENCY_ALL_RED):
            overflow = self.phase_elapsed_ms - self.phase_duration_ms
            started = self._transition_phase()
            self.phase_elapsed_ms = min(overflow, self.phase_duration_ms)
            if started:
                ped_just_started_remaining = overflow

        if self._ped_state == PedState.INACTIVE:
            ped_advance = 0
        elif ped_just_started_remaining is not None:
            ped_advance = ped_just_started_remaining
        else:
            ped_advance = ms
        self._advance_pedestrian(ped_advance)


def demo() -> None:
    cfg = Config(
        ns_green_ms=1000, ns_yellow_ms=200, ew_green_ms=1000, ew_yellow_ms=200,
        all_red_ms=100, max_green_extension_ms=500,
        ped_walk_ms=500, ped_flashing_ms=300,
    )
    i = Intersection(cfg)
    print(f"initial: phase={i.phase.value} ns={i.ns_light().value} ew={i.ew_light().value}")

    i.tick(1000)
    print(f"after 1000ms: phase={i.phase.value}")
    i.tick(200)
    print(f"after +200ms: phase={i.phase.value}")
    i.tick(100)
    print(f"after +100ms: phase={i.phase.value}")

    # Adaptive green
    i2 = Intersection(cfg)
    i2.set_ns_queue(3)
    i2.tick(1000)
    print(f"with queue: phase={i2.phase.value} duration={i2.phase_duration_ms}")

    # Emergency
    i3 = Intersection(cfg)
    i3.tick(300)
    i3.begin_emergency()
    print(f"emergency started: phase={i3.phase.value}")
    i3.tick(10000)
    print(f"frozen: phase={i3.phase.value}")
    i3.end_emergency()
    print(f"resumed: phase={i3.phase.value} elapsed={i3.phase_elapsed_ms}")

    # Pedestrian
    i4 = Intersection(cfg)
    i4.request_ped_crossing()
    print(f"ped requested, signal={i4.ped_signal().value}")
    i4.tick(1200)
    print(f"after 1200ms: phase={i4.phase.value} ped={i4.ped_signal().value}")
    i4.tick(500)
    print(f"after +500ms: ped={i4.ped_signal().value}")
    i4.tick(300)
    print(f"after +300ms: ped={i4.ped_signal().value}")


if __name__ == "__main__":
    demo()
