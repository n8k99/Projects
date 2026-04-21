//! Screen Saver — idle FSM + bouncing-line physics + dim ramp + activity reset.
//!
//! Fifteenth Graphics project. Distinctive moves:
//!
//!   * **Six-state idle FSM** — Active → Dimming → Saver → Locked →
//!     Waking → Active. Each transition is time-triggered except Waking,
//!     which is input-triggered.
//!   * **Time-parameterized physics** — each bouncing line endpoint has
//!     a velocity; `tick(ms)` integrates position, reflects off canvas
//!     edges. Integer-scaled (positions and velocities in sub-pixel
//!     units × 100) for cross-language byte-identity.
//!   * **Dim ramp opacity** — during Dimming, opacity interpolates from
//!     0 to 255 linearly over `dim_ms`. Exposed as a scalar; UI can blend
//!     pixel content with black at that ratio.
//!   * **Activity stream resets idle** — any input event pushes
//!     `last_activity_ms = elapsed`. Idle is derived: `elapsed -
//!     last_activity_ms >= idle_ms`. Pure comparison.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SaverState {
    Active,
    Dimming,
    Saver,
    Locked,
    Waking,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Config {
    pub idle_ms: u64,
    pub dim_ms: u64,
    pub auto_lock_ms: u64,
    pub wake_ms: u64,
    pub canvas_width: i32,
    pub canvas_height: i32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            idle_ms: 60_000,
            dim_ms: 3_000,
            auto_lock_ms: 30_000,
            wake_ms: 500,
            canvas_width: 1920,
            canvas_height: 1080,
        }
    }
}

/// Position and velocity in sub-pixel units (×100).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Endpoint {
    pub x: i64, // sub-pixel × 100
    pub y: i64,
    pub vx: i64, // sub-pixel per ms × 100 (i.e. ×10000 per second)
    pub vy: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RgbColor { pub r: u8, pub g: u8, pub b: u8 }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BouncingLine {
    pub p1: Endpoint,
    pub p2: Endpoint,
    pub color: RgbColor,
}

impl BouncingLine {
    pub fn pixel_endpoints(&self) -> ((i32, i32), (i32, i32)) {
        ((self.p1.x as i32 / 100, self.p1.y as i32 / 100),
         (self.p2.x as i32 / 100, self.p2.y as i32 / 100))
    }
}

#[derive(Debug, Clone)]
pub enum SaverEvent {
    StateChanged { from: SaverState, to: SaverState, at_ms: u64 },
    ActivityResetTimer { at_ms: u64 },
    LineBounced { line_index: usize, axis: char, at_ms: u64 },
}

#[derive(Debug, Clone)]
pub struct ScreenSaver {
    pub state: SaverState,
    pub config: Config,
    pub lines: Vec<BouncingLine>,
    pub elapsed_ms: u64,
    pub last_activity_ms: u64,
    pub state_entered_ms: u64,
    pub events: Vec<SaverEvent>,
}

impl ScreenSaver {
    pub fn new(config: Config) -> Self {
        Self {
            state: SaverState::Active,
            config, lines: vec![],
            elapsed_ms: 0, last_activity_ms: 0, state_entered_ms: 0,
            events: vec![],
        }
    }

    pub fn add_line(&mut self, line: BouncingLine) {
        self.lines.push(line);
    }

    /// Current opacity during Dimming (0..=255). 0 at entry, 255 at dim_ms.
    pub fn dim_opacity(&self) -> u16 {
        if self.state != SaverState::Dimming { return 0; }
        let since = self.elapsed_ms.saturating_sub(self.state_entered_ms);
        if self.config.dim_ms == 0 { return 255; }
        ((since.min(self.config.dim_ms) * 255) / self.config.dim_ms) as u16
    }

    pub fn record_activity(&mut self) {
        self.last_activity_ms = self.elapsed_ms;
        self.events.push(SaverEvent::ActivityResetTimer { at_ms: self.elapsed_ms });
        // If we're in Saver/Dimming, transition to Waking
        match self.state {
            SaverState::Dimming | SaverState::Saver => {
                self.transition_to(SaverState::Waking);
            }
            SaverState::Locked => {
                // Locked requires explicit unlock(); activity alone doesn't wake
            }
            _ => {}
        }
    }

    pub fn unlock(&mut self) {
        if self.state == SaverState::Locked {
            self.transition_to(SaverState::Waking);
            self.last_activity_ms = self.elapsed_ms;
        }
    }

    fn transition_to(&mut self, new_state: SaverState) {
        if self.state == new_state { return; }
        let from = self.state;
        self.state = new_state;
        self.state_entered_ms = self.elapsed_ms;
        self.events.push(SaverEvent::StateChanged {
            from, to: new_state, at_ms: self.elapsed_ms,
        });
    }

    /// Integrate physics: update positions, reflect off edges.
    fn integrate_physics(&mut self, ms: u64) {
        let cw = self.config.canvas_width as i64 * 100;
        let ch = self.config.canvas_height as i64 * 100;
        let ms_i = ms as i64;
        for (idx, line) in self.lines.iter_mut().enumerate() {
            for (label, ep) in [('1', &mut line.p1), ('2', &mut line.p2)] {
                ep.x += ep.vx * ms_i / 100;
                ep.y += ep.vy * ms_i / 100;
                if ep.x < 0 {
                    ep.x = -ep.x;
                    ep.vx = -ep.vx;
                    self.events.push(SaverEvent::LineBounced {
                        line_index: idx, axis: 'x', at_ms: self.elapsed_ms,
                    });
                }
                if ep.x > cw {
                    ep.x = 2 * cw - ep.x;
                    ep.vx = -ep.vx;
                    self.events.push(SaverEvent::LineBounced {
                        line_index: idx, axis: 'x', at_ms: self.elapsed_ms,
                    });
                }
                if ep.y < 0 {
                    ep.y = -ep.y;
                    ep.vy = -ep.vy;
                    self.events.push(SaverEvent::LineBounced {
                        line_index: idx, axis: 'y', at_ms: self.elapsed_ms,
                    });
                }
                if ep.y > ch {
                    ep.y = 2 * ch - ep.y;
                    ep.vy = -ep.vy;
                    self.events.push(SaverEvent::LineBounced {
                        line_index: idx, axis: 'y', at_ms: self.elapsed_ms,
                    });
                }
                let _ = label;
            }
        }
    }

    pub fn tick(&mut self, ms: u64) {
        self.elapsed_ms += ms;
        // FSM transitions
        let since_state = self.elapsed_ms - self.state_entered_ms;
        match self.state {
            SaverState::Active => {
                let idle = self.elapsed_ms.saturating_sub(self.last_activity_ms);
                if idle >= self.config.idle_ms {
                    self.transition_to(SaverState::Dimming);
                }
            }
            SaverState::Dimming => {
                if since_state >= self.config.dim_ms {
                    self.transition_to(SaverState::Saver);
                }
            }
            SaverState::Saver => {
                if since_state >= self.config.auto_lock_ms {
                    self.transition_to(SaverState::Locked);
                }
                self.integrate_physics(ms);
            }
            SaverState::Locked => {
                // no auto-transition
            }
            SaverState::Waking => {
                if since_state >= self.config.wake_ms {
                    self.transition_to(SaverState::Active);
                    self.last_activity_ms = self.elapsed_ms;
                }
            }
        }
    }
}

// ---------- tests ----------

#[cfg(test)]
mod tests {
    use super::*;

    fn small_config() -> Config {
        Config {
            idle_ms: 1_000, dim_ms: 200, auto_lock_ms: 500, wake_ms: 100,
            canvas_width: 100, canvas_height: 100,
        }
    }

    #[test]
    fn starts_active() {
        let s = ScreenSaver::new(small_config());
        assert_eq!(s.state, SaverState::Active);
    }

    #[test]
    fn idle_transitions_to_dimming() {
        let mut s = ScreenSaver::new(small_config());
        s.tick(500);
        assert_eq!(s.state, SaverState::Active);
        s.tick(600);
        assert_eq!(s.state, SaverState::Dimming);
    }

    #[test]
    fn dimming_transitions_to_saver() {
        let mut s = ScreenSaver::new(small_config());
        s.tick(1100); // enters Dimming
        s.tick(250); // dim_ms=200 exceeded
        assert_eq!(s.state, SaverState::Saver);
    }

    #[test]
    fn saver_transitions_to_locked_after_auto_lock() {
        let mut s = ScreenSaver::new(small_config());
        s.tick(1100); // Dimming
        s.tick(250); // Saver
        s.tick(600); // past auto_lock_ms=500
        assert_eq!(s.state, SaverState::Locked);
    }

    #[test]
    fn activity_resets_in_active() {
        let mut s = ScreenSaver::new(small_config());
        s.tick(500);
        s.record_activity();
        s.tick(500);
        // last_activity was at 500; now at 1000, idle only 500ms < 1000
        assert_eq!(s.state, SaverState::Active);
    }

    #[test]
    fn activity_during_dimming_wakes() {
        let mut s = ScreenSaver::new(small_config());
        s.tick(1100); // Dimming
        s.record_activity();
        assert_eq!(s.state, SaverState::Waking);
    }

    #[test]
    fn activity_during_saver_wakes() {
        let mut s = ScreenSaver::new(small_config());
        s.tick(1100); // Dimming
        s.tick(250); // Saver
        s.record_activity();
        assert_eq!(s.state, SaverState::Waking);
    }

    #[test]
    fn activity_during_locked_does_not_wake() {
        let mut s = ScreenSaver::new(small_config());
        s.tick(1100); // Dimming
        s.tick(250); // Saver
        s.tick(600); // Locked
        s.record_activity();
        assert_eq!(s.state, SaverState::Locked);
    }

    #[test]
    fn explicit_unlock_wakes() {
        let mut s = ScreenSaver::new(small_config());
        s.tick(1100); s.tick(250); s.tick(600);
        assert_eq!(s.state, SaverState::Locked);
        s.unlock();
        assert_eq!(s.state, SaverState::Waking);
    }

    #[test]
    fn waking_transitions_back_to_active() {
        let mut s = ScreenSaver::new(small_config());
        s.tick(1100); // Dimming
        s.record_activity(); // Waking
        s.tick(150); // wake_ms=100 passed
        assert_eq!(s.state, SaverState::Active);
    }

    #[test]
    fn dim_opacity_ramps() {
        let mut s = ScreenSaver::new(small_config());
        s.tick(1100); // enters Dimming at elapsed=1100
        assert_eq!(s.dim_opacity(), 0);
        s.tick(100); // half of dim_ms=200
        let o = s.dim_opacity();
        assert!(o > 0 && o < 255);
    }

    #[test]
    fn dim_opacity_zero_when_not_dimming() {
        let s = ScreenSaver::new(small_config());
        assert_eq!(s.dim_opacity(), 0);
    }

    #[test]
    fn physics_only_advances_in_saver() {
        let mut s = ScreenSaver::new(small_config());
        s.add_line(BouncingLine {
            p1: Endpoint { x: 10_00, y: 10_00, vx: 100, vy: 0 },
            p2: Endpoint { x: 50_00, y: 50_00, vx: 100, vy: 0 },
            color: RgbColor { r: 255, g: 0, b: 0 },
        });
        s.tick(200); // in Active
        assert_eq!(s.lines[0].p1.x, 10_00); // no physics in Active
        s.tick(1000); // Dimming
        s.tick(200); // Saver
        s.tick(100); // physics integrates
        assert!(s.lines[0].p1.x > 10_00);
    }

    #[test]
    fn line_bounces_off_edge() {
        let mut s = ScreenSaver::new(small_config());
        // Put an endpoint near the right edge, moving right fast.
        s.add_line(BouncingLine {
            p1: Endpoint { x: 95_00, y: 50_00, vx: 10_000, vy: 0 }, // 1 px/ms
            p2: Endpoint { x: 50_00, y: 50_00, vx: 0, vy: 0 },
            color: RgbColor { r: 255, g: 255, b: 255 },
        });
        // Advance to Saver
        s.tick(1100); s.tick(200);
        // Canvas width = 100 px = 10_000 sub-px; at 1 px/ms for 10ms → x = 9500 + 1000 = 10500
        // Reflection: new x = 2*10000 - 10500 = 9500; vx flipped to -10_000
        s.tick(10);
        // Check that vx flipped
        assert!(s.lines[0].p1.vx < 0);
    }

    #[test]
    fn events_log_transitions() {
        let mut s = ScreenSaver::new(small_config());
        s.tick(1100); s.tick(250); s.tick(600);
        let transitions: Vec<_> = s.events.iter().filter(|e|
            matches!(e, SaverEvent::StateChanged { .. })).collect();
        // Active → Dimming → Saver → Locked = 3 transitions
        assert_eq!(transitions.len(), 3);
    }
}
