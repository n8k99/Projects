//! Media Player Widget — playback FSM + playlist with repeat/shuffle modes.
//!
//! Three-state playback (Stopped, Paused, Playing) applied to a playlist
//! with repeat and shuffle modes. `play()`, `pause()`, `stop()` are
//! context-dependent: `play()` starts from track 0 when Stopped, resumes
//! from position when Paused, and is a no-op when already Playing.
//!
//! `tick(ms)` advances playback time and emits events on boundaries
//! (track-end, playlist-end). This is a pure state model — no audio
//! engine, no filesystem, no actual playback. The FSM is the point.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayerState {
    Stopped,
    Playing,
    Paused,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RepeatMode {
    None,
    One,
    All,
}

#[derive(Debug, Clone)]
pub struct Track {
    pub id: u64,
    pub title: String,
    pub artist: String,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    TrackStarted { id: u64 },
    TrackEnded { id: u64 },
    PlaylistEnded,
    StateChanged { new: PlayerState },
}

pub struct Player {
    playlist: Vec<Track>,
    state: PlayerState,
    /// Index into the shuffle order (not the playlist) when shuffle is on;
    /// index into the playlist otherwise.
    current_idx: Option<usize>,
    position_ms: u64,
    volume: f64,
    repeat: RepeatMode,
    shuffle: bool,
    /// Permutation of 0..playlist.len() used when shuffle is on.
    shuffle_order: Vec<usize>,
    /// Simple deterministic RNG state for shuffle.
    rng_state: u64,
}

impl Player {
    pub fn new() -> Self {
        Self {
            playlist: Vec::new(), state: PlayerState::Stopped,
            current_idx: None, position_ms: 0, volume: 1.0,
            repeat: RepeatMode::None, shuffle: false,
            shuffle_order: Vec::new(), rng_state: 0xdeadbeef,
        }
    }

    pub fn load_playlist(&mut self, tracks: Vec<Track>) {
        self.playlist = tracks;
        self.state = PlayerState::Stopped;
        self.current_idx = None;
        self.position_ms = 0;
        self.rebuild_shuffle_order();
    }

    pub fn state(&self) -> PlayerState { self.state }
    pub fn position_ms(&self) -> u64 { self.position_ms }
    pub fn volume(&self) -> f64 { self.volume }
    pub fn repeat(&self) -> RepeatMode { self.repeat }
    pub fn shuffle(&self) -> bool { self.shuffle }
    pub fn playlist(&self) -> &[Track] { &self.playlist }

    pub fn current_track(&self) -> Option<&Track> {
        let idx = self.current_idx?;
        let pl_idx = if self.shuffle { *self.shuffle_order.get(idx)? } else { idx };
        self.playlist.get(pl_idx)
    }

    pub fn set_volume(&mut self, v: f64) {
        self.volume = v.clamp(0.0, 1.0);
    }

    pub fn set_repeat(&mut self, mode: RepeatMode) { self.repeat = mode; }

    pub fn set_shuffle(&mut self, on: bool) {
        if on == self.shuffle { return; }
        self.shuffle = on;
        self.rebuild_shuffle_order();
        // Preserve "which track is current" across the mode flip.
        if let (Some(cur), true) = (self.current_idx, on) {
            // Find current playlist index in the new shuffle order.
            if let Some(new_idx) = self.shuffle_order.iter().position(|&i| i == cur) {
                self.current_idx = Some(new_idx);
            }
        }
        if !on {
            if let Some(cur) = self.current_idx {
                self.current_idx = self.shuffle_order.get(cur).copied();
            }
        }
    }

    /// Play/resume. Context-dependent:
    /// - Stopped + non-empty playlist → start track 0 (or first in shuffle order).
    /// - Paused → resume from current position.
    /// - Playing → no-op.
    /// Returns events produced by the action.
    pub fn play(&mut self) -> Vec<Event> {
        let mut events = Vec::new();
        match self.state {
            PlayerState::Playing => {}
            PlayerState::Paused => {
                self.state = PlayerState::Playing;
                events.push(Event::StateChanged { new: self.state });
            }
            PlayerState::Stopped => {
                if self.playlist.is_empty() { return events; }
                self.current_idx = Some(0);
                self.position_ms = 0;
                self.state = PlayerState::Playing;
                if let Some(t) = self.current_track() {
                    events.push(Event::TrackStarted { id: t.id });
                }
                events.push(Event::StateChanged { new: self.state });
            }
        }
        events
    }

    pub fn pause(&mut self) -> Vec<Event> {
        if matches!(self.state, PlayerState::Playing) {
            self.state = PlayerState::Paused;
            vec![Event::StateChanged { new: self.state }]
        } else {
            Vec::new()
        }
    }

    pub fn stop(&mut self) -> Vec<Event> {
        if matches!(self.state, PlayerState::Stopped) {
            return Vec::new();
        }
        self.state = PlayerState::Stopped;
        self.position_ms = 0;
        self.current_idx = None;
        vec![Event::StateChanged { new: self.state }]
    }

    /// Jump to the next track. Respects repeat/shuffle. If at playlist end
    /// with Repeat::None, stops.
    pub fn next_track(&mut self) -> Vec<Event> {
        let mut events = Vec::new();
        let Some(cur) = self.current_idx else {
            // No current track — treat as play-from-start.
            return self.play();
        };
        if matches!(self.repeat, RepeatMode::One) {
            self.position_ms = 0;
            if let Some(t) = self.current_track() {
                events.push(Event::TrackStarted { id: t.id });
            }
            return events;
        }
        let next_idx = cur + 1;
        if next_idx < self.playlist.len() {
            self.current_idx = Some(next_idx);
            self.position_ms = 0;
            if let Some(t) = self.current_track() {
                events.push(Event::TrackStarted { id: t.id });
            }
        } else if matches!(self.repeat, RepeatMode::All) {
            // Reshuffle for the next lap so repeat-all-with-shuffle gives a
            // different order each time through.
            if self.shuffle { self.rebuild_shuffle_order(); }
            self.current_idx = Some(0);
            self.position_ms = 0;
            if let Some(t) = self.current_track() {
                events.push(Event::TrackStarted { id: t.id });
            }
        } else {
            self.state = PlayerState::Stopped;
            self.current_idx = None;
            self.position_ms = 0;
            events.push(Event::PlaylistEnded);
            events.push(Event::StateChanged { new: self.state });
        }
        events
    }

    pub fn previous_track(&mut self) -> Vec<Event> {
        let mut events = Vec::new();
        let Some(cur) = self.current_idx else { return events; };
        if cur == 0 {
            // Restart the current track.
            self.position_ms = 0;
            if let Some(t) = self.current_track() {
                events.push(Event::TrackStarted { id: t.id });
            }
            return events;
        }
        self.current_idx = Some(cur - 1);
        self.position_ms = 0;
        if let Some(t) = self.current_track() {
            events.push(Event::TrackStarted { id: t.id });
        }
        events
    }

    pub fn seek_ms(&mut self, pos: u64) -> bool {
        let Some(t) = self.current_track() else { return false; };
        if pos > t.duration_ms { return false; }
        self.position_ms = pos;
        true
    }

    /// Advance playback by `delta_ms` milliseconds. Produces events when
    /// tracks end and the next one starts. Only advances when Playing.
    pub fn tick(&mut self, delta_ms: u64) -> Vec<Event> {
        if !matches!(self.state, PlayerState::Playing) { return Vec::new(); }
        let mut events = Vec::new();
        let mut remaining = delta_ms;
        while remaining > 0 {
            let Some(cur_track) = self.current_track().cloned() else {
                break;
            };
            let time_left = cur_track.duration_ms.saturating_sub(self.position_ms);
            if remaining < time_left {
                self.position_ms += remaining;
                remaining = 0;
            } else {
                remaining -= time_left;
                self.position_ms = cur_track.duration_ms;
                events.push(Event::TrackEnded { id: cur_track.id });
                let next_events = self.next_track();
                events.extend(next_events);
                if matches!(self.state, PlayerState::Stopped) { break; }
            }
        }
        events
    }

    fn rebuild_shuffle_order(&mut self) {
        self.shuffle_order = (0..self.playlist.len()).collect();
        if !self.shuffle || self.shuffle_order.len() <= 1 { return; }
        // Fisher-Yates with deterministic LCG.
        for i in (1..self.shuffle_order.len()).rev() {
            self.rng_state = self.rng_state
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let j = (self.rng_state >> 32) as usize % (i + 1);
            self.shuffle_order.swap(i, j);
        }
    }
}

impl Default for Player {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn t(id: u64, dur: u64) -> Track {
        Track { id, title: format!("Track {id}"), artist: "test".into(), duration_ms: dur }
    }

    #[test]
    fn new_player_is_stopped() {
        let p = Player::new();
        assert_eq!(p.state(), PlayerState::Stopped);
        assert!(p.current_track().is_none());
    }

    #[test]
    fn play_from_stopped_starts_first_track() {
        let mut p = Player::new();
        p.load_playlist(vec![t(1, 1000), t(2, 2000)]);
        let ev = p.play();
        assert_eq!(p.state(), PlayerState::Playing);
        assert_eq!(p.current_track().unwrap().id, 1);
        assert!(ev.contains(&Event::TrackStarted { id: 1 }));
    }

    #[test]
    fn pause_then_play_resumes_from_position() {
        let mut p = Player::new();
        p.load_playlist(vec![t(1, 10_000)]);
        p.play();
        p.tick(3000);
        assert_eq!(p.position_ms(), 3000);
        p.pause();
        assert_eq!(p.state(), PlayerState::Paused);
        // While paused, tick does nothing.
        p.tick(5000);
        assert_eq!(p.position_ms(), 3000);
        p.play();
        p.tick(2000);
        assert_eq!(p.position_ms(), 5000);
    }

    #[test]
    fn play_when_already_playing_is_noop() {
        let mut p = Player::new();
        p.load_playlist(vec![t(1, 5000)]);
        p.play();
        p.tick(1000);
        let before = p.position_ms();
        let ev = p.play();
        assert!(ev.is_empty());
        assert_eq!(p.position_ms(), before);
    }

    #[test]
    fn tick_advances_across_track_boundaries() {
        let mut p = Player::new();
        p.load_playlist(vec![t(1, 1000), t(2, 2000), t(3, 500)]);
        p.play();
        let ev = p.tick(1500);   // finishes track 1, 500ms into track 2
        assert!(ev.contains(&Event::TrackEnded { id: 1 }));
        assert!(ev.contains(&Event::TrackStarted { id: 2 }));
        assert_eq!(p.current_track().unwrap().id, 2);
        assert_eq!(p.position_ms(), 500);
    }

    #[test]
    fn playlist_end_stops_with_repeat_none() {
        let mut p = Player::new();
        p.load_playlist(vec![t(1, 1000)]);
        p.play();
        let ev = p.tick(2000);
        assert!(ev.contains(&Event::PlaylistEnded));
        assert_eq!(p.state(), PlayerState::Stopped);
    }

    #[test]
    fn repeat_all_wraps_to_start() {
        let mut p = Player::new();
        p.load_playlist(vec![t(1, 1000), t(2, 1000)]);
        p.set_repeat(RepeatMode::All);
        p.play();
        // 2500ms of 2000ms total: 1 full cycle + 500ms into track 1.
        let ev = p.tick(2500);
        assert!(ev.contains(&Event::TrackEnded { id: 2 }));
        assert_eq!(p.current_track().unwrap().id, 1);
        assert_eq!(p.position_ms(), 500);
        assert_eq!(p.state(), PlayerState::Playing);
    }

    #[test]
    fn repeat_one_restarts_same_track() {
        let mut p = Player::new();
        p.load_playlist(vec![t(1, 1000), t(2, 1000)]);
        p.set_repeat(RepeatMode::One);
        p.play();
        p.tick(1000);
        assert_eq!(p.current_track().unwrap().id, 1);
        assert_eq!(p.position_ms(), 0);
    }

    #[test]
    fn next_and_previous_track() {
        let mut p = Player::new();
        p.load_playlist(vec![t(1, 1000), t(2, 1000), t(3, 1000)]);
        p.play();
        p.next_track();
        assert_eq!(p.current_track().unwrap().id, 2);
        p.next_track();
        assert_eq!(p.current_track().unwrap().id, 3);
        p.previous_track();
        assert_eq!(p.current_track().unwrap().id, 2);
    }

    #[test]
    fn seek_within_track() {
        let mut p = Player::new();
        p.load_playlist(vec![t(1, 10_000)]);
        p.play();
        assert!(p.seek_ms(5000));
        assert_eq!(p.position_ms(), 5000);
        assert!(!p.seek_ms(15_000));   // past duration
    }

    #[test]
    fn volume_clamps_to_0_1() {
        let mut p = Player::new();
        p.set_volume(1.5);
        assert_eq!(p.volume(), 1.0);
        p.set_volume(-0.5);
        assert_eq!(p.volume(), 0.0);
        p.set_volume(0.7);
        assert!((p.volume() - 0.7).abs() < 1e-9);
    }

    #[test]
    fn shuffle_plays_every_track_exactly_once_per_lap() {
        let mut p = Player::new();
        p.load_playlist((1..=5).map(|i| t(i, 100)).collect());
        p.set_shuffle(true);
        p.play();
        let mut seen = std::collections::HashSet::new();
        seen.insert(p.current_track().unwrap().id);
        for _ in 0..4 { p.next_track(); seen.insert(p.current_track().unwrap().id); }
        assert_eq!(seen.len(), 5);
    }

    #[test]
    fn stop_resets_position_and_current() {
        let mut p = Player::new();
        p.load_playlist(vec![t(1, 10_000)]);
        p.play();
        p.tick(5000);
        p.stop();
        assert_eq!(p.state(), PlayerState::Stopped);
        assert_eq!(p.position_ms(), 0);
        assert!(p.current_track().is_none());
    }
}
