"""Mp3 Player — playlist navigation with shuffle/repeat + position tracking.

Playlist (ordered track IDs) + queue_order (shuffle-aware) + queue_position.
Repeat: Off (end) / One (same track) / All (wrap). Shuffle: seeded Fisher-Yates.
History stack enables "prev" after first 3s rule.
"""

from __future__ import annotations
from dataclasses import dataclass, field
from enum import Enum


@dataclass
class Track:
    id: int
    title: str
    artist: str
    duration_ms: int


@dataclass
class Library:
    tracks: dict[int, Track] = field(default_factory=dict)

    def add(self, t: Track) -> None: self.tracks[t.id] = t
    def get(self, id: int) -> Track | None: return self.tracks.get(id)


@dataclass
class Playlist:
    id: int
    title: str
    track_ids: list[int] = field(default_factory=list)


class RepeatMode(Enum):
    OFF = "off"
    ONE = "one"
    ALL = "all"


class PlayerState(Enum):
    IDLE = "idle"
    PLAYING = "playing"
    PAUSED = "paused"
    STOPPED = "stopped"


class PlayEventKind(Enum):
    STATE_CHANGED = "state_changed"
    TRACK_CHANGED = "track_changed"
    REPEAT_CHANGED = "repeat_changed"
    SHUFFLE_CHANGED = "shuffle_changed"


@dataclass
class PlayEvent:
    kind: PlayEventKind
    detail: str = ""


class _NextAction:
    __slots__ = ("kind", "pos")
    def __init__(self, kind: str, pos: int = 0):
        self.kind = kind  # "advance" | "repeat" | "end"
        self.pos = pos


@dataclass
class Player:
    playlist: Playlist | None = None
    queue_order: list[int] = field(default_factory=list)
    queue_position: int = 0
    position_ms: int = 0
    state: PlayerState = PlayerState.IDLE
    repeat: RepeatMode = RepeatMode.OFF
    shuffle: bool = False
    shuffle_seed: int = 42
    history: list[int] = field(default_factory=list)
    events: list[PlayEvent] = field(default_factory=list)

    def load_playlist(self, playlist: Playlist) -> None:
        self.queue_order = list(playlist.track_ids)
        self.playlist = playlist
        self.queue_position = 0
        self.position_ms = 0
        if self.shuffle:
            self._apply_shuffle()
        self._set_state(PlayerState.STOPPED)

    def play(self) -> None:
        if self.queue_order:
            self._set_state(PlayerState.PLAYING)

    def pause(self) -> None:
        if self.state == PlayerState.PLAYING:
            self._set_state(PlayerState.PAUSED)

    def stop(self) -> None:
        self._set_state(PlayerState.STOPPED)
        self.position_ms = 0

    def current_track_id(self) -> int | None:
        if 0 <= self.queue_position < len(self.queue_order):
            return self.queue_order[self.queue_position]
        return None

    def tick(self, ms: int, library: Library) -> None:
        if self.state != PlayerState.PLAYING or not self.queue_order:
            return
        cur_id = self.current_track_id()
        if cur_id is None: return
        track = library.get(cur_id)
        if track is None: return
        remaining = max(0, track.duration_ms - self.position_ms)
        if ms >= remaining:
            self.history.append(cur_id)
            self.position_ms = 0
            action = self._compute_next()
            if action.kind == "advance":
                self.queue_position = action.pos
                self._emit_track_changed()
            elif action.kind == "repeat":
                self._emit_track_changed()
            else:
                self._set_state(PlayerState.STOPPED)
        else:
            self.position_ms += ms

    def next(self, library: Library) -> bool:
        cur_id = self.current_track_id()
        if cur_id is None: return False
        self.history.append(cur_id)
        self.position_ms = 0
        action = self._compute_next()
        if action.kind == "advance":
            self.queue_position = action.pos
            self._emit_track_changed()
            return True
        if action.kind == "repeat":
            self._emit_track_changed()
            return True
        self._set_state(PlayerState.STOPPED)
        return False

    def prev(self, library: Library) -> bool:
        if self.position_ms > 3000:
            self.position_ms = 0
            return True
        if self.history:
            prev_id = self.history.pop()
            if prev_id in self.queue_order:
                self.queue_position = self.queue_order.index(prev_id)
                self.position_ms = 0
                self._emit_track_changed()
                return True
        self.position_ms = 0
        return False

    def set_repeat(self, mode: RepeatMode) -> None:
        if self.repeat != mode:
            self.repeat = mode
            self.events.append(PlayEvent(PlayEventKind.REPEAT_CHANGED, mode.value))

    def set_shuffle(self, on: bool) -> None:
        if self.shuffle == on: return
        self.shuffle = on
        self.events.append(PlayEvent(PlayEventKind.SHUFFLE_CHANGED,
                                      "on" if on else "off"))
        if on:
            self._apply_shuffle()
        else:
            self._unshuffle()

    def _set_state(self, state: PlayerState) -> None:
        if self.state != state:
            old = self.state
            self.state = state
            self.events.append(PlayEvent(
                PlayEventKind.STATE_CHANGED,
                f"{old.value} -> {state.value}"))

    def _emit_track_changed(self) -> None:
        id = self.current_track_id()
        if id is not None:
            self.events.append(PlayEvent(PlayEventKind.TRACK_CHANGED, f"track {id}"))

    def _compute_next(self) -> _NextAction:
        if self.repeat == RepeatMode.ONE:
            return _NextAction("repeat")
        if self.repeat == RepeatMode.ALL:
            return _NextAction("advance", (self.queue_position + 1) % len(self.queue_order))
        if self.queue_position + 1 < len(self.queue_order):
            return _NextAction("advance", self.queue_position + 1)
        return _NextAction("end")

    def _apply_shuffle(self) -> None:
        if self.playlist is None: return
        self.queue_order = list(self.playlist.track_ids)
        # Fisher-Yates with same LCG as G096.
        state = (self.shuffle_seed + 1) & 0xFFFFFFFFFFFFFFFF
        n = len(self.queue_order)
        for i in range(n - 1, 0, -1):
            state = (state * 6364136223846793005 + 1442695040888963407) & 0xFFFFFFFFFFFFFFFF
            r = (state >> 33) & 0xFFFFFFFF
            j = r % (i + 1)
            self.queue_order[i], self.queue_order[j] = (
                self.queue_order[j], self.queue_order[i])
        cur_id = self.current_track_id()
        if cur_id is not None and cur_id in self.queue_order:
            pos = self.queue_order.index(cur_id)
            self.queue_order[0], self.queue_order[pos] = (
                self.queue_order[pos], self.queue_order[0])
            self.queue_position = 0

    def _unshuffle(self) -> None:
        if self.playlist is None: return
        cur_id = self.current_track_id()
        self.queue_order = list(self.playlist.track_ids)
        if cur_id is not None and cur_id in self.queue_order:
            self.queue_position = self.queue_order.index(cur_id)


def demo() -> None:
    lib = Library()
    for (id, title, dur) in [(1, "Alpha", 120_000), (2, "Beta", 180_000),
                              (3, "Gamma", 150_000), (4, "Delta", 200_000)]:
        lib.add(Track(id, title, "Various", dur))

    p = Player()
    p.load_playlist(Playlist(1, "Mix", [1, 2, 3, 4]))
    p.play()

    print(f"start: track={p.current_track_id()} state={p.state.value}")
    p.tick(125_000, lib)
    print(f"after 125s: track={p.current_track_id()} pos={p.position_ms}ms")

    p.set_repeat(RepeatMode.ALL)
    p.queue_position = 3
    p.position_ms = 0
    p.tick(210_000, lib)
    print(f"after wrap: track={p.current_track_id()} state={p.state.value}")

    p.set_shuffle(True)
    print(f"shuffle order: {p.queue_order}")


if __name__ == "__main__":
    demo()
