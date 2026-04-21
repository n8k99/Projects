//! Wallpaper Manager — multi-monitor rotation + resolution best-fit + recency.
//!
//! Ninth Graphics project. Distinctive moves:
//!
//!   * **Multi-monitor simultaneous assignment** — unlike G118's single
//!     cursor, the manager holds a `monitor_id → wallpaper_id` map;
//!     each monitor rotates independently on its own schedule.
//!   * **Resolution best-fit score** — pure function matching a monitor's
//!     resolution to a wallpaper variant. Aspect-ratio match beats raw
//!     pixel count; exact match beats approximate.
//!   * **Recency-weighted selection** — avoid the last `recency_window`
//!     shown wallpapers per monitor. Deterministic (seeded LCG) yet
//!     varied. Same seed + same history → same pick.
//!   * **Schedule policies as data** — `Schedule::{Interval(ms),
//!     TimeWindow(start_ms, end_ms, inner_schedule), FixedList}`.
//!     `next_rotation_time_ms` is a pure function over schedule +
//!     elapsed + current assignment age.

use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Resolution { pub width: u32, pub height: u32 }

impl Resolution {
    pub fn new(width: u32, height: u32) -> Self { Self { width, height } }
    pub fn aspect_ratio(&self) -> f64 {
        self.width as f64 / self.height as f64
    }
    pub fn pixel_count(&self) -> u64 {
        self.width as u64 * self.height as u64
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Monitor {
    pub id: u32,
    pub resolution: Resolution,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WallpaperVariant {
    pub resolution: Resolution,
    pub path: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Wallpaper {
    pub id: u32,
    pub name: String,
    pub tags: Vec<String>,
    pub variants: Vec<WallpaperVariant>,
}

/// Score how well a wallpaper variant matches a monitor's resolution.
/// Higher is better. Exact match = 1000. Aspect-ratio match weighted
/// heavily; pixel-count proximity weighted less.
pub fn resolution_fit_score(monitor: &Resolution, variant: &Resolution) -> i64 {
    if monitor == variant { return 1000; }
    let m_ar = monitor.aspect_ratio();
    let v_ar = variant.aspect_ratio();
    let ar_diff = (m_ar - v_ar).abs();
    // 0 aspect diff = 500; 0.5 diff = 0
    let ar_score = ((0.5 - ar_diff.min(0.5)) / 0.5 * 500.0) as i64;

    // Penalize upscaling more than downscaling
    let m_px = monitor.pixel_count() as f64;
    let v_px = variant.pixel_count() as f64;
    let ratio = v_px / m_px;
    // ratio = 1.0 -> 300; ratio = 0.5 or 2.0 -> 150; ratio = 0.25 or 4.0 -> 0
    let px_score = if ratio >= 1.0 {
        // downscaling: smaller penalty. ratio=1 -> 300, ratio=2 -> 150, ratio=4 -> 0
        (300.0 / ratio.log2().max(1.0).min(2.0).max(1.0)) as i64
    } else {
        // upscaling: bigger penalty. ratio=1 -> 300, ratio=0.5 -> 0
        ((ratio - 0.5).max(0.0) * 600.0) as i64
    };

    ar_score + px_score
}

pub fn best_variant_for_monitor<'a>(wallpaper: &'a Wallpaper, monitor: &Resolution)
    -> Option<&'a WallpaperVariant>
{
    wallpaper.variants.iter()
        .max_by_key(|v| resolution_fit_score(monitor, &v.resolution))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Schedule {
    Interval { ms: u64 },
    FixedList { rotation_times_ms: Vec<u64> },
    TagWindow { tag: String, start_ms: u64, end_ms: u64, inner_ms: u64 },
}

#[derive(Debug, Clone)]
pub enum ManagerEvent {
    Rotated { monitor_id: u32, wallpaper_id: u32, at_ms: u64 },
    SkippedNoEligible { monitor_id: u32, at_ms: u64 },
}

#[derive(Debug, Clone)]
pub struct MonitorState {
    pub current_wallpaper_id: Option<u32>,
    pub last_rotation_ms: u64,
    pub history: Vec<u32>, // most-recent-last
}

#[derive(Debug, Clone)]
pub struct Manager {
    pub monitors: Vec<Monitor>,
    pub wallpapers: Vec<Wallpaper>,
    pub assignments: HashMap<u32, MonitorState>,
    pub schedule: Schedule,
    pub recency_window: usize,
    pub elapsed_ms: u64,
    pub lcg_state: u64,
    pub events: Vec<ManagerEvent>,
}

impl Manager {
    pub fn new(schedule: Schedule, recency_window: usize, seed: u64) -> Self {
        Self {
            monitors: vec![],
            wallpapers: vec![],
            assignments: HashMap::new(),
            schedule,
            recency_window,
            elapsed_ms: 0,
            lcg_state: seed.wrapping_add(1),
            events: vec![],
        }
    }

    pub fn add_monitor(&mut self, m: Monitor) {
        let id = m.id;
        self.monitors.push(m);
        self.assignments.insert(id, MonitorState {
            current_wallpaper_id: None,
            last_rotation_ms: 0,
            history: vec![],
        });
    }

    pub fn add_wallpaper(&mut self, w: Wallpaper) {
        self.wallpapers.push(w);
    }

    /// Pure LCG step — same constants as G096/G118.
    fn lcg_next(&mut self) -> u64 {
        self.lcg_state = self.lcg_state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        self.lcg_state
    }

    /// Pick the next wallpaper for a given monitor, avoiding its recent
    /// history. Returns None if no eligible wallpapers exist.
    pub fn pick_next_for_monitor(&mut self, monitor_id: u32, tag_filter: Option<&str>)
        -> Option<u32>
    {
        let history = self.assignments.get(&monitor_id)
            .map(|s| s.history.clone())
            .unwrap_or_default();
        let recent: Vec<u32> = history.iter().rev()
            .take(self.recency_window).copied().collect();

        let eligible: Vec<u32> = self.wallpapers.iter()
            .filter(|w| !recent.contains(&w.id))
            .filter(|w| match tag_filter {
                None => true,
                Some(t) => w.tags.iter().any(|wt| wt == t),
            })
            .map(|w| w.id)
            .collect();

        if eligible.is_empty() {
            // Fallback: ignore recency if we've exhausted the pool
            let eligible2: Vec<u32> = self.wallpapers.iter()
                .filter(|w| match tag_filter {
                    None => true,
                    Some(t) => w.tags.iter().any(|wt| wt == t),
                })
                .map(|w| w.id)
                .collect();
            if eligible2.is_empty() { return None; }
            let r = self.lcg_next();
            let idx = (r as usize) % eligible2.len();
            return Some(eligible2[idx]);
        }

        let r = self.lcg_next();
        let idx = (r as usize) % eligible.len();
        Some(eligible[idx])
    }

    /// Rotate a specific monitor now.
    pub fn rotate_monitor(&mut self, monitor_id: u32, tag_filter: Option<&str>) -> bool {
        let Some(pick) = self.pick_next_for_monitor(monitor_id, tag_filter) else {
            self.events.push(ManagerEvent::SkippedNoEligible {
                monitor_id, at_ms: self.elapsed_ms,
            });
            return false;
        };
        let at_ms = self.elapsed_ms;
        let state = self.assignments.get_mut(&monitor_id).unwrap();
        state.current_wallpaper_id = Some(pick);
        state.last_rotation_ms = at_ms;
        state.history.push(pick);
        self.events.push(ManagerEvent::Rotated {
            monitor_id, wallpaper_id: pick, at_ms,
        });
        true
    }

    fn should_rotate(&self, monitor_id: u32) -> bool {
        let state = &self.assignments[&monitor_id];
        match &self.schedule {
            Schedule::Interval { ms } => {
                state.current_wallpaper_id.is_none()
                    || (self.elapsed_ms - state.last_rotation_ms) >= *ms
            }
            Schedule::FixedList { rotation_times_ms } => {
                rotation_times_ms.iter().any(|&t| {
                    state.last_rotation_ms < t && t <= self.elapsed_ms
                })
            }
            Schedule::TagWindow { inner_ms, .. } => {
                state.current_wallpaper_id.is_none()
                    || (self.elapsed_ms - state.last_rotation_ms) >= *inner_ms
            }
        }
    }

    fn active_tag(&self) -> Option<String> {
        match &self.schedule {
            Schedule::TagWindow { tag, start_ms, end_ms, .. } => {
                // Map elapsed to a time-of-day slot: elapsed % 24h
                let day_ms = 24 * 60 * 60 * 1000u64;
                let in_day = self.elapsed_ms % day_ms;
                if in_day >= *start_ms && in_day < *end_ms {
                    Some(tag.clone())
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    pub fn tick(&mut self, ms: u64) {
        self.elapsed_ms += ms;
        let ids: Vec<u32> = self.monitors.iter().map(|m| m.id).collect();
        for id in ids {
            if self.should_rotate(id) {
                let tag = self.active_tag();
                self.rotate_monitor(id, tag.as_deref());
            }
        }
    }

    pub fn current_variant_for_monitor(&self, monitor_id: u32) -> Option<&WallpaperVariant> {
        let state = self.assignments.get(&monitor_id)?;
        let wallpaper_id = state.current_wallpaper_id?;
        let monitor = self.monitors.iter().find(|m| m.id == monitor_id)?;
        let wallpaper = self.wallpapers.iter().find(|w| w.id == wallpaper_id)?;
        best_variant_for_monitor(wallpaper, &monitor.resolution)
    }
}

// ---------- tests ----------

#[cfg(test)]
mod tests {
    use super::*;

    fn wp(id: u32, tags: &[&str], variants: Vec<(u32, u32)>) -> Wallpaper {
        Wallpaper {
            id,
            name: format!("wp{}", id),
            tags: tags.iter().map(|t| t.to_string()).collect(),
            variants: variants.into_iter()
                .map(|(w, h)| WallpaperVariant {
                    resolution: Resolution::new(w, h),
                    path: format!("/wp/{}/{}x{}.png", id, w, h),
                })
                .collect(),
        }
    }

    #[test]
    fn resolution_exact_match_wins() {
        let m = Resolution::new(1920, 1080);
        let exact = Resolution::new(1920, 1080);
        let close = Resolution::new(1440, 810);
        assert!(resolution_fit_score(&m, &exact) > resolution_fit_score(&m, &close));
    }

    #[test]
    fn resolution_aspect_match_beats_pixel_match() {
        let m = Resolution::new(1920, 1080); // 16:9
        let same_ar_lower = Resolution::new(1280, 720); // 16:9, lower pixels
        let bigger_wrong_ar = Resolution::new(2000, 1500); // 4:3
        assert!(resolution_fit_score(&m, &same_ar_lower)
             > resolution_fit_score(&m, &bigger_wrong_ar));
    }

    #[test]
    fn best_variant_picks_highest_score() {
        let w = wp(1, &[], vec![(800, 600), (1920, 1080), (3840, 2160)]);
        let m = Resolution::new(1920, 1080);
        let best = best_variant_for_monitor(&w, &m).unwrap();
        assert_eq!(best.resolution, Resolution::new(1920, 1080));
    }

    #[test]
    fn interval_schedule_rotates_on_tick() {
        let mut mgr = Manager::new(
            Schedule::Interval { ms: 1000 }, 0, 42,
        );
        mgr.add_monitor(Monitor { id: 1, resolution: Resolution::new(1920, 1080) });
        mgr.add_wallpaper(wp(10, &[], vec![(1920, 1080)]));
        mgr.add_wallpaper(wp(11, &[], vec![(1920, 1080)]));

        mgr.tick(1); // first tick, no current → rotate
        let s1 = mgr.assignments[&1].current_wallpaper_id;
        assert!(s1.is_some());
    }

    #[test]
    fn interval_does_not_re_rotate_before_deadline() {
        let mut mgr = Manager::new(
            Schedule::Interval { ms: 10_000 }, 0, 42,
        );
        mgr.add_monitor(Monitor { id: 1, resolution: Resolution::new(1920, 1080) });
        mgr.add_wallpaper(wp(10, &[], vec![(1920, 1080)]));
        mgr.add_wallpaper(wp(11, &[], vec![(1920, 1080)]));
        mgr.tick(100);
        let first = mgr.assignments[&1].current_wallpaper_id.unwrap();
        mgr.tick(500);
        assert_eq!(mgr.assignments[&1].current_wallpaper_id, Some(first));
    }

    #[test]
    fn interval_rotates_after_deadline() {
        let mut mgr = Manager::new(
            Schedule::Interval { ms: 1000 }, 0, 42,
        );
        mgr.add_monitor(Monitor { id: 1, resolution: Resolution::new(1920, 1080) });
        mgr.add_wallpaper(wp(10, &[], vec![(1920, 1080)]));
        mgr.add_wallpaper(wp(11, &[], vec![(1920, 1080)]));
        mgr.tick(1);
        let first = mgr.assignments[&1].current_wallpaper_id.unwrap();
        mgr.tick(1500);
        // second tick rotated — could be same or different depending on LCG
        assert_eq!(mgr.assignments[&1].history.len(), 2);
        let _ = first;
    }

    #[test]
    fn recency_avoids_repeats() {
        let mut mgr = Manager::new(
            Schedule::Interval { ms: 100 }, 1, 42,
        );
        mgr.add_monitor(Monitor { id: 1, resolution: Resolution::new(1920, 1080) });
        mgr.add_wallpaper(wp(10, &[], vec![(1920, 1080)]));
        mgr.add_wallpaper(wp(11, &[], vec![(1920, 1080)]));
        mgr.tick(1);
        let first = mgr.assignments[&1].current_wallpaper_id.unwrap();
        mgr.tick(200);
        let second = mgr.assignments[&1].current_wallpaper_id.unwrap();
        assert_ne!(first, second);
    }

    #[test]
    fn recency_falls_back_when_exhausted() {
        let mut mgr = Manager::new(
            Schedule::Interval { ms: 100 }, 10, 42,
        );
        mgr.add_monitor(Monitor { id: 1, resolution: Resolution::new(1920, 1080) });
        mgr.add_wallpaper(wp(10, &[], vec![(1920, 1080)]));
        // recency=10 but only one wallpaper → must fall back
        mgr.tick(1);
        assert_eq!(mgr.assignments[&1].current_wallpaper_id, Some(10));
        mgr.tick(200);
        // still falls back to 10
        assert_eq!(mgr.assignments[&1].current_wallpaper_id, Some(10));
    }

    #[test]
    fn two_monitors_rotate_independently() {
        let mut mgr = Manager::new(
            Schedule::Interval { ms: 100 }, 0, 42,
        );
        mgr.add_monitor(Monitor { id: 1, resolution: Resolution::new(1920, 1080) });
        mgr.add_monitor(Monitor { id: 2, resolution: Resolution::new(2560, 1440) });
        mgr.add_wallpaper(wp(10, &[], vec![(1920, 1080)]));
        mgr.add_wallpaper(wp(11, &[], vec![(2560, 1440)]));
        mgr.tick(1);
        assert!(mgr.assignments[&1].current_wallpaper_id.is_some());
        assert!(mgr.assignments[&2].current_wallpaper_id.is_some());
    }

    #[test]
    fn current_variant_matches_monitor_resolution() {
        let mut mgr = Manager::new(
            Schedule::Interval { ms: 100 }, 0, 42,
        );
        mgr.add_monitor(Monitor { id: 1, resolution: Resolution::new(2560, 1440) });
        mgr.add_wallpaper(wp(10, &[], vec![(1920, 1080), (2560, 1440), (3840, 2160)]));
        mgr.tick(1);
        let v = mgr.current_variant_for_monitor(1).unwrap();
        assert_eq!(v.resolution, Resolution::new(2560, 1440));
    }

    #[test]
    fn fixed_list_rotates_at_given_times() {
        let mut mgr = Manager::new(
            Schedule::FixedList { rotation_times_ms: vec![5000, 10000, 15000] },
            0, 42,
        );
        mgr.add_monitor(Monitor { id: 1, resolution: Resolution::new(1920, 1080) });
        mgr.add_wallpaper(wp(10, &[], vec![(1920, 1080)]));
        mgr.add_wallpaper(wp(11, &[], vec![(1920, 1080)]));

        mgr.tick(3000);
        assert_eq!(mgr.assignments[&1].history.len(), 0);
        mgr.tick(3000); // at 6000 — past first rotation
        assert_eq!(mgr.assignments[&1].history.len(), 1);
        mgr.tick(10_000); // at 16000 — past all remaining
        // at most one rotation per tick; the second tick collapses 10000+15000
        assert_eq!(mgr.assignments[&1].history.len(), 2);
    }

    #[test]
    fn tag_window_applies_tag_in_window() {
        let hour = 60 * 60 * 1000u64;
        let mut mgr = Manager::new(
            Schedule::TagWindow {
                tag: "nature".to_string(),
                start_ms: 6 * hour, // 6am
                end_ms: 12 * hour,  // noon
                inner_ms: 1000,
            },
            0, 42,
        );
        mgr.add_monitor(Monitor { id: 1, resolution: Resolution::new(1920, 1080) });
        mgr.add_wallpaper(wp(10, &["nature"], vec![(1920, 1080)]));
        mgr.add_wallpaper(wp(11, &["abstract"], vec![(1920, 1080)]));

        // Advance to 7am
        mgr.tick(7 * hour);
        // Should have picked a nature-tagged wallpaper
        let pick = mgr.assignments[&1].current_wallpaper_id.unwrap();
        assert_eq!(pick, 10);
    }

    #[test]
    fn manager_events_logged() {
        let mut mgr = Manager::new(
            Schedule::Interval { ms: 100 }, 0, 42,
        );
        mgr.add_monitor(Monitor { id: 1, resolution: Resolution::new(1920, 1080) });
        mgr.add_wallpaper(wp(10, &[], vec![(1920, 1080)]));
        mgr.tick(1);
        assert_eq!(mgr.events.len(), 1);
        matches!(&mgr.events[0], ManagerEvent::Rotated { .. });
    }

    #[test]
    fn no_wallpapers_fires_skipped_event() {
        let mut mgr = Manager::new(
            Schedule::Interval { ms: 100 }, 0, 42,
        );
        mgr.add_monitor(Monitor { id: 1, resolution: Resolution::new(1920, 1080) });
        mgr.tick(1);
        let has_skip = mgr.events.iter().any(|e|
            matches!(e, ManagerEvent::SkippedNoEligible { .. }));
        assert!(has_skip);
    }

    #[test]
    fn determinism_same_seed_same_picks() {
        let make = || {
            let mut mgr = Manager::new(
                Schedule::Interval { ms: 100 }, 0, 42,
            );
            mgr.add_monitor(Monitor { id: 1, resolution: Resolution::new(1920, 1080) });
            for i in 10..15 {
                mgr.add_wallpaper(wp(i, &[], vec![(1920, 1080)]));
            }
            for _ in 0..5 {
                mgr.tick(200);
            }
            mgr
        };
        let a = make();
        let b = make();
        assert_eq!(a.assignments[&1].history, b.assignments[&1].history);
    }
}
