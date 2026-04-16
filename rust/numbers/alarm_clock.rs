//! Alarm Clock — schedule and trigger time-based alerts.

/// A single alarm entry.
#[derive(Debug, Clone, PartialEq)]
pub struct Alarm {
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
    pub label: String,
}

/// Manages a collection of time-based alarms.
///
/// The `check` method takes an explicit "now" tuple `(h, m, s)` so that
/// alarm logic is deterministic and testable without real clocks.
pub struct AlarmClock {
    alarms: Vec<Alarm>,
}

impl AlarmClock {
    /// Create an empty alarm clock.
    pub fn new() -> Self {
        AlarmClock { alarms: Vec::new() }
    }

    /// Schedule an alarm for the given time with an optional label.
    pub fn set_alarm(&mut self, hour: u8, minute: u8, second: u8, label: &str) {
        assert!(hour < 24 && minute < 60 && second < 60,
                "invalid time: {:02}:{:02}:{:02}", hour, minute, second);
        self.alarms.push(Alarm {
            hour,
            minute,
            second,
            label: label.to_string(),
        });
    }

    /// Check all pending alarms against the given time `(h, m, s)`.
    ///
    /// Returns the list of alarms that have triggered (time <= now).
    /// Triggered alarms are removed from the pending list.
    pub fn check(&mut self, now: (u8, u8, u8)) -> Vec<Alarm> {
        let now_secs = now.0 as u32 * 3600 + now.1 as u32 * 60 + now.2 as u32;
        let mut triggered = Vec::new();
        let mut remaining = Vec::new();

        for alarm in self.alarms.drain(..) {
            let alarm_secs = alarm.hour as u32 * 3600
                + alarm.minute as u32 * 60
                + alarm.second as u32;
            if now_secs >= alarm_secs {
                triggered.push(alarm);
            } else {
                remaining.push(alarm);
            }
        }
        self.alarms = remaining;
        triggered
    }

    /// Return a slice of all pending (not yet triggered) alarms.
    pub fn pending(&self) -> &[Alarm] {
        &self.alarms
    }

    /// Cancel the first pending alarm with the given label.
    ///
    /// Returns `true` if an alarm was removed, `false` otherwise.
    pub fn cancel(&mut self, label: &str) -> bool {
        if let Some(pos) = self.alarms.iter().position(|a| a.label == label) {
            self.alarms.remove(pos);
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_and_pending() {
        let mut clock = AlarmClock::new();
        clock.set_alarm(7, 0, 0, "morning");
        clock.set_alarm(12, 0, 0, "noon");
        assert_eq!(clock.pending().len(), 2);
    }

    #[test]
    fn check_triggers_past_alarms() {
        let mut clock = AlarmClock::new();
        clock.set_alarm(6, 0, 0, "early");
        clock.set_alarm(23, 59, 59, "late");
        let triggered = clock.check((12, 0, 0));
        assert_eq!(triggered.len(), 1);
        assert_eq!(triggered[0].label, "early");
        assert_eq!(clock.pending().len(), 1);
    }

    #[test]
    fn check_triggers_exact_time() {
        let mut clock = AlarmClock::new();
        clock.set_alarm(8, 30, 0, "standup");
        let triggered = clock.check((8, 30, 0));
        assert_eq!(triggered.len(), 1);
        assert_eq!(triggered[0].label, "standup");
    }

    #[test]
    fn cancel_existing() {
        let mut clock = AlarmClock::new();
        clock.set_alarm(7, 0, 0, "morning");
        assert!(clock.cancel("morning"));
        assert_eq!(clock.pending().len(), 0);
    }

    #[test]
    fn cancel_nonexistent() {
        let mut clock = AlarmClock::new();
        clock.set_alarm(7, 0, 0, "morning");
        assert!(!clock.cancel("evening"));
        assert_eq!(clock.pending().len(), 1);
    }

    #[test]
    fn check_removes_triggered() {
        let mut clock = AlarmClock::new();
        clock.set_alarm(0, 0, 0, "midnight");
        let first = clock.check((12, 0, 0));
        assert_eq!(first.len(), 1);
        let second = clock.check((12, 0, 0));
        assert_eq!(second.len(), 0);
    }

    #[test]
    #[should_panic(expected = "invalid time")]
    fn invalid_hour() {
        let mut clock = AlarmClock::new();
        clock.set_alarm(25, 0, 0, "bad");
    }
}
