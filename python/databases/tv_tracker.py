"""TV Show Tracker — hierarchical show/season/episode state + next-up.

Library = shows with episodes. Watchlist = set of watched EpisodeKeys.
Progress roll-up from episode count. Next-up = first unwatched in
(season, episode) order. Airing-since query filters by aired_ms.
"""

from __future__ import annotations
from dataclasses import dataclass, field


@dataclass(frozen=True)
class EpisodeKey:
    show_id: int
    season: int
    episode: int


@dataclass
class Episode:
    show_id: int
    season: int
    episode: int
    title: str
    duration_min: int
    aired_ms: int

    def key(self) -> EpisodeKey:
        return EpisodeKey(self.show_id, self.season, self.episode)


@dataclass
class Show:
    id: int
    title: str
    episodes: list[Episode] = field(default_factory=list)

    def total_episodes(self) -> int:
        return len(self.episodes)

    def sorted_episodes(self) -> list[Episode]:
        return sorted(self.episodes, key=lambda e: (e.season, e.episode))

    def total_runtime_min(self) -> int:
        return sum(e.duration_min for e in self.episodes)

    def seasons(self) -> list[int]:
        return sorted({e.season for e in self.episodes})


@dataclass
class Library:
    shows: dict[int, Show] = field(default_factory=dict)

    def add_show(self, show: Show) -> None:
        self.shows[show.id] = show

    def add_episode(self, episode: Episode) -> None:
        show = self.shows.get(episode.show_id)
        if show is None:
            raise ValueError(f"unknown show: {episode.show_id}")
        key = episode.key()
        show.episodes = [e for e in show.episodes if e.key() != key]
        show.episodes.append(episode)

    def get(self, show_id: int) -> Show | None:
        return self.shows.get(show_id)

    def airing_since(self, since_ms: int) -> list[Episode]:
        out = [e for show in self.shows.values() for e in show.episodes
               if e.aired_ms >= since_ms]
        out.sort(key=lambda e: (e.aired_ms, e.show_id, e.season, e.episode))
        return out


@dataclass
class Watchlist:
    watched: set[EpisodeKey] = field(default_factory=set)

    def mark_watched(self, show_id: int, season: int, episode: int) -> None:
        self.watched.add(EpisodeKey(show_id, season, episode))

    def mark_unwatched(self, show_id: int, season: int, episode: int) -> None:
        self.watched.discard(EpisodeKey(show_id, season, episode))

    def is_watched(self, show_id: int, season: int, episode: int) -> bool:
        return EpisodeKey(show_id, season, episode) in self.watched

    def mark_show_watched(self, show: Show) -> None:
        for e in show.episodes:
            self.watched.add(e.key())

    def mark_season_watched(self, show: Show, season: int) -> None:
        for e in show.episodes:
            if e.season == season:
                self.watched.add(e.key())

    def watched_count(self, show_id: int) -> int:
        return sum(1 for k in self.watched if k.show_id == show_id)


@dataclass
class Progress:
    show_id: int
    watched: int
    total: int
    percent: int


def show_progress(show: Show, watchlist: Watchlist) -> Progress:
    total = show.total_episodes()
    watched = watchlist.watched_count(show.id)
    percent = 0 if total == 0 else watched * 100 // total
    return Progress(show.id, watched, total, percent)


def next_up(show: Show, watchlist: Watchlist) -> Episode | None:
    for e in show.sorted_episodes():
        if not watchlist.is_watched(e.show_id, e.season, e.episode):
            return e
    return None


def currently_watching(library: Library, watchlist: Watchlist) -> list[Show]:
    out = []
    for show in library.shows.values():
        w = watchlist.watched_count(show.id)
        if w > 0 and w < show.total_episodes():
            out.append(show)
    out.sort(key=lambda s: s.id)
    return out


def demo() -> None:
    lib = Library()
    show = Show(1, "Breaking Bad")
    for season in range(1, 3):
        for episode in range(1, 4):
            show.episodes.append(Episode(
                show_id=1, season=season, episode=episode,
                title=f"S{season}E{episode}",
                duration_min=47,
                aired_ms=1_000_000 + season * 86_400_000 * 7 + episode * 86_400_000,
            ))
    lib.add_show(show)

    wl = Watchlist()
    wl.mark_watched(1, 1, 1)
    wl.mark_watched(1, 1, 2)

    p = show_progress(show, wl)
    print(f"progress: {p.watched}/{p.total} = {p.percent}%")

    n = next_up(show, wl)
    print(f"next up: S{n.season}E{n.episode} — {n.title}")

    cw = currently_watching(lib, wl)
    print(f"currently watching: {[s.title for s in cw]}")


if __name__ == "__main__":
    demo()
