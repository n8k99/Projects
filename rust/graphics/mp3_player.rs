//! Mp3 Player — playlist navigation with shuffle/repeat + position tracking.
//!
//! Fifth Graphics project. Every music app — iTunes, Spotify, VLC,
//! Winamp — ships the same navigation kernel: a playlist, a queue
//! position, a shuffle mode (deterministic via seed), and a repeat
//! mode (Off / One / All) that changes what "next" means.
//! G118 models this kernel as pure state + transitions.
//!
//! Distinctive moves:
//!   * **Playlist as ordered track IDs**, separate from the track
//!      library.
//!   * **Shuffle as a deterministic permutation** — seeded Fisher-Yates
//!      over the playlist, generated once per "enable shuffle".
//!   * **Repeat modes change next-track logic** — Off: end at last;
//!      One: same track again; All: wrap to first.
//!   * **Position tracking within a track** plus tick-based drain.
//!   * **History stack** — ordered list of recently-played track IDs,
//!      for "previous track".

use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Track {
    pub id: u64,
    pub title: String,
    pub artist: String,
    pub duration_ms: u32,
}

#[derive(Debug, Clone)]
pub struct Library {
    pub tracks: BTreeMap<u64, Track>,
}

impl Library {
    pub fn new() -> Self { Self { tracks: BTreeMap::new() } }
    pub fn add(&mut self, track: Track) { self.tracks.insert(track.id, track); }
    pub fn get(&self, id: u64) -> Option<&Track> { self.tracks.get(&id) }
}

impl Default for Library { fn default() -> Self { Self::new() } }

#[derive(Debug, Clone)]
pub struct Playlist {
    pub id: u64,
    pub title: String,
    pub track_ids: Vec<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RepeatMode { Off, One, All }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayerState { Idle, Playing, Paused, Stopped }

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayEvent {
    pub kind: PlayEventKind,
    pub detail: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayEventKind {
    StateChanged,
    TrackChanged,
    RepeatChanged,
    ShuffleChanged,
}

#[derive(Debug, Clone)]
pub struct Player {
    pub playlist: Option<Playlist>,
    pub queue_order: Vec<u64>,       // effective play order (respects shuffle)
    pub queue_position: usize,        // index into queue_order
    pub position_ms: u32,             // within current track
    pub state: PlayerState,
    pub repeat: RepeatMode,
    pub shuffle: bool,
    pub shuffle_seed: u64,
    pub history: Vec<u64>,            // track IDs in order they've been played
    pub events: Vec<PlayEvent>,
}

impl Player {
    pub fn new() -> Self {
        Self {
            playlist: None,
            queue_order: Vec::new(),
            queue_position: 0,
            position_ms: 0,
            state: PlayerState::Idle,
            repeat: RepeatMode::Off,
            shuffle: false,
            shuffle_seed: 42,
            history: Vec::new(),
            events: Vec::new(),
        }
    }

    pub fn load_playlist(&mut self, playlist: Playlist) {
        self.queue_order = playlist.track_ids.clone();
        self.playlist = Some(playlist);
        self.queue_position = 0;
        self.position_ms = 0;
        if self.shuffle { self.apply_shuffle(); }
        self.set_state(PlayerState::Stopped);
    }

    pub fn play(&mut self) {
        if !self.queue_order.is_empty() {
            self.set_state(PlayerState::Playing);
        }
    }

    pub fn pause(&mut self) {
        if self.state == PlayerState::Playing {
            self.set_state(PlayerState::Paused);
        }
    }

    pub fn stop(&mut self) {
        self.set_state(PlayerState::Stopped);
        self.position_ms = 0;
    }

    /// Advance playback position by `ms`. Auto-advances across tracks.
    pub fn tick(&mut self, ms: u32, library: &Library) {
        if self.state != PlayerState::Playing { return; }
        if self.queue_order.is_empty() { return; }
        let Some(cur_id) = self.current_track_id() else { return; };
        let Some(track) = library.get(cur_id) else { return; };
        let remaining = track.duration_ms.saturating_sub(self.position_ms);
        if ms >= remaining {
            // Track finished — record history, advance.
            self.history.push(cur_id);
            self.position_ms = 0;
            let advance = self.compute_next();
            match advance {
                NextAction::Advance(next_pos) => {
                    self.queue_position = next_pos;
                    self.emit_track_changed(library);
                }
                NextAction::Repeat => {
                    // Stay on same track; position already 0.
                    self.emit_track_changed(library);
                }
                NextAction::End => {
                    self.set_state(PlayerState::Stopped);
                }
            }
        } else {
            self.position_ms += ms;
        }
    }

    /// Skip to next track (user-triggered).
    pub fn next(&mut self, library: &Library) -> bool {
        let Some(cur_id) = self.current_track_id() else { return false; };
        self.history.push(cur_id);
        self.position_ms = 0;
        match self.compute_next() {
            NextAction::Advance(p) => {
                self.queue_position = p;
                self.emit_track_changed(library);
                true
            }
            NextAction::Repeat => {
                self.emit_track_changed(library);
                true
            }
            NextAction::End => {
                self.set_state(PlayerState::Stopped);
                false
            }
        }
    }

    /// Go to previous track from history. If history empty, stays at start of current.
    pub fn prev(&mut self, library: &Library) -> bool {
        if self.position_ms > 3000 {
            // In most players, prev within first 3s of a track goes to actual previous;
            // otherwise just restarts the current track.
            self.position_ms = 0;
            return true;
        }
        match self.history.pop() {
            Some(prev_id) => {
                // Find its position in queue_order.
                if let Some(pos) = self.queue_order.iter().position(|&id| id == prev_id) {
                    self.queue_position = pos;
                    self.position_ms = 0;
                    self.emit_track_changed(library);
                    return true;
                }
                false
            }
            None => {
                self.position_ms = 0;
                false
            }
        }
    }

    pub fn set_repeat(&mut self, mode: RepeatMode) {
        if self.repeat != mode {
            self.repeat = mode;
            self.events.push(PlayEvent {
                kind: PlayEventKind::RepeatChanged,
                detail: format!("{mode:?}"),
            });
        }
    }

    pub fn set_shuffle(&mut self, on: bool) {
        if self.shuffle == on { return; }
        self.shuffle = on;
        self.events.push(PlayEvent {
            kind: PlayEventKind::ShuffleChanged,
            detail: if on { "on".into() } else { "off".into() },
        });
        if on { self.apply_shuffle(); }
        else { self.unshuffle(); }
    }

    pub fn current_track_id(&self) -> Option<u64> {
        self.queue_order.get(self.queue_position).copied()
    }

    fn set_state(&mut self, state: PlayerState) {
        if self.state != state {
            let old = self.state;
            self.state = state;
            self.events.push(PlayEvent {
                kind: PlayEventKind::StateChanged,
                detail: format!("{old:?} -> {state:?}"),
            });
        }
    }

    fn emit_track_changed(&mut self, _library: &Library) {
        if let Some(id) = self.current_track_id() {
            self.events.push(PlayEvent {
                kind: PlayEventKind::TrackChanged,
                detail: format!("track {id}"),
            });
        }
    }

    fn compute_next(&self) -> NextAction {
        match self.repeat {
            RepeatMode::One => NextAction::Repeat,
            RepeatMode::All => {
                let next = (self.queue_position + 1) % self.queue_order.len();
                NextAction::Advance(next)
            }
            RepeatMode::Off => {
                if self.queue_position + 1 < self.queue_order.len() {
                    NextAction::Advance(self.queue_position + 1)
                } else {
                    NextAction::End
                }
            }
        }
    }

    fn apply_shuffle(&mut self) {
        if let Some(pl) = &self.playlist {
            self.queue_order = pl.track_ids.clone();
            // Fisher-Yates with seeded LCG (same constants as G096).
            let mut state = self.shuffle_seed.wrapping_add(1);
            let n = self.queue_order.len();
            for i in (1..n).rev() {
                state = state.wrapping_mul(6364136223846793005)
                    .wrapping_add(1442695040888963407);
                let r = (state >> 33) as usize;
                let j = r % (i + 1);
                self.queue_order.swap(i, j);
            }
            // Try to keep current track at position 0.
            if let Some(cur_id) = self.current_track_id_from_playlist() {
                if let Some(pos) = self.queue_order.iter().position(|&x| x == cur_id) {
                    self.queue_order.swap(0, pos);
                    self.queue_position = 0;
                }
            }
        }
    }

    fn unshuffle(&mut self) {
        if let Some(pl) = &self.playlist {
            let cur_id = self.current_track_id();
            self.queue_order = pl.track_ids.clone();
            if let Some(id) = cur_id {
                if let Some(pos) = self.queue_order.iter().position(|&x| x == id) {
                    self.queue_position = pos;
                }
            }
        }
    }

    fn current_track_id_from_playlist(&self) -> Option<u64> {
        self.queue_order.get(self.queue_position).copied()
    }
}

impl Default for Player { fn default() -> Self { Self::new() } }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NextAction {
    Advance(usize),
    Repeat,
    End,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_library() -> Library {
        let mut lib = Library::new();
        for (id, title, dur) in [
            (1, "Alpha", 120_000),
            (2, "Beta", 180_000),
            (3, "Gamma", 150_000),
            (4, "Delta", 200_000),
        ] {
            lib.add(Track {
                id, title: title.into(), artist: "Various".into(),
                duration_ms: dur,
            });
        }
        lib
    }

    fn sample_playlist() -> Playlist {
        Playlist {
            id: 1, title: "Mix".into(),
            track_ids: vec![1, 2, 3, 4],
        }
    }

    #[test]
    fn starts_idle() {
        let p = Player::new();
        assert_eq!(p.state, PlayerState::Idle);
    }

    #[test]
    fn load_playlist_transitions_to_stopped() {
        let mut p = Player::new();
        p.load_playlist(sample_playlist());
        assert_eq!(p.state, PlayerState::Stopped);
        assert_eq!(p.queue_order.len(), 4);
        assert_eq!(p.current_track_id(), Some(1));
    }

    #[test]
    fn play_starts_playback() {
        let mut p = Player::new();
        p.load_playlist(sample_playlist());
        p.play();
        assert_eq!(p.state, PlayerState::Playing);
    }

    #[test]
    fn tick_advances_position() {
        let mut p = Player::new();
        p.load_playlist(sample_playlist());
        p.play();
        let lib = sample_library();
        p.tick(5000, &lib);
        assert_eq!(p.position_ms, 5000);
    }

    #[test]
    fn tick_advances_track_at_end() {
        let mut p = Player::new();
        p.load_playlist(sample_playlist());
        p.play();
        let lib = sample_library();
        // First track is 120s, drain all of it + 5s.
        p.tick(125_000, &lib);
        assert_eq!(p.current_track_id(), Some(2));
        assert_eq!(p.position_ms, 0);
    }

    #[test]
    fn repeat_off_ends_after_last_track() {
        let mut p = Player::new();
        p.load_playlist(sample_playlist());
        p.queue_position = 3;  // last
        p.play();
        let lib = sample_library();
        p.tick(250_000, &lib);  // overshoot last track
        assert_eq!(p.state, PlayerState::Stopped);
    }

    #[test]
    fn repeat_all_wraps_to_first() {
        let mut p = Player::new();
        p.load_playlist(sample_playlist());
        p.set_repeat(RepeatMode::All);
        p.queue_position = 3;
        p.play();
        let lib = sample_library();
        p.tick(250_000, &lib);
        assert_eq!(p.current_track_id(), Some(1));
        assert_eq!(p.state, PlayerState::Playing);
    }

    #[test]
    fn repeat_one_replays_same_track() {
        let mut p = Player::new();
        p.load_playlist(sample_playlist());
        p.set_repeat(RepeatMode::One);
        p.play();
        let lib = sample_library();
        let before = p.current_track_id();
        p.tick(130_000, &lib);
        assert_eq!(p.current_track_id(), before);
    }

    #[test]
    fn next_user_action_advances() {
        let mut p = Player::new();
        p.load_playlist(sample_playlist());
        p.play();
        let lib = sample_library();
        p.next(&lib);
        assert_eq!(p.current_track_id(), Some(2));
    }

    #[test]
    fn prev_within_3s_restarts_track() {
        let mut p = Player::new();
        p.load_playlist(sample_playlist());
        p.play();
        let lib = sample_library();
        p.tick(10_000, &lib);
        p.prev(&lib);
        assert_eq!(p.current_track_id(), Some(1));
        assert_eq!(p.position_ms, 0);
    }

    #[test]
    fn prev_after_3s_goes_to_previous_track() {
        let mut p = Player::new();
        p.load_playlist(sample_playlist());
        p.play();
        let lib = sample_library();
        p.tick(10_000, &lib);
        p.next(&lib);  // now on track 2; history has 1
        assert_eq!(p.current_track_id(), Some(2));
        p.prev(&lib);  // at position 0 of track 2 → go back to 1
        assert_eq!(p.current_track_id(), Some(1));
    }

    #[test]
    fn shuffle_permutes_queue() {
        let mut p = Player::new();
        p.load_playlist(sample_playlist());
        let before = p.queue_order.clone();
        p.set_shuffle(true);
        // With this seed + 4 tracks, we expect a different order.
        assert_ne!(p.queue_order, before);
        // Same track count.
        assert_eq!(p.queue_order.len(), 4);
        // All original ids still present.
        for id in &before {
            assert!(p.queue_order.contains(id));
        }
    }

    #[test]
    fn shuffle_is_deterministic() {
        let mut p1 = Player::new();
        let mut p2 = Player::new();
        p1.load_playlist(sample_playlist());
        p2.load_playlist(sample_playlist());
        p1.set_shuffle(true);
        p2.set_shuffle(true);
        assert_eq!(p1.queue_order, p2.queue_order);
    }

    #[test]
    fn unshuffle_restores_playlist_order() {
        let mut p = Player::new();
        p.load_playlist(sample_playlist());
        p.set_shuffle(true);
        p.set_shuffle(false);
        assert_eq!(p.queue_order, vec![1, 2, 3, 4]);
    }

    #[test]
    fn pause_stops_tick_drain() {
        let mut p = Player::new();
        p.load_playlist(sample_playlist());
        p.play();
        let lib = sample_library();
        p.tick(5000, &lib);
        p.pause();
        let before = p.position_ms;
        p.tick(5000, &lib);
        assert_eq!(p.position_ms, before);
    }

    #[test]
    fn stop_resets_position() {
        let mut p = Player::new();
        p.load_playlist(sample_playlist());
        p.play();
        let lib = sample_library();
        p.tick(10_000, &lib);
        p.stop();
        assert_eq!(p.position_ms, 0);
        assert_eq!(p.state, PlayerState::Stopped);
    }

    #[test]
    fn empty_playlist_does_nothing() {
        let mut p = Player::new();
        p.load_playlist(Playlist { id: 0, title: "".into(), track_ids: vec![] });
        p.play();
        assert_eq!(p.state, PlayerState::Stopped);
    }
}
