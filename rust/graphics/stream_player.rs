//! Stream Player — segmented video with buffer-based adaptive bitrate selection.
//!
//! Fourth Graphics project. Every HLS/DASH/MSS video client on the web
//! ships the same kernel: a stream has multiple bitrate renditions; the
//! player downloads one segment at a time; an **ABR algorithm**
//! (adaptive bitrate) picks each segment's quality based on current
//! buffer health and observed bandwidth. Buffer gets low → drop
//! quality. Bandwidth has headroom → step up quality.
//!
//! G117 models the kernel without real network I/O: `set_bandwidth`
//! simulates network conditions, `load_segment` schedules the next
//! download, `tick` drains the buffer over time. The ABR logic is
//! pure — a pure function over `(buffer_ms, bandwidth_bps, bitrates,
//! segment_bytes)` → chosen bitrate.

use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayerState {
    Idle,
    Buffering,
    Playing,
    Paused,
    Ended,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Segment {
    pub index: u32,
    pub duration_ms: u32,
    /// Bytes per bitrate: `bitrate_bps -> size_bytes`.
    pub sizes_by_bitrate: BTreeMap<u32, u64>,
}

impl Segment {
    pub fn size_for(&self, bitrate_bps: u32) -> Option<u64> {
        self.sizes_by_bitrate.get(&bitrate_bps).copied()
    }
}

#[derive(Debug, Clone)]
pub struct Stream {
    /// Available bitrates in bits-per-second, sorted ascending.
    pub bitrates_bps: Vec<u32>,
    pub segments: Vec<Segment>,
    /// Safety margin applied to the bandwidth estimate (0.0..=1.0).
    pub safety_factor: f64,
    /// Target buffer in ms — below this the player down-switches.
    pub target_buffer_ms: u32,
}

impl Stream {
    pub fn total_duration_ms(&self) -> u32 {
        self.segments.iter().map(|s| s.duration_ms).sum()
    }

    /// Pick the highest bitrate whose download at the given bandwidth
    /// would complete before the buffer drains. Falls back to the lowest
    /// if nothing fits.
    pub fn pick_bitrate(
        &self,
        buffer_ms: u32,
        bandwidth_bps: u32,
        segment: &Segment,
    ) -> u32 {
        let effective_bw = (bandwidth_bps as f64 * self.safety_factor).max(1.0);
        let raw_budget = buffer_ms.max(segment.duration_ms);
        let budget_ms = raw_budget.min(self.target_buffer_ms * 2) as f64;
        let mut chosen = self.bitrates_bps[0];
        for &bitrate in &self.bitrates_bps {
            if let Some(bytes) = segment.size_for(bitrate) {
                let download_ms = (bytes as f64 * 8.0 / effective_bw) * 1000.0;
                if download_ms <= budget_ms || bitrate == self.bitrates_bps[0] {
                    if download_ms <= budget_ms {
                        chosen = bitrate;
                    }
                }
            }
        }
        chosen
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerEvent {
    pub kind: EventKind,
    pub detail: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventKind {
    StateChanged,
    SegmentLoaded,
    BitrateSwitched,
    Rebuffer,
    Seeked,
}

#[derive(Debug, Clone)]
pub struct Player {
    pub state: PlayerState,
    pub current_segment: u32,
    pub buffer_ms: u32,
    pub played_ms: u32,
    pub bandwidth_bps: u32,
    pub current_bitrate: u32,
    pub events: Vec<PlayerEvent>,
    pub stream: Option<Stream>,
}

impl Player {
    pub fn new() -> Self {
        Self {
            state: PlayerState::Idle,
            current_segment: 0,
            buffer_ms: 0,
            played_ms: 0,
            bandwidth_bps: 1_000_000,  // 1 Mbps default
            current_bitrate: 0,
            events: Vec::new(),
            stream: None,
        }
    }

    pub fn attach(&mut self, stream: Stream) {
        self.current_bitrate = stream.bitrates_bps[0];
        self.stream = Some(stream);
        self.set_state(PlayerState::Buffering);
    }

    pub fn set_bandwidth(&mut self, bandwidth_bps: u32) {
        self.bandwidth_bps = bandwidth_bps;
    }

    pub fn set_state(&mut self, state: PlayerState) {
        if self.state != state {
            let old = self.state;
            self.state = state;
            self.events.push(PlayerEvent {
                kind: EventKind::StateChanged,
                detail: format!("{old:?} -> {state:?}"),
            });
        }
    }

    /// Download the next segment into the buffer. Picks bitrate via ABR.
    /// Returns true on success (segment existed + loaded).
    pub fn load_next_segment(&mut self) -> bool {
        let Some(stream) = self.stream.clone() else { return false; };
        if (self.current_segment as usize) >= stream.segments.len() {
            return false;
        }
        let segment = stream.segments[self.current_segment as usize].clone();
        let new_bitrate = stream.pick_bitrate(self.buffer_ms, self.bandwidth_bps, &segment);
        if new_bitrate != self.current_bitrate {
            self.events.push(PlayerEvent {
                kind: EventKind::BitrateSwitched,
                detail: format!("{} -> {}", self.current_bitrate, new_bitrate),
            });
            self.current_bitrate = new_bitrate;
        }
        self.buffer_ms = self.buffer_ms.saturating_add(segment.duration_ms);
        self.current_segment += 1;
        self.events.push(PlayerEvent {
            kind: EventKind::SegmentLoaded,
            detail: format!("segment {} @ {}bps", segment.index, new_bitrate),
        });
        if self.state == PlayerState::Buffering && self.buffer_ms >= stream.target_buffer_ms {
            self.set_state(PlayerState::Playing);
        }
        true
    }

    /// Drain the buffer by `ms` (playback). Transitions state based on
    /// buffer and end-of-stream.
    pub fn tick(&mut self, ms: u32) {
        if self.state != PlayerState::Playing { return; }
        let drained = ms.min(self.buffer_ms);
        self.buffer_ms -= drained;
        self.played_ms += drained;
        if let Some(stream) = &self.stream {
            let at_end = (self.current_segment as usize) >= stream.segments.len();
            if self.buffer_ms == 0 && at_end {
                self.set_state(PlayerState::Ended);
            } else if self.buffer_ms == 0 && !at_end {
                self.events.push(PlayerEvent {
                    kind: EventKind::Rebuffer,
                    detail: format!("buffer empty at {}ms", self.played_ms),
                });
                self.set_state(PlayerState::Buffering);
            }
        }
    }

    pub fn pause(&mut self) {
        if self.state == PlayerState::Playing { self.set_state(PlayerState::Paused); }
    }

    pub fn play(&mut self) {
        if self.state == PlayerState::Paused { self.set_state(PlayerState::Playing); }
    }

    /// Seek to segment N. Clears buffer; transitions to Buffering.
    pub fn seek_to_segment(&mut self, segment_idx: u32) -> bool {
        let Some(stream) = &self.stream else { return false; };
        if (segment_idx as usize) >= stream.segments.len() { return false; }
        self.current_segment = segment_idx;
        self.buffer_ms = 0;
        self.played_ms = stream.segments[..segment_idx as usize]
            .iter().map(|s| s.duration_ms).sum();
        self.events.push(PlayerEvent {
            kind: EventKind::Seeked,
            detail: format!("segment {segment_idx}"),
        });
        self.set_state(PlayerState::Buffering);
        true
    }
}

impl Default for Player { fn default() -> Self { Self::new() } }

#[cfg(test)]
mod tests {
    use super::*;

    fn mk_segment(index: u32, duration_ms: u32, sizes: &[(u32, u64)]) -> Segment {
        Segment {
            index, duration_ms,
            sizes_by_bitrate: sizes.iter().map(|&(k, v)| (k, v)).collect(),
        }
    }

    fn sample_stream() -> Stream {
        Stream {
            bitrates_bps: vec![500_000, 1_000_000, 2_000_000, 4_000_000],
            target_buffer_ms: 6000,
            safety_factor: 0.85,
            segments: (0..5).map(|i| {
                mk_segment(i, 2000, &[
                    (500_000, 125_000),     // 500 kbps * 2s / 8 = 125 KB
                    (1_000_000, 250_000),
                    (2_000_000, 500_000),
                    (4_000_000, 1_000_000),
                ])
            }).collect(),
        }
    }

    #[test]
    fn initial_state_is_idle() {
        let p = Player::new();
        assert_eq!(p.state, PlayerState::Idle);
    }

    #[test]
    fn attach_transitions_to_buffering() {
        let mut p = Player::new();
        p.attach(sample_stream());
        assert_eq!(p.state, PlayerState::Buffering);
        assert_eq!(p.current_bitrate, 500_000);  // starts at lowest
    }

    #[test]
    fn abr_picks_highest_bitrate_that_fits() {
        let stream = sample_stream();
        // 10 Mbps × 0.85 = 8.5 Mbps effective. 4 Mbps segment = 1MB = 8 Mbits,
        // download_ms = 8 / 8.5 * 1000 = 941ms < 6000ms buffer ceiling.
        let bw = 10_000_000;
        let seg = &stream.segments[0];
        let picked = stream.pick_bitrate(6000, bw, seg);
        assert_eq!(picked, 4_000_000);
    }

    #[test]
    fn abr_drops_when_bandwidth_low() {
        let stream = sample_stream();
        // 600 kbps × 0.85 = 510 kbps effective.
        // 500 kbps segment: 125KB = 1 Mbit / 0.51 Mbps = 1960ms. With 12000ms ceiling,
        // that still fits. 1 Mbps segment: 2 Mbit / 0.51 = 3920ms — also fits.
        // 2 Mbps: 4 Mbit / 0.51 = 7843ms > 6000ms × 2 = 12000ms ceiling? No, fits.
        // 4 Mbps: 8 Mbit / 0.51 = 15686ms > 12000ms → doesn't fit.
        let picked = stream.pick_bitrate(6000, 600_000, &stream.segments[0]);
        assert!(picked <= 2_000_000);  // at most 2 Mbps
    }

    #[test]
    fn abr_never_returns_empty_when_lowest_unfit() {
        // Ridiculous conditions: 1 bps bandwidth. Should still pick lowest.
        let stream = sample_stream();
        let picked = stream.pick_bitrate(0, 1, &stream.segments[0]);
        assert_eq!(picked, 500_000);  // the lowest
    }

    #[test]
    fn load_next_segment_increments_index() {
        let mut p = Player::new();
        p.attach(sample_stream());
        p.set_bandwidth(10_000_000);
        p.load_next_segment();
        assert_eq!(p.current_segment, 1);
        assert_eq!(p.buffer_ms, 2000);
    }

    #[test]
    fn buffering_transitions_to_playing_when_full() {
        let mut p = Player::new();
        p.attach(sample_stream());
        p.set_bandwidth(10_000_000);
        // Target is 6000ms; load 3 × 2000 = 6000.
        for _ in 0..3 { p.load_next_segment(); }
        assert_eq!(p.state, PlayerState::Playing);
    }

    #[test]
    fn tick_drains_buffer_while_playing() {
        let mut p = Player::new();
        p.attach(sample_stream());
        p.set_bandwidth(10_000_000);
        for _ in 0..3 { p.load_next_segment(); }
        let before = p.buffer_ms;
        p.tick(500);
        assert_eq!(p.buffer_ms, before - 500);
        assert_eq!(p.played_ms, 500);
    }

    #[test]
    fn empty_buffer_triggers_rebuffer_event() {
        let mut p = Player::new();
        p.attach(sample_stream());
        p.set_bandwidth(10_000_000);
        for _ in 0..3 { p.load_next_segment(); }
        // Drain fully without loading more.
        p.tick(6000);
        assert_eq!(p.state, PlayerState::Buffering);
        assert!(p.events.iter().any(|e| e.kind == EventKind::Rebuffer));
    }

    #[test]
    fn full_playback_ends_cleanly() {
        let mut p = Player::new();
        p.attach(sample_stream());
        p.set_bandwidth(10_000_000);
        // Load all 5 segments.
        for _ in 0..5 { p.load_next_segment(); }
        // 10,000ms total. Drain in two ticks.
        p.tick(5000);
        p.tick(5000);
        assert_eq!(p.state, PlayerState::Ended);
        assert_eq!(p.played_ms, 10_000);
    }

    #[test]
    fn pause_and_resume() {
        let mut p = Player::new();
        p.attach(sample_stream());
        p.set_bandwidth(10_000_000);
        for _ in 0..3 { p.load_next_segment(); }
        p.pause();
        assert_eq!(p.state, PlayerState::Paused);
        let before = p.buffer_ms;
        p.tick(500);  // paused: no drain
        assert_eq!(p.buffer_ms, before);
        p.play();
        assert_eq!(p.state, PlayerState::Playing);
    }

    #[test]
    fn seek_resets_buffer_and_moves_to_buffering() {
        let mut p = Player::new();
        p.attach(sample_stream());
        p.set_bandwidth(10_000_000);
        for _ in 0..3 { p.load_next_segment(); }
        p.seek_to_segment(4);
        assert_eq!(p.current_segment, 4);
        assert_eq!(p.buffer_ms, 0);
        assert_eq!(p.played_ms, 8000);  // first 4 segments
        assert_eq!(p.state, PlayerState::Buffering);
    }

    #[test]
    fn seek_past_end_rejected() {
        let mut p = Player::new();
        p.attach(sample_stream());
        assert!(!p.seek_to_segment(999));
    }

    #[test]
    fn bitrate_switch_event_fires() {
        let mut p = Player::new();
        p.attach(sample_stream());
        p.set_bandwidth(10_000_000);
        p.load_next_segment();   // should switch up
        assert!(p.events.iter().any(|e| e.kind == EventKind::BitrateSwitched));
    }

    #[test]
    fn total_duration_sums_segments() {
        let stream = sample_stream();
        assert_eq!(stream.total_duration_ms(), 10_000);
    }

    #[test]
    fn no_stream_means_no_loads() {
        let mut p = Player::new();
        assert!(!p.load_next_segment());
    }
}
