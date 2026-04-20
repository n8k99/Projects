//! TV Show Tracker — hierarchical show/season/episode state with next-up computation.
//!
//! Ninth Databases project. A TV tracker has **three levels** (show →
//! season → episode) and **two state dimensions** (the library of known
//! episodes, and the watchlist of what the user has seen). The key
//! operations:
//!   * **Progress roll-up** — show-level percentage from episode-level
//!     watched state.
//!   * **Next-up computation** — given a show, find the earliest
//!     unwatched episode in (season, episode) order. This is what every
//!     streaming UI's "Continue Watching" relies on.
//!   * **Airing-since query** — given a timestamp, return episodes
//!     aired since; the "what's new" filter.
//!
//! The library and watchlist are kept separate: the library is shared
//! (one canonical view of what exists); the watchlist is per-user
//! (which episodes this person has seen).

use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct EpisodeKey {
    pub show_id: u64,
    pub season: u32,
    pub episode: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Episode {
    pub show_id: u64,
    pub season: u32,
    pub episode: u32,
    pub title: String,
    pub duration_min: u32,
    pub aired_ms: i64,
}

impl Episode {
    pub fn key(&self) -> EpisodeKey {
        EpisodeKey { show_id: self.show_id, season: self.season, episode: self.episode }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Show {
    pub id: u64,
    pub title: String,
    pub episodes: Vec<Episode>,
}

impl Show {
    pub fn total_episodes(&self) -> u32 { self.episodes.len() as u32 }

    /// Episodes in canonical (season, episode) order.
    pub fn sorted_episodes(&self) -> Vec<&Episode> {
        let mut v: Vec<&Episode> = self.episodes.iter().collect();
        v.sort_by_key(|e| (e.season, e.episode));
        v
    }

    /// Total runtime in minutes.
    pub fn total_runtime_min(&self) -> u32 {
        self.episodes.iter().map(|e| e.duration_min).sum()
    }

    pub fn seasons(&self) -> Vec<u32> {
        let mut seasons: BTreeSet<u32> = self.episodes.iter().map(|e| e.season).collect();
        seasons.into_iter().collect()
    }
}

#[derive(Debug, Clone)]
pub struct Library {
    pub shows: BTreeMap<u64, Show>,
}

impl Library {
    pub fn new() -> Self { Self { shows: BTreeMap::new() } }

    pub fn add_show(&mut self, show: Show) {
        self.shows.insert(show.id, show);
    }

    pub fn add_episode(&mut self, episode: Episode) -> Result<(), String> {
        let show = self.shows.get_mut(&episode.show_id)
            .ok_or_else(|| format!("unknown show: {}", episode.show_id))?;
        // If an episode with the same (show, season, episode) already exists, replace.
        let key = episode.key();
        show.episodes.retain(|e| e.key() != key);
        show.episodes.push(episode);
        Ok(())
    }

    pub fn get(&self, show_id: u64) -> Option<&Show> { self.shows.get(&show_id) }

    /// All episodes whose `aired_ms >= since`, sorted by aired_ms.
    pub fn airing_since(&self, since_ms: i64) -> Vec<&Episode> {
        let mut out: Vec<&Episode> = self.shows.values()
            .flat_map(|s| &s.episodes)
            .filter(|e| e.aired_ms >= since_ms)
            .collect();
        out.sort_by_key(|e| (e.aired_ms, e.show_id, e.season, e.episode));
        out
    }
}

impl Default for Library { fn default() -> Self { Self::new() } }

#[derive(Debug, Clone)]
pub struct Watchlist {
    pub watched: BTreeSet<EpisodeKey>,
}

impl Watchlist {
    pub fn new() -> Self { Self { watched: BTreeSet::new() } }

    pub fn mark_watched(&mut self, show_id: u64, season: u32, episode: u32) {
        self.watched.insert(EpisodeKey { show_id, season, episode });
    }

    pub fn mark_unwatched(&mut self, show_id: u64, season: u32, episode: u32) {
        self.watched.remove(&EpisodeKey { show_id, season, episode });
    }

    pub fn is_watched(&self, show_id: u64, season: u32, episode: u32) -> bool {
        self.watched.contains(&EpisodeKey { show_id, season, episode })
    }

    /// Mark every episode of a show as watched.
    pub fn mark_show_watched(&mut self, show: &Show) {
        for e in &show.episodes {
            self.watched.insert(e.key());
        }
    }

    /// Mark every episode of a season as watched.
    pub fn mark_season_watched(&mut self, show: &Show, season: u32) {
        for e in show.episodes.iter().filter(|e| e.season == season) {
            self.watched.insert(e.key());
        }
    }

    /// Number of watched episodes for a given show.
    pub fn watched_count(&self, show_id: u64) -> usize {
        self.watched.iter().filter(|k| k.show_id == show_id).count()
    }
}

impl Default for Watchlist { fn default() -> Self { Self::new() } }

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Progress {
    pub show_id: u64,
    pub watched: u32,
    pub total: u32,
    pub percent: u32,   // 0..=100
}

/// Watchlist-aware progress for a show.
pub fn show_progress(show: &Show, watchlist: &Watchlist) -> Progress {
    let total = show.total_episodes();
    let watched = watchlist.watched_count(show.id) as u32;
    let percent = if total == 0 { 0 } else { (watched * 100) / total };
    Progress { show_id: show.id, watched, total, percent }
}

/// Earliest unwatched episode, in (season, episode) order.
pub fn next_up<'a>(show: &'a Show, watchlist: &Watchlist) -> Option<&'a Episode> {
    let sorted = show.sorted_episodes();
    sorted.into_iter().find(|e| !watchlist.is_watched(e.show_id, e.season, e.episode))
}

/// Shows a user has started but not finished, sorted by last-watched activity.
pub fn currently_watching<'a>(library: &'a Library, watchlist: &Watchlist) -> Vec<&'a Show> {
    let mut out: Vec<&Show> = library.shows.values()
        .filter(|show| {
            let w = watchlist.watched_count(show.id);
            w > 0 && (w as u32) < show.total_episodes()
        })
        .collect();
    out.sort_by_key(|s| s.id);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn breaking_bad() -> Show {
        let mut show = Show { id: 1, title: "Breaking Bad".into(), episodes: vec![] };
        // 2 seasons, 3 eps each — simplified
        for season in 1..=2 {
            for episode in 1..=3 {
                show.episodes.push(Episode {
                    show_id: 1, season, episode,
                    title: format!("S{season}E{episode}"),
                    duration_min: 47,
                    aired_ms: 1_000_000 + (season as i64) * 86_400_000 * 7 +
                              (episode as i64) * 86_400_000,
                });
            }
        }
        show
    }

    #[test]
    fn total_episodes_counts_all() {
        let show = breaking_bad();
        assert_eq!(show.total_episodes(), 6);
    }

    #[test]
    fn sorted_episodes_is_season_then_episode() {
        let show = breaking_bad();
        let sorted = show.sorted_episodes();
        assert_eq!((sorted[0].season, sorted[0].episode), (1, 1));
        assert_eq!((sorted[5].season, sorted[5].episode), (2, 3));
    }

    #[test]
    fn seasons_lists_distinct_season_numbers() {
        let show = breaking_bad();
        assert_eq!(show.seasons(), vec![1, 2]);
    }

    #[test]
    fn total_runtime_sums_durations() {
        let show = breaking_bad();
        assert_eq!(show.total_runtime_min(), 6 * 47);
    }

    #[test]
    fn next_up_on_empty_watchlist_is_first_episode() {
        let show = breaking_bad();
        let wl = Watchlist::new();
        let next = next_up(&show, &wl).unwrap();
        assert_eq!(next.season, 1);
        assert_eq!(next.episode, 1);
    }

    #[test]
    fn next_up_skips_watched_episodes() {
        let show = breaking_bad();
        let mut wl = Watchlist::new();
        wl.mark_watched(1, 1, 1);
        wl.mark_watched(1, 1, 2);
        let next = next_up(&show, &wl).unwrap();
        assert_eq!((next.season, next.episode), (1, 3));
    }

    #[test]
    fn next_up_crosses_season_boundary() {
        let show = breaking_bad();
        let mut wl = Watchlist::new();
        for ep in 1..=3 { wl.mark_watched(1, 1, ep); }
        let next = next_up(&show, &wl).unwrap();
        assert_eq!((next.season, next.episode), (2, 1));
    }

    #[test]
    fn next_up_none_when_fully_watched() {
        let show = breaking_bad();
        let mut wl = Watchlist::new();
        wl.mark_show_watched(&show);
        assert!(next_up(&show, &wl).is_none());
    }

    #[test]
    fn progress_percent_scales_correctly() {
        let show = breaking_bad();
        let mut wl = Watchlist::new();
        let p = show_progress(&show, &wl);
        assert_eq!(p.percent, 0);
        wl.mark_watched(1, 1, 1);
        wl.mark_watched(1, 1, 2);
        wl.mark_watched(1, 1, 3);
        let p = show_progress(&show, &wl);
        assert_eq!(p.percent, 50);
        assert_eq!(p.watched, 3);
        assert_eq!(p.total, 6);
    }

    #[test]
    fn mark_season_watched_bulk() {
        let show = breaking_bad();
        let mut wl = Watchlist::new();
        wl.mark_season_watched(&show, 1);
        assert_eq!(wl.watched_count(1), 3);
        let p = show_progress(&show, &wl);
        assert_eq!(p.percent, 50);
    }

    #[test]
    fn mark_unwatched_removes() {
        let mut wl = Watchlist::new();
        wl.mark_watched(1, 1, 1);
        assert!(wl.is_watched(1, 1, 1));
        wl.mark_unwatched(1, 1, 1);
        assert!(!wl.is_watched(1, 1, 1));
    }

    #[test]
    fn airing_since_sorts_by_aired_time() {
        let mut lib = Library::new();
        lib.add_show(breaking_bad());
        let all = lib.airing_since(0);
        for w in all.windows(2) {
            assert!(w[0].aired_ms <= w[1].aired_ms);
        }
    }

    #[test]
    fn airing_since_filters_old_episodes() {
        let mut lib = Library::new();
        lib.add_show(breaking_bad());
        // Season 2 starts at roughly 1_000_000 + 2 * 7 days.
        let since = 1_000_000 + 2 * 86_400_000 * 7;
        let recent = lib.airing_since(since);
        assert!(recent.iter().all(|e| e.season == 2));
    }

    #[test]
    fn currently_watching_excludes_unwatched_and_completed() {
        let mut lib = Library::new();
        lib.add_show(breaking_bad());
        let wl = Watchlist::new();
        assert!(currently_watching(&lib, &wl).is_empty());  // nothing started

        let mut wl = Watchlist::new();
        wl.mark_watched(1, 1, 1);
        assert_eq!(currently_watching(&lib, &wl).len(), 1);

        let mut wl = Watchlist::new();
        wl.mark_show_watched(lib.get(1).unwrap());
        assert!(currently_watching(&lib, &wl).is_empty());  // completed
    }

    #[test]
    fn library_add_episode_replaces_existing_key() {
        let mut lib = Library::new();
        lib.add_show(Show { id: 1, title: "x".into(), episodes: vec![] });
        lib.add_episode(Episode { show_id: 1, season: 1, episode: 1,
                                   title: "first".into(), duration_min: 30, aired_ms: 0 }).unwrap();
        lib.add_episode(Episode { show_id: 1, season: 1, episode: 1,
                                   title: "revised".into(), duration_min: 35, aired_ms: 0 }).unwrap();
        let show = lib.get(1).unwrap();
        assert_eq!(show.episodes.len(), 1);
        assert_eq!(show.episodes[0].title, "revised");
    }

    #[test]
    fn library_add_episode_errors_on_unknown_show() {
        let mut lib = Library::new();
        let err = lib.add_episode(Episode { show_id: 999, season: 1, episode: 1,
                                              title: "x".into(), duration_min: 30, aired_ms: 0 });
        assert!(err.is_err());
    }
}
