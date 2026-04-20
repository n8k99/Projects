//! Event Scheduler and Calendar — intervals, recurrence, overlap detection.
//!
//! Sixth Databases project. Introduces the **half-open interval
//! `[start, end)`** — the convention every date-range API uses
//! (ISO-8601, SQL `BETWEEN` excluded via `<`, Outlook, Google Calendar).
//! Overlap between `(a_start, a_end)` and `(b_start, b_end)`:
//!   `a_start < b_end && b_start < a_end`.
//! One inequality per endpoint, total, symmetric, no off-by-one.
//!
//! Recurrence expansion: a recurring event (Daily/Weekly/Monthly) has a
//! template (start_ms, duration_ms). A query for a date range expands
//! the template into concrete occurrences whose start falls in the
//! range.
//!
//! Time is integer milliseconds from a fixed epoch. No timezones. The
//! caller does timezone math outside this engine.

use std::collections::BTreeMap;

const MS_PER_DAY: i64 = 86_400_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Recurrence { None, Daily, Weekly, Monthly }

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Event {
    pub id: u64,
    pub title: String,
    pub start_ms: i64,
    pub end_ms: i64,
    pub recurrence: Recurrence,
    pub series_end_ms: Option<i64>,  // when recurrence stops
}

impl Event {
    pub fn overlaps(&self, other_start: i64, other_end: i64) -> bool {
        self.start_ms < other_end && other_start < self.end_ms
    }

    pub fn duration_ms(&self) -> i64 { self.end_ms - self.start_ms }
}

#[derive(Debug, Clone)]
pub struct Calendar {
    pub events: BTreeMap<u64, Event>,
}

impl Calendar {
    pub fn new() -> Self { Self { events: BTreeMap::new() } }

    pub fn add(&mut self, event: Event) {
        self.events.insert(event.id, event);
    }

    pub fn get(&self, id: u64) -> Option<&Event> { self.events.get(&id) }

    /// Expand events (including recurrences) into concrete occurrences
    /// that overlap the `[from, to)` window. Returns a sorted Vec.
    pub fn events_between(&self, from_ms: i64, to_ms: i64) -> Vec<Occurrence> {
        let mut out = Vec::new();
        for event in self.events.values() {
            for occ in expand(event, from_ms, to_ms) {
                out.push(occ);
            }
        }
        out.sort_by_key(|o| (o.start_ms, o.event_id));
        out
    }

    /// Conflicts for a candidate event — returns event IDs that overlap.
    pub fn find_conflicts(&self, start_ms: i64, end_ms: i64) -> Vec<u64> {
        let mut out = Vec::new();
        for event in self.events.values() {
            for occ in expand(event, start_ms, end_ms) {
                if occ.start_ms < end_ms && start_ms < occ.end_ms {
                    out.push(occ.event_id);
                    break;  // one conflict per event is enough
                }
            }
        }
        out.sort();
        out
    }
}

impl Default for Calendar { fn default() -> Self { Self::new() } }

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Occurrence {
    pub event_id: u64,
    pub title: String,
    pub start_ms: i64,
    pub end_ms: i64,
}

fn expand(event: &Event, from_ms: i64, to_ms: i64) -> Vec<Occurrence> {
    let mut out = Vec::new();
    let duration = event.duration_ms();
    let window_cutoff = event.series_end_ms.unwrap_or(i64::MAX);

    match event.recurrence {
        Recurrence::None => {
            if event.overlaps(from_ms, to_ms) {
                out.push(Occurrence {
                    event_id: event.id,
                    title: event.title.clone(),
                    start_ms: event.start_ms,
                    end_ms: event.end_ms,
                });
            }
        }
        Recurrence::Daily => {
            push_interval_series(event, duration, MS_PER_DAY, from_ms, to_ms, window_cutoff, &mut out);
        }
        Recurrence::Weekly => {
            push_interval_series(event, duration, 7 * MS_PER_DAY, from_ms, to_ms, window_cutoff, &mut out);
        }
        Recurrence::Monthly => {
            // Monthly: 30 days per step (simplification — real calendars handle month ends).
            push_interval_series(event, duration, 30 * MS_PER_DAY, from_ms, to_ms, window_cutoff, &mut out);
        }
    }
    out
}

fn push_interval_series(
    event: &Event,
    duration: i64,
    step: i64,
    from_ms: i64,
    to_ms: i64,
    series_end: i64,
    out: &mut Vec<Occurrence>,
) {
    if step <= 0 { return; }
    // Start at event.start_ms and advance by `step` until past `to_ms` or `series_end`.
    // Fast-forward: skip occurrences that are entirely before `from_ms`.
    let mut start = event.start_ms;
    if start < from_ms {
        let gap = from_ms - start;
        let skips = (gap - duration).max(0) / step;
        start += skips * step;
    }
    while start < to_ms && start <= series_end {
        let end = start + duration;
        if end > from_ms {
            out.push(Occurrence {
                event_id: event.id,
                title: event.title.clone(),
                start_ms: start,
                end_ms: end,
            });
        }
        start += step;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helpers: days expressed in ms.
    fn day(n: i64) -> i64 { n * MS_PER_DAY }

    fn sample_calendar() -> Calendar {
        let mut cal = Calendar::new();
        cal.add(Event {
            id: 1, title: "Standup".into(),
            start_ms: day(10), end_ms: day(10) + 30 * 60_000,  // 30-minute meeting
            recurrence: Recurrence::Daily,
            series_end_ms: Some(day(30)),
        });
        cal.add(Event {
            id: 2, title: "Weekly Review".into(),
            start_ms: day(14) + 15 * 3_600_000,  // day 14, 15:00
            end_ms: day(14) + 16 * 3_600_000,
            recurrence: Recurrence::Weekly,
            series_end_ms: None,
        });
        cal.add(Event {
            id: 3, title: "One-off".into(),
            start_ms: day(12),
            end_ms: day(12) + 2 * 3_600_000,
            recurrence: Recurrence::None,
            series_end_ms: None,
        });
        cal
    }

    #[test]
    fn overlap_predicate() {
        let e = Event { id: 1, title: "x".into(),
                        start_ms: 100, end_ms: 200,
                        recurrence: Recurrence::None, series_end_ms: None };
        assert!(e.overlaps(150, 250));
        assert!(e.overlaps(50, 150));
        assert!(e.overlaps(50, 250));
        assert!(e.overlaps(120, 180));
        // Touching endpoints don't overlap (half-open intervals).
        assert!(!e.overlaps(200, 300));
        assert!(!e.overlaps(0, 100));
    }

    #[test]
    fn events_between_respects_window() {
        let cal = sample_calendar();
        let occs = cal.events_between(day(10), day(11));
        // Day 10: one daily standup + no weekly (not yet started) + no one-off
        assert_eq!(occs.len(), 1);
        assert_eq!(occs[0].event_id, 1);
    }

    #[test]
    fn daily_recurrence_expands_into_range() {
        let cal = sample_calendar();
        let occs = cal.events_between(day(10), day(15));
        // 5 days × daily standup = 5 occurrences + one-off on day 12 = 6
        let daily_count = occs.iter().filter(|o| o.event_id == 1).count();
        assert_eq!(daily_count, 5);
    }

    #[test]
    fn weekly_recurrence_generates_weekly_occurrences() {
        let cal = sample_calendar();
        let occs = cal.events_between(day(14), day(35));
        let weekly_count = occs.iter().filter(|o| o.event_id == 2).count();
        // Day 14, 21, 28 → 3 weekly occurrences
        assert_eq!(weekly_count, 3);
    }

    #[test]
    fn series_end_caps_recurrences() {
        let cal = sample_calendar();
        // Standup series_end is day 30. Query through day 40 — should stop at 30.
        let occs = cal.events_between(day(10), day(40));
        let daily_count = occs.iter().filter(|o| o.event_id == 1).count();
        // Days 10 through 30 inclusive = 21 days (series_end=day 30 is inclusive start)
        assert_eq!(daily_count, 21);
    }

    #[test]
    fn one_off_in_range() {
        let cal = sample_calendar();
        let occs = cal.events_between(day(12), day(13));
        let one_off = occs.iter().filter(|o| o.event_id == 3).count();
        assert_eq!(one_off, 1);
    }

    #[test]
    fn one_off_outside_range() {
        let cal = sample_calendar();
        let occs = cal.events_between(day(20), day(21));
        let one_off = occs.iter().filter(|o| o.event_id == 3).count();
        assert_eq!(one_off, 0);
    }

    #[test]
    fn find_conflicts_returns_overlapping_events() {
        let cal = sample_calendar();
        // Propose a meeting on day 12 from 10:00 to 12:00 — collides with one-off (also day 12).
        let start = day(12);
        let end = day(12) + 12 * 3_600_000;
        let conflicts = cal.find_conflicts(start, end);
        assert!(conflicts.contains(&3));
        assert!(conflicts.contains(&1));  // daily standup at day 12 midnight falls in this range
    }

    #[test]
    fn find_conflicts_empty_when_slot_is_free() {
        let cal = sample_calendar();
        // Day 50 is past the daily series, no weekly, no one-off.
        let start = day(50);
        let end = day(50) + 3_600_000;
        assert!(cal.find_conflicts(start, end).is_empty());
    }

    #[test]
    fn events_between_is_sorted_by_start() {
        let cal = sample_calendar();
        let occs = cal.events_between(day(10), day(20));
        for w in occs.windows(2) {
            assert!(w[0].start_ms <= w[1].start_ms);
        }
    }

    #[test]
    fn empty_range_returns_no_occurrences() {
        let cal = sample_calendar();
        let occs = cal.events_between(day(10), day(10));
        assert!(occs.is_empty());
    }

    #[test]
    fn query_before_first_event_is_empty() {
        let cal = sample_calendar();
        let occs = cal.events_between(day(0), day(5));
        assert!(occs.is_empty());
    }

    #[test]
    fn calendar_with_no_events_returns_empty() {
        let cal = Calendar::new();
        let occs = cal.events_between(day(0), day(100));
        assert!(occs.is_empty());
    }

    #[test]
    fn recurrence_series_no_end_goes_forever_in_query() {
        let cal = sample_calendar();
        // Weekly event (no series_end) — query way in the future.
        let occs = cal.events_between(day(100), day(110));
        let weekly_count = occs.iter().filter(|o| o.event_id == 2).count();
        assert!(weekly_count >= 1);
    }
}
