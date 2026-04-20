"""Stream Player — segmented video with buffer-based adaptive bitrate.

ABR kernel every HLS/DASH client uses: pick the highest bitrate whose
download at observed bandwidth completes before the buffer drains.
States: Idle → Buffering → Playing → (Paused|Ended). Events captured
as a log for testability.
"""

from __future__ import annotations
from dataclasses import dataclass, field
from enum import Enum


class PlayerState(Enum):
    IDLE = "idle"
    BUFFERING = "buffering"
    PLAYING = "playing"
    PAUSED = "paused"
    ENDED = "ended"


@dataclass
class Segment:
    index: int
    duration_ms: int
    sizes_by_bitrate: dict[int, int] = field(default_factory=dict)

    def size_for(self, bitrate_bps: int) -> int | None:
        return self.sizes_by_bitrate.get(bitrate_bps)


@dataclass
class Stream:
    bitrates_bps: list[int]
    segments: list[Segment]
    safety_factor: float = 0.85
    target_buffer_ms: int = 6000

    def total_duration_ms(self) -> int:
        return sum(s.duration_ms for s in self.segments)

    def pick_bitrate(self, buffer_ms: int, bandwidth_bps: int,
                      segment: Segment) -> int:
        effective_bw = max(1.0, bandwidth_bps * self.safety_factor)
        raw_budget = max(buffer_ms, segment.duration_ms)
        budget_ms = min(raw_budget, self.target_buffer_ms * 2)
        chosen = self.bitrates_bps[0]
        for bitrate in self.bitrates_bps:
            bytes_ = segment.size_for(bitrate)
            if bytes_ is None:
                continue
            download_ms = (bytes_ * 8.0 / effective_bw) * 1000.0
            if download_ms <= budget_ms:
                chosen = bitrate
        return chosen


class EventKind(Enum):
    STATE_CHANGED = "state_changed"
    SEGMENT_LOADED = "segment_loaded"
    BITRATE_SWITCHED = "bitrate_switched"
    REBUFFER = "rebuffer"
    SEEKED = "seeked"


@dataclass
class PlayerEvent:
    kind: EventKind
    detail: str = ""


@dataclass
class Player:
    state: PlayerState = PlayerState.IDLE
    current_segment: int = 0
    buffer_ms: int = 0
    played_ms: int = 0
    bandwidth_bps: int = 1_000_000
    current_bitrate: int = 0
    events: list[PlayerEvent] = field(default_factory=list)
    stream: Stream | None = None

    def attach(self, stream: Stream) -> None:
        self.stream = stream
        self.current_bitrate = stream.bitrates_bps[0]
        self._set_state(PlayerState.BUFFERING)

    def set_bandwidth(self, bps: int) -> None:
        self.bandwidth_bps = bps

    def _set_state(self, state: PlayerState) -> None:
        if self.state != state:
            old = self.state
            self.state = state
            self.events.append(PlayerEvent(
                EventKind.STATE_CHANGED, f"{old.value} -> {state.value}"))

    def load_next_segment(self) -> bool:
        if self.stream is None:
            return False
        if self.current_segment >= len(self.stream.segments):
            return False
        segment = self.stream.segments[self.current_segment]
        new_bitrate = self.stream.pick_bitrate(
            self.buffer_ms, self.bandwidth_bps, segment)
        if new_bitrate != self.current_bitrate:
            self.events.append(PlayerEvent(
                EventKind.BITRATE_SWITCHED,
                f"{self.current_bitrate} -> {new_bitrate}"))
            self.current_bitrate = new_bitrate
        self.buffer_ms += segment.duration_ms
        self.current_segment += 1
        self.events.append(PlayerEvent(
            EventKind.SEGMENT_LOADED,
            f"segment {segment.index} @ {new_bitrate}bps"))
        if (self.state == PlayerState.BUFFERING
                and self.buffer_ms >= self.stream.target_buffer_ms):
            self._set_state(PlayerState.PLAYING)
        return True

    def tick(self, ms: int) -> None:
        if self.state != PlayerState.PLAYING or self.stream is None:
            return
        drained = min(ms, self.buffer_ms)
        self.buffer_ms -= drained
        self.played_ms += drained
        at_end = self.current_segment >= len(self.stream.segments)
        if self.buffer_ms == 0 and at_end:
            self._set_state(PlayerState.ENDED)
        elif self.buffer_ms == 0 and not at_end:
            self.events.append(PlayerEvent(
                EventKind.REBUFFER, f"buffer empty at {self.played_ms}ms"))
            self._set_state(PlayerState.BUFFERING)

    def pause(self) -> None:
        if self.state == PlayerState.PLAYING:
            self._set_state(PlayerState.PAUSED)

    def play(self) -> None:
        if self.state == PlayerState.PAUSED:
            self._set_state(PlayerState.PLAYING)

    def seek_to_segment(self, segment_idx: int) -> bool:
        if self.stream is None:
            return False
        if segment_idx < 0 or segment_idx >= len(self.stream.segments):
            return False
        self.current_segment = segment_idx
        self.buffer_ms = 0
        self.played_ms = sum(s.duration_ms
                              for s in self.stream.segments[:segment_idx])
        self.events.append(PlayerEvent(EventKind.SEEKED,
                                        f"segment {segment_idx}"))
        self._set_state(PlayerState.BUFFERING)
        return True


def demo() -> None:
    stream = Stream(
        bitrates_bps=[500_000, 1_000_000, 2_000_000, 4_000_000],
        segments=[
            Segment(i, 2000, {
                500_000: 125_000,
                1_000_000: 250_000,
                2_000_000: 500_000,
                4_000_000: 1_000_000,
            }) for i in range(5)
        ],
    )
    p = Player()
    p.attach(stream)
    p.set_bandwidth(10_000_000)

    for _ in range(3):
        p.load_next_segment()
    print(f"after 3 loads: state={p.state.value} "
          f"bitrate={p.current_bitrate} buffer={p.buffer_ms}ms")

    p.tick(6000)
    print(f"after drain: state={p.state.value} "
          f"played={p.played_ms}ms buffer={p.buffer_ms}ms")


if __name__ == "__main__":
    demo()
