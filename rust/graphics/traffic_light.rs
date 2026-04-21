//! Traffic Light — coordinated super-state FSM + preemption + adaptive timing.
//!
//! Twelfth Graphics project. Distinctive moves:
//!
//!   * **Coordinated super-state** — the intersection's state is one of
//!     five phases: NSGreen, NSYellow, AllRed, EWGreen, EWYellow. The
//!     individual light colours for each direction are *derived* from
//!     the phase. Invalid combinations (NS green while EW green) are
//!     unrepresentable.
//!   * **Emergency preemption with state preservation** — when an
//!     emergency begins, the current phase and elapsed are saved;
//!     intersection enters `EmergencyAllRed`. When emergency ends,
//!     resume from the saved phase.
//!   * **Adaptive green extension** — green phases can extend up to
//!     `max_green_extension_ms` if a queue is detected in that direction.
//!     The decision is a pure function of queue + elapsed + cap.
//!   * **Pedestrian walk gated to AllRed** — `request_ped_crossing`
//!     only triggers the walk phase during AllRed; otherwise it queues
//!     the request for the next AllRed.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    NSGreen,
    NSYellow,
    AllRed,
    EWGreen,
    EWYellow,
    EmergencyAllRed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LightColor { Red, Yellow, Green }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PedSignal { DontWalk, Walk, Flashing }

impl Phase {
    pub fn ns_light(self) -> LightColor {
        match self {
            Phase::NSGreen => LightColor::Green,
            Phase::NSYellow => LightColor::Yellow,
            _ => LightColor::Red,
        }
    }

    pub fn ew_light(self) -> LightColor {
        match self {
            Phase::EWGreen => LightColor::Green,
            Phase::EWYellow => LightColor::Yellow,
            _ => LightColor::Red,
        }
    }

    /// The nominal next phase in the cycle (ignoring preemption).
    pub fn next_in_cycle(self) -> Phase {
        match self {
            Phase::NSGreen => Phase::NSYellow,
            Phase::NSYellow => Phase::AllRed,
            Phase::AllRed => Phase::EWGreen,
            Phase::EWGreen => Phase::EWYellow,
            Phase::EWYellow => Phase::AllRed, // then NS, but we flip after AllRed → EW → AllRed → NS via a flag
            Phase::EmergencyAllRed => Phase::EmergencyAllRed,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Config {
    pub ns_green_ms: u64,
    pub ns_yellow_ms: u64,
    pub ew_green_ms: u64,
    pub ew_yellow_ms: u64,
    pub all_red_ms: u64,
    pub max_green_extension_ms: u64,
    pub ped_walk_ms: u64,
    pub ped_flashing_ms: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            ns_green_ms: 20_000,
            ns_yellow_ms: 3_000,
            ew_green_ms: 20_000,
            ew_yellow_ms: 3_000,
            all_red_ms: 2_000,
            max_green_extension_ms: 10_000,
            ped_walk_ms: 8_000,
            ped_flashing_ms: 5_000,
        }
    }
}

#[derive(Debug, Clone)]
pub enum IntersectionEvent {
    PhaseChanged { from: Phase, to: Phase, at_ms: u64 },
    GreenExtended { direction: Direction, by_ms: u64 },
    PedRequested { at_ms: u64 },
    PedWalkStarted { at_ms: u64 },
    PedWalkEnded { at_ms: u64 },
    EmergencyStarted { at_ms: u64 },
    EmergencyEnded { at_ms: u64 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction { NS, EW }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PedState { Inactive, Walk, Flashing }

#[derive(Debug, Clone)]
pub struct Intersection {
    pub phase: Phase,
    pub phase_elapsed_ms: u64,
    pub phase_duration_ms: u64, // effective duration (may include extension)
    pub cycle_side: Direction, // tracks AllRed → NS or AllRed → EW
    pub config: Config,
    pub ns_queue: u32,
    pub ew_queue: u32,
    pub ped_requested: bool,
    ped_state: PedState,
    ped_elapsed_ms: u64,
    saved_phase: Option<(Phase, u64, Direction)>, // (phase, elapsed, cycle_side)
    pub total_elapsed_ms: u64,
    pub events: Vec<IntersectionEvent>,
}

impl Intersection {
    pub fn new(config: Config) -> Self {
        let phase = Phase::NSGreen;
        let phase_duration_ms = config.ns_green_ms;
        Self {
            phase, phase_elapsed_ms: 0, phase_duration_ms,
            cycle_side: Direction::EW, // after next AllRed, go to EW
            config,
            ns_queue: 0, ew_queue: 0,
            ped_requested: false,
            ped_state: PedState::Inactive,
            ped_elapsed_ms: 0,
            saved_phase: None,
            total_elapsed_ms: 0,
            events: vec![],
        }
    }

    pub fn ns_light(&self) -> LightColor { self.phase.ns_light() }
    pub fn ew_light(&self) -> LightColor { self.phase.ew_light() }
    pub fn ped_signal(&self) -> PedSignal {
        match self.ped_state {
            PedState::Inactive => PedSignal::DontWalk,
            PedState::Walk => PedSignal::Walk,
            PedState::Flashing => PedSignal::Flashing,
        }
    }

    pub fn phase_duration_for(&self, p: Phase) -> u64 {
        match p {
            Phase::NSGreen => self.config.ns_green_ms,
            Phase::NSYellow => self.config.ns_yellow_ms,
            Phase::EWGreen => self.config.ew_green_ms,
            Phase::EWYellow => self.config.ew_yellow_ms,
            Phase::AllRed => self.config.all_red_ms,
            Phase::EmergencyAllRed => u64::MAX, // cleared by end_emergency()
        }
    }

    pub fn set_ns_queue(&mut self, n: u32) { self.ns_queue = n; }
    pub fn set_ew_queue(&mut self, n: u32) { self.ew_queue = n; }

    pub fn request_ped_crossing(&mut self) {
        if self.ped_state != PedState::Inactive { return; }
        if !self.ped_requested {
            self.ped_requested = true;
            self.events.push(IntersectionEvent::PedRequested {
                at_ms: self.total_elapsed_ms,
            });
        }
    }

    pub fn begin_emergency(&mut self) {
        if self.phase == Phase::EmergencyAllRed { return; }
        self.saved_phase = Some((self.phase, self.phase_elapsed_ms, self.cycle_side));
        let at = self.total_elapsed_ms;
        let from = self.phase;
        self.phase = Phase::EmergencyAllRed;
        self.phase_elapsed_ms = 0;
        self.phase_duration_ms = u64::MAX;
        self.ped_state = PedState::Inactive;
        self.events.push(IntersectionEvent::EmergencyStarted { at_ms: at });
        self.events.push(IntersectionEvent::PhaseChanged {
            from, to: Phase::EmergencyAllRed, at_ms: at,
        });
    }

    pub fn end_emergency(&mut self) {
        if self.phase != Phase::EmergencyAllRed { return; }
        let at = self.total_elapsed_ms;
        let from = self.phase;
        if let Some((saved_phase, saved_elapsed, saved_side)) = self.saved_phase.take() {
            self.phase = saved_phase;
            self.phase_elapsed_ms = saved_elapsed;
            self.phase_duration_ms = self.phase_duration_for(saved_phase);
            self.cycle_side = saved_side;
        } else {
            self.phase = Phase::AllRed;
            self.phase_elapsed_ms = 0;
            self.phase_duration_ms = self.config.all_red_ms;
        }
        self.events.push(IntersectionEvent::EmergencyEnded { at_ms: at });
        self.events.push(IntersectionEvent::PhaseChanged {
            from, to: self.phase, at_ms: at,
        });
    }

    /// Try to extend the current green phase if a queue is detected.
    /// Returns bytes extended.
    fn consider_extension(&mut self) -> u64 {
        let direction = match self.phase {
            Phase::NSGreen => Some(Direction::NS),
            Phase::EWGreen => Some(Direction::EW),
            _ => None,
        };
        let Some(dir) = direction else { return 0; };
        let queue = match dir { Direction::NS => self.ns_queue, Direction::EW => self.ew_queue };
        if queue == 0 { return 0; }

        let base = match dir {
            Direction::NS => self.config.ns_green_ms,
            Direction::EW => self.config.ew_green_ms,
        };
        let already_extended = self.phase_duration_ms.saturating_sub(base);
        let remaining_cap = self.config.max_green_extension_ms.saturating_sub(already_extended);
        if remaining_cap == 0 { return 0; }

        // grant a second batch proportional to queue (capped)
        let grant = (queue as u64 * 2_000).min(remaining_cap);
        self.phase_duration_ms += grant;
        if grant > 0 {
            self.events.push(IntersectionEvent::GreenExtended {
                direction: dir, by_ms: grant,
            });
        }
        grant
    }

    fn next_phase(&self) -> (Phase, Direction) {
        match self.phase {
            Phase::NSGreen => (Phase::NSYellow, self.cycle_side),
            Phase::NSYellow => (Phase::AllRed, Direction::EW),
            Phase::AllRed => match self.cycle_side {
                Direction::EW => (Phase::EWGreen, Direction::NS),
                Direction::NS => (Phase::NSGreen, Direction::EW),
            },
            Phase::EWGreen => (Phase::EWYellow, self.cycle_side),
            Phase::EWYellow => (Phase::AllRed, Direction::NS),
            Phase::EmergencyAllRed => (Phase::EmergencyAllRed, self.cycle_side),
        }
    }

    fn transition_phase(&mut self) -> bool {
        let (new_phase, new_side) = self.next_phase();
        let from = self.phase;
        self.phase = new_phase;
        self.cycle_side = new_side;
        self.phase_elapsed_ms = 0;
        self.phase_duration_ms = self.phase_duration_for(new_phase);
        self.events.push(IntersectionEvent::PhaseChanged {
            from, to: new_phase, at_ms: self.total_elapsed_ms,
        });

        // Gate pedestrian walk to AllRed entry
        if new_phase == Phase::AllRed && self.ped_requested {
            self.ped_requested = false;
            self.ped_state = PedState::Walk;
            self.ped_elapsed_ms = 0;
            self.events.push(IntersectionEvent::PedWalkStarted {
                at_ms: self.total_elapsed_ms,
            });
            return true;
        }
        false
    }

    fn advance_pedestrian(&mut self, ms: u64) {
        if self.ped_state == PedState::Inactive { return; }
        self.ped_elapsed_ms += ms;
        match self.ped_state {
            PedState::Walk if self.ped_elapsed_ms >= self.config.ped_walk_ms => {
                self.ped_state = PedState::Flashing;
                self.ped_elapsed_ms = 0;
            }
            PedState::Flashing if self.ped_elapsed_ms >= self.config.ped_flashing_ms => {
                self.ped_state = PedState::Inactive;
                self.ped_elapsed_ms = 0;
                self.events.push(IntersectionEvent::PedWalkEnded {
                    at_ms: self.total_elapsed_ms,
                });
            }
            _ => {}
        }
    }

    pub fn tick(&mut self, ms: u64) {
        self.total_elapsed_ms += ms;
        if self.phase == Phase::EmergencyAllRed {
            // Emergency never auto-advances; pedestrian also frozen.
            return;
        }

        self.phase_elapsed_ms += ms;

        // Try to extend green phases at the end of their nominal duration
        if (self.phase == Phase::NSGreen || self.phase == Phase::EWGreen)
            && self.phase_elapsed_ms >= self.phase_duration_ms
        {
            self.consider_extension();
        }

        let mut ped_just_started_at_remaining: Option<u64> = None;
        while self.phase_elapsed_ms >= self.phase_duration_ms
            && self.phase != Phase::EmergencyAllRed
        {
            let overflow = self.phase_elapsed_ms - self.phase_duration_ms;
            let started = self.transition_phase();
            self.phase_elapsed_ms = overflow.min(self.phase_duration_ms);
            if started {
                ped_just_started_at_remaining = Some(overflow);
            }
        }

        // Pedestrian advances by the time it has actually been active during
        // this tick: either `ms` (active before tick) or `overflow` (just started).
        let ped_advance_ms = match (self.ped_state, ped_just_started_at_remaining) {
            (PedState::Inactive, _) => 0,
            (_, Some(remaining)) => remaining,
            _ => ms,
        };
        self.advance_pedestrian(ped_advance_ms);
    }
}

// ---------- tests ----------

#[cfg(test)]
mod tests {
    use super::*;

    fn small_config() -> Config {
        Config {
            ns_green_ms: 1000,
            ns_yellow_ms: 200,
            ew_green_ms: 1000,
            ew_yellow_ms: 200,
            all_red_ms: 100,
            max_green_extension_ms: 500,
            ped_walk_ms: 500,
            ped_flashing_ms: 300,
        }
    }

    #[test]
    fn initial_phase_is_ns_green() {
        let i = Intersection::new(small_config());
        assert_eq!(i.phase, Phase::NSGreen);
        assert_eq!(i.ns_light(), LightColor::Green);
        assert_eq!(i.ew_light(), LightColor::Red);
    }

    #[test]
    fn phase_cycle_nsgreen_to_nsyellow_to_allred_to_ewgreen() {
        let mut i = Intersection::new(small_config());
        i.tick(1000); // NSGreen done
        assert_eq!(i.phase, Phase::NSYellow);
        i.tick(200); // NSYellow done
        assert_eq!(i.phase, Phase::AllRed);
        i.tick(100); // AllRed done
        assert_eq!(i.phase, Phase::EWGreen);
        i.tick(1000); // EWGreen done
        assert_eq!(i.phase, Phase::EWYellow);
        i.tick(200); // EWYellow done
        assert_eq!(i.phase, Phase::AllRed);
        i.tick(100); // AllRed done
        assert_eq!(i.phase, Phase::NSGreen);
    }

    #[test]
    fn lights_are_complementary_during_green() {
        let mut i = Intersection::new(small_config());
        assert_eq!((i.ns_light(), i.ew_light()), (LightColor::Green, LightColor::Red));
        i.tick(1000);
        assert_eq!((i.ns_light(), i.ew_light()), (LightColor::Yellow, LightColor::Red));
        i.tick(200);
        assert_eq!((i.ns_light(), i.ew_light()), (LightColor::Red, LightColor::Red));
        i.tick(100);
        assert_eq!((i.ns_light(), i.ew_light()), (LightColor::Red, LightColor::Green));
    }

    #[test]
    fn green_extends_when_queue_detected() {
        let mut i = Intersection::new(small_config());
        i.set_ns_queue(3);
        i.tick(1000); // should extend by min(3*2000, 500) = 500
        // NSGreen should still be active (not yet at duration + extension)
        assert_eq!(i.phase, Phase::NSGreen);
        assert_eq!(i.phase_duration_ms, 1500);
        i.tick(500); // extension used
        assert_eq!(i.phase, Phase::NSYellow);
    }

    #[test]
    fn extension_capped_at_max() {
        let mut i = Intersection::new(small_config());
        i.set_ns_queue(100); // huge queue would want massive extension
        i.tick(1000);
        assert_eq!(i.phase_duration_ms, 1500); // capped at 1000 + 500
    }

    #[test]
    fn no_extension_without_queue() {
        let mut i = Intersection::new(small_config());
        i.set_ns_queue(0);
        i.tick(1000);
        assert_eq!(i.phase, Phase::NSYellow);
    }

    #[test]
    fn emergency_jumps_to_emergency_all_red() {
        let mut i = Intersection::new(small_config());
        i.tick(500); // mid NSGreen
        i.begin_emergency();
        assert_eq!(i.phase, Phase::EmergencyAllRed);
        assert_eq!(i.ns_light(), LightColor::Red);
        assert_eq!(i.ew_light(), LightColor::Red);
    }

    #[test]
    fn emergency_freezes_time() {
        let mut i = Intersection::new(small_config());
        i.begin_emergency();
        i.tick(10_000);
        assert_eq!(i.phase, Phase::EmergencyAllRed);
    }

    #[test]
    fn end_emergency_restores_saved_phase() {
        let mut i = Intersection::new(small_config());
        i.tick(300); // 300ms into NSGreen
        i.begin_emergency();
        i.tick(10_000);
        i.end_emergency();
        assert_eq!(i.phase, Phase::NSGreen);
        assert_eq!(i.phase_elapsed_ms, 300); // resumed from saved
    }

    #[test]
    fn ped_request_gated_to_allred() {
        let mut i = Intersection::new(small_config());
        i.request_ped_crossing();
        assert!(i.ped_requested);
        assert_eq!(i.ped_signal(), PedSignal::DontWalk); // not activated yet
        // advance through NSGreen + NSYellow → AllRed
        i.tick(1000); // NSYellow
        i.tick(200);  // AllRed, ped walk starts
        assert_eq!(i.ped_signal(), PedSignal::Walk);
        assert!(!i.ped_requested); // consumed
    }

    #[test]
    fn ped_transitions_walk_flashing_inactive() {
        let mut i = Intersection::new(small_config());
        i.request_ped_crossing();
        i.tick(1200); // into AllRed → ped walk
        assert_eq!(i.ped_signal(), PedSignal::Walk);
        i.tick(500); // walk 500ms done → flashing
        assert_eq!(i.ped_signal(), PedSignal::Flashing);
        i.tick(300); // flashing done → inactive
        assert_eq!(i.ped_signal(), PedSignal::DontWalk);
    }

    #[test]
    fn events_log_lifecycle() {
        let mut i = Intersection::new(small_config());
        i.tick(1500);
        let phase_changes = i.events.iter().filter(|e|
            matches!(e, IntersectionEvent::PhaseChanged { .. })).count();
        assert!(phase_changes >= 2);
    }

    #[test]
    fn emergency_during_emergency_is_noop() {
        let mut i = Intersection::new(small_config());
        i.begin_emergency();
        let before = i.events.len();
        i.begin_emergency();
        assert_eq!(i.events.len(), before);
    }

    #[test]
    fn end_emergency_without_emergency_is_noop() {
        let mut i = Intersection::new(small_config());
        let before = i.events.len();
        i.end_emergency();
        assert_eq!(i.events.len(), before);
    }
}
