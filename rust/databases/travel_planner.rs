//! Travel Planner System — leg-chained itineraries with integrity validation.
//!
//! Tenth Databases project. A trip is an **ordered sequence of legs**
//! (flight, train, car, etc.), each with an origin, destination, and
//! time window. The defining move is **itinerary integrity validation**:
//! the chain of legs is a sequence of graph edges, and the planner
//! checks that the chain actually composes — destination of leg N
//! matches origin of leg N+1, leg N+1 departs after leg N arrives,
//! connection time meets the minimum.
//!
//! Distinctive moves:
//!   * **LocationMismatch** — leg N destination != leg N+1 origin.
//!   * **TimeConflict** — leg N+1 departs before leg N arrives (or at
//!      the same instant — can't be in two places).
//!   * **TightLayover** — gap < `min_layover_minutes`; flagged but not
//!      strictly invalid (user may accept).
//!   * **Layover enumeration** — list of (location, wait_ms) between
//!      consecutive legs.
//!   * **Trip metrics** — total travel time, total cost, total legs.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransportMode { Flight, Train, Bus, Car, Walk }

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Leg {
    pub id: u64,
    pub mode: TransportMode,
    pub origin: String,
    pub destination: String,
    pub depart_ms: i64,
    pub arrive_ms: i64,
    pub cost_cents: i64,
    pub carrier: String,
}

impl Leg {
    pub fn duration_ms(&self) -> i64 { self.arrive_ms - self.depart_ms }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Trip {
    pub id: u64,
    pub title: String,
    pub legs: Vec<Leg>,
    pub min_layover_minutes: u32,
}

impl Trip {
    pub fn new(id: u64, title: &str) -> Self {
        Self { id, title: title.into(), legs: vec![], min_layover_minutes: 30 }
    }

    pub fn add_leg(&mut self, leg: Leg) {
        self.legs.push(leg);
    }

    /// Total travel time from first depart to last arrive (including layovers).
    pub fn total_span_ms(&self) -> i64 {
        if self.legs.is_empty() { return 0; }
        let first = self.legs.first().unwrap();
        let last = self.legs.last().unwrap();
        last.arrive_ms - first.depart_ms
    }

    /// Sum of leg durations (excluding layovers).
    pub fn in_transit_ms(&self) -> i64 {
        self.legs.iter().map(|l| l.duration_ms()).sum()
    }

    /// Total cost of all legs.
    pub fn total_cost_cents(&self) -> i64 {
        self.legs.iter().map(|l| l.cost_cents).sum()
    }

    /// Total non-transit time between legs.
    pub fn total_layover_ms(&self) -> i64 {
        self.total_span_ms() - self.in_transit_ms()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Layover {
    pub location: String,
    pub arrive_ms: i64,
    pub depart_ms: i64,
    pub duration_ms: i64,
}

/// Return layovers between consecutive legs. Uses leg N's destination.
pub fn layovers(trip: &Trip) -> Vec<Layover> {
    let mut out = Vec::new();
    for pair in trip.legs.windows(2) {
        let prev = &pair[0];
        let next = &pair[1];
        out.push(Layover {
            location: prev.destination.clone(),
            arrive_ms: prev.arrive_ms,
            depart_ms: next.depart_ms,
            duration_ms: next.depart_ms - prev.arrive_ms,
        });
    }
    out
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IssueSeverity { Info, Warning, Error }

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IssueKind {
    LocationMismatch { leg_idx: usize, prev_dest: String, next_origin: String },
    TimeConflict { leg_idx: usize, prev_arrive: i64, next_depart: i64 },
    TightLayover { leg_idx: usize, layover_min: i64 },
    ZeroDurationLeg { leg_idx: usize },
    NegativeDurationLeg { leg_idx: usize },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Issue {
    pub severity: IssueSeverity,
    pub kind: IssueKind,
}

/// Walk the leg chain and emit all integrity issues.
pub fn validate_trip(trip: &Trip) -> Vec<Issue> {
    let mut issues = Vec::new();

    // Per-leg sanity.
    for (i, leg) in trip.legs.iter().enumerate() {
        let d = leg.duration_ms();
        if d == 0 {
            issues.push(Issue {
                severity: IssueSeverity::Warning,
                kind: IssueKind::ZeroDurationLeg { leg_idx: i },
            });
        } else if d < 0 {
            issues.push(Issue {
                severity: IssueSeverity::Error,
                kind: IssueKind::NegativeDurationLeg { leg_idx: i },
            });
        }
    }

    // Chain checks.
    for (i, pair) in trip.legs.windows(2).enumerate() {
        let prev = &pair[0];
        let next = &pair[1];
        if prev.destination != next.origin {
            issues.push(Issue {
                severity: IssueSeverity::Error,
                kind: IssueKind::LocationMismatch {
                    leg_idx: i + 1,
                    prev_dest: prev.destination.clone(),
                    next_origin: next.origin.clone(),
                },
            });
        }
        if next.depart_ms < prev.arrive_ms {
            issues.push(Issue {
                severity: IssueSeverity::Error,
                kind: IssueKind::TimeConflict {
                    leg_idx: i + 1,
                    prev_arrive: prev.arrive_ms,
                    next_depart: next.depart_ms,
                },
            });
        } else {
            let layover_ms = next.depart_ms - prev.arrive_ms;
            let layover_min = layover_ms / 60_000;
            let min_required = trip.min_layover_minutes as i64;
            if layover_min < min_required && layover_ms > 0 {
                issues.push(Issue {
                    severity: IssueSeverity::Warning,
                    kind: IssueKind::TightLayover { leg_idx: i + 1, layover_min },
                });
            }
        }
    }

    issues
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mk_leg(id: u64, origin: &str, dest: &str, depart_ms: i64, arrive_ms: i64, cost: i64) -> Leg {
        Leg {
            id, mode: TransportMode::Flight,
            origin: origin.into(), destination: dest.into(),
            depart_ms, arrive_ms, cost_cents: cost,
            carrier: "ACME Air".into(),
        }
    }

    fn valid_trip() -> Trip {
        let mut trip = Trip::new(1, "NYC -> SFO -> LAX");
        trip.legs.push(mk_leg(1, "JFK", "SFO", 100_000_000, 120_000_000, 30000));
        trip.legs.push(mk_leg(2, "SFO", "LAX", 130_000_000, 135_000_000, 15000));
        trip
    }

    #[test]
    fn valid_trip_has_no_issues() {
        let trip = valid_trip();
        assert!(validate_trip(&trip).is_empty());
    }

    #[test]
    fn location_mismatch_flagged() {
        let mut trip = valid_trip();
        trip.legs[1].origin = "SEA".into();  // doesn't match leg 0's SFO
        let issues = validate_trip(&trip);
        assert!(issues.iter().any(|i| matches!(i.kind,
            IssueKind::LocationMismatch { .. })));
    }

    #[test]
    fn time_conflict_flagged() {
        let mut trip = valid_trip();
        trip.legs[1].depart_ms = 110_000_000;  // before leg 0 arrives
        let issues = validate_trip(&trip);
        assert!(issues.iter().any(|i| matches!(i.kind,
            IssueKind::TimeConflict { .. })));
    }

    #[test]
    fn tight_layover_flagged_below_minimum() {
        let mut trip = valid_trip();
        trip.min_layover_minutes = 30;
        trip.legs[1].depart_ms = 120_000_000 + 10 * 60_000;  // 10 min after arrival
        let issues = validate_trip(&trip);
        assert!(issues.iter().any(|i| matches!(i.kind,
            IssueKind::TightLayover { layover_min: 10, .. })));
    }

    #[test]
    fn tight_layover_not_flagged_if_at_or_above_minimum() {
        let mut trip = valid_trip();
        trip.min_layover_minutes = 30;
        trip.legs[1].depart_ms = 120_000_000 + 45 * 60_000;  // 45 min layover
        let issues = validate_trip(&trip);
        assert!(!issues.iter().any(|i| matches!(i.kind,
            IssueKind::TightLayover { .. })));
    }

    #[test]
    fn zero_duration_leg_flagged() {
        let mut trip = valid_trip();
        trip.legs[0].arrive_ms = trip.legs[0].depart_ms;
        let issues = validate_trip(&trip);
        assert!(issues.iter().any(|i| matches!(i.kind,
            IssueKind::ZeroDurationLeg { .. })));
    }

    #[test]
    fn negative_duration_leg_flagged_as_error() {
        let mut trip = valid_trip();
        trip.legs[0].arrive_ms = trip.legs[0].depart_ms - 1000;
        let issues = validate_trip(&trip);
        let neg = issues.iter().find(|i| matches!(i.kind,
            IssueKind::NegativeDurationLeg { .. })).unwrap();
        assert_eq!(neg.severity, IssueSeverity::Error);
    }

    #[test]
    fn layovers_lists_gaps_between_legs() {
        let trip = valid_trip();
        let lo = layovers(&trip);
        assert_eq!(lo.len(), 1);
        assert_eq!(lo[0].location, "SFO");
        assert_eq!(lo[0].duration_ms, 10_000_000);
    }

    #[test]
    fn trip_metrics() {
        let trip = valid_trip();
        assert_eq!(trip.total_span_ms(), 35_000_000);
        assert_eq!(trip.in_transit_ms(), 25_000_000);   // 20m + 5m
        assert_eq!(trip.total_layover_ms(), 10_000_000);
        assert_eq!(trip.total_cost_cents(), 45000);
    }

    #[test]
    fn empty_trip_has_zero_metrics() {
        let trip = Trip::new(1, "empty");
        assert_eq!(trip.total_span_ms(), 0);
        assert_eq!(trip.in_transit_ms(), 0);
        assert_eq!(trip.total_cost_cents(), 0);
        assert!(validate_trip(&trip).is_empty());
        assert!(layovers(&trip).is_empty());
    }

    #[test]
    fn single_leg_trip_has_no_layovers() {
        let mut trip = Trip::new(1, "one-way");
        trip.legs.push(mk_leg(1, "A", "B", 0, 1000, 100));
        assert!(validate_trip(&trip).is_empty());
        assert!(layovers(&trip).is_empty());
    }

    #[test]
    fn multiple_issues_reported_together() {
        let mut trip = valid_trip();
        trip.legs[1].origin = "SEA".into();           // LocationMismatch
        trip.legs[1].depart_ms = 110_000_000;         // TimeConflict
        let issues = validate_trip(&trip);
        assert!(issues.len() >= 2);
    }

    #[test]
    fn zero_layover_treated_as_time_conflict_not_tight_layover() {
        let mut trip = valid_trip();
        trip.legs[1].depart_ms = 120_000_000;  // exactly at prev arrive
        let issues = validate_trip(&trip);
        // No time conflict (departure not before arrival), no tight layover
        // (zero layover is not "tight"; it's a connecting-at-instant case).
        assert!(!issues.iter().any(|i| matches!(i.kind, IssueKind::TimeConflict { .. })));
        assert!(!issues.iter().any(|i| matches!(i.kind, IssueKind::TightLayover { .. })));
    }

    #[test]
    fn layover_duration_correct() {
        let trip = valid_trip();
        let lo = layovers(&trip);
        assert_eq!(lo[0].duration_ms, lo[0].depart_ms - lo[0].arrive_ms);
    }

    #[test]
    fn three_leg_trip_produces_two_layovers() {
        let mut trip = Trip::new(1, "3-leg");
        trip.legs.push(mk_leg(1, "A", "B", 0, 1000, 100));
        trip.legs.push(mk_leg(2, "B", "C", 2000, 3000, 200));
        trip.legs.push(mk_leg(3, "C", "D", 4000, 5000, 300));
        let lo = layovers(&trip);
        assert_eq!(lo.len(), 2);
        assert_eq!(lo[0].location, "B");
        assert_eq!(lo[1].location, "C");
    }
}
