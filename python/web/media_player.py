"""Media Player Widget — playback FSM + playlist with repeat/shuffle modes.

Three-state playback (Stopped, Paused, Playing) applied to a playlist with
repeat and shuffle modes. `play()`, `pause()`, `stop()` are context-dependent:
play starts from track 0 when Stopped, resumes when Paused, no-op when Playing.

`tick(ms)` advances playback time and emits events on boundaries. Pure
state model — no audio engine, no filesystem, no actual playback.
"""

from __future__ import annotations
from dataclasses import dataclass, field
from enum import Enum


class PlayerState(Enum):
    STOPPED = "stopped"
    PLAYING = "playing"
    PAUSED = "paused"


class RepeatMode(Enum):
    NONE = "none"
    ONE = "one"
    ALL = "all"


@dataclass(frozen=True)
class Track:
    id: int
    title: str
    artist: str
    duration_ms: int


@dataclass(frozen=True)
class Event:
    kind: str         # "track_started" | "track_ended" | "playlist_ended" | "state_changed"
    track_id: int | None = None
    new_state: PlayerState | None = None


class Player:
    def __init__(self) -> None:
        self._playlist: list[Track] = []
        self._state = PlayerState.STOPPED
        self._current_idx: int | None = None
        self._position_ms = 0
        self._volume = 1.0
        self._repeat = RepeatMode.NONE
        self._shuffle = False
        self._shuffle_order: list[int] = []
        self._rng_state = 0xDEADBEEF

    def load_playlist(self, tracks: list[Track]) -> None:
        self._playlist = list(tracks)
        self._state = PlayerState.STOPPED
        self._current_idx = None
        self._position_ms = 0
        self._rebuild_shuffle_order()

    @property
    def state(self) -> PlayerState: return self._state
    @property
    def position_ms(self) -> int: return self._position_ms
    @property
    def volume(self) -> float: return self._volume
    @property
    def repeat(self) -> RepeatMode: return self._repeat
    @property
    def shuffle(self) -> bool: return self._shuffle

    def current_track(self) -> Track | None:
        if self._current_idx is None:
            return None
        pl_idx = self._shuffle_order[self._current_idx] if self._shuffle else self._current_idx
        if 0 <= pl_idx < len(self._playlist):
            return self._playlist[pl_idx]
        return None

    def set_volume(self, v: float) -> None:
        self._volume = max(0.0, min(1.0, v))

    def set_repeat(self, mode: RepeatMode) -> None:
        self._repeat = mode

    def set_shuffle(self, on: bool) -> None:
        if on == self._shuffle:
            return
        self._shuffle = on
        self._rebuild_shuffle_order()
        if self._current_idx is not None:
            if on:
                # Find current playlist index in shuffle order.
                try:
                    self._current_idx = self._shuffle_order.index(self._current_idx)
                except ValueError:
                    pass
            else:
                # Currently shuffle-indexed; convert back to playlist index.
                self._current_idx = self._shuffle_order[self._current_idx]

    def play(self) -> list[Event]:
        if self._state is PlayerState.PLAYING:
            return []
        if self._state is PlayerState.PAUSED:
            self._state = PlayerState.PLAYING
            return [Event(kind="state_changed", new_state=self._state)]
        # Stopped
        if not self._playlist:
            return []
        self._current_idx = 0
        self._position_ms = 0
        self._state = PlayerState.PLAYING
        out = []
        t = self.current_track()
        if t is not None:
            out.append(Event(kind="track_started", track_id=t.id))
        out.append(Event(kind="state_changed", new_state=self._state))
        return out

    def pause(self) -> list[Event]:
        if self._state is PlayerState.PLAYING:
            self._state = PlayerState.PAUSED
            return [Event(kind="state_changed", new_state=self._state)]
        return []

    def stop(self) -> list[Event]:
        if self._state is PlayerState.STOPPED:
            return []
        self._state = PlayerState.STOPPED
        self._position_ms = 0
        self._current_idx = None
        return [Event(kind="state_changed", new_state=self._state)]

    def next_track(self) -> list[Event]:
        if self._current_idx is None:
            return self.play()
        if self._repeat is RepeatMode.ONE:
            self._position_ms = 0
            t = self.current_track()
            return [Event(kind="track_started", track_id=t.id)] if t else []
        nxt = self._current_idx + 1
        if nxt < len(self._playlist):
            self._current_idx = nxt
            self._position_ms = 0
            t = self.current_track()
            return [Event(kind="track_started", track_id=t.id)] if t else []
        if self._repeat is RepeatMode.ALL:
            if self._shuffle:
                self._rebuild_shuffle_order()
            self._current_idx = 0
            self._position_ms = 0
            t = self.current_track()
            return [Event(kind="track_started", track_id=t.id)] if t else []
        # No repeat — stop at end.
        self._state = PlayerState.STOPPED
        self._current_idx = None
        self._position_ms = 0
        return [Event(kind="playlist_ended"),
                Event(kind="state_changed", new_state=self._state)]

    def previous_track(self) -> list[Event]:
        if self._current_idx is None:
            return []
        if self._current_idx == 0:
            self._position_ms = 0
            t = self.current_track()
            return [Event(kind="track_started", track_id=t.id)] if t else []
        self._current_idx -= 1
        self._position_ms = 0
        t = self.current_track()
        return [Event(kind="track_started", track_id=t.id)] if t else []

    def seek_ms(self, pos: int) -> bool:
        t = self.current_track()
        if t is None or pos > t.duration_ms:
            return False
        self._position_ms = pos
        return True

    def tick(self, delta_ms: int) -> list[Event]:
        if self._state is not PlayerState.PLAYING:
            return []
        events: list[Event] = []
        remaining = delta_ms
        while remaining > 0:
            cur = self.current_track()
            if cur is None:
                break
            time_left = max(0, cur.duration_ms - self._position_ms)
            if remaining < time_left:
                self._position_ms += remaining
                remaining = 0
            else:
                remaining -= time_left
                self._position_ms = cur.duration_ms
                events.append(Event(kind="track_ended", track_id=cur.id))
                events.extend(self.next_track())
                if self._state is PlayerState.STOPPED:
                    break
        return events

    def _rebuild_shuffle_order(self) -> None:
        n = len(self._playlist)
        self._shuffle_order = list(range(n))
        if not self._shuffle or n <= 1:
            return
        for i in range(n - 1, 0, -1):
            self._rng_state = (self._rng_state * 6364136223846793005
                               + 1442695040888963407) & 0xFFFFFFFFFFFFFFFF
            j = (self._rng_state >> 32) % (i + 1)
            self._shuffle_order[i], self._shuffle_order[j] = \
                self._shuffle_order[j], self._shuffle_order[i]


if __name__ == "__main__":
    p = Player()
    p.load_playlist([
        Track(id=1, title="Intro",   artist="The Band",   duration_ms=30_000),
        Track(id=2, title="Track A", artist="The Band",   duration_ms=180_000),
        Track(id=3, title="Track B", artist="The Band",   duration_ms=210_000),
        Track(id=4, title="Outro",   artist="The Band",   duration_ms=60_000),
    ])

    print(f"initial state: {p.state.value}")
    for ev in p.play():
        print(f"  event: {ev}")
    print(f"now playing: {p.current_track().title}")

    # Advance 4 minutes: should reach track 3 (intro 30s + Track A 180s = 210s, +30s into B).
    for ev in p.tick(240_000):
        print(f"  event: {ev}")
    print(f"now playing: {p.current_track().title} at {p.position_ms/1000:.0f}s")

    p.pause()
    p.tick(10_000)  # no advancement while paused
    print(f"after pause+tick(10s), position: {p.position_ms/1000:.0f}s (unchanged)")

    p.play()
    p.set_repeat(RepeatMode.ONE)
    for _ in range(3):
        p.tick(p.current_track().duration_ms + 100)
    print(f"after 3 loops of repeat-one: {p.current_track().title}")

    p.stop()
    print(f"final state: {p.state.value}")
