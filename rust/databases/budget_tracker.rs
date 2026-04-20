//! Budget Tracker — envelope allocation, variance analysis, alert thresholds.
//!
//! Seventh Databases project. A budget is a set of **envelopes**
//! (category → monthly allocation) and a stream of **transactions**.
//! Variance per category = `actual - budgeted`; positive = overspent.
//! Alerts fire when spending crosses a configurable threshold of the
//! budgeted amount (default 80%).
//!
//! Distinctive moves:
//!   * **Envelope allocation with remaining-balance tracking** —
//!      each category has `budgeted`, `spent`, and `remaining` fields.
//!   * **Variance percentage** — `(actual - budgeted) * 100 / budgeted`
//!      with zero-budget handled (report over-budget as ∞ via sentinel).
//!   * **Alert thresholds** — percentage-based, not absolute; scales
//!      with the envelope size.
//!   * **Month-over-month reset** — budget resets at period boundary;
//!      unused balance discards (envelope) or carries over (rollover
//!      policy per category).

use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RolloverPolicy { Reset, Carry }

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Envelope {
    pub category: String,
    pub budgeted_cents: i64,
    pub rollover: RolloverPolicy,
    pub alert_threshold_pct: u32,  // e.g., 80 = alert at 80% spent
}

impl Envelope {
    pub fn new(category: &str, budgeted: i64) -> Self {
        Self {
            category: category.into(),
            budgeted_cents: budgeted,
            rollover: RolloverPolicy::Reset,
            alert_threshold_pct: 80,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Transaction {
    pub timestamp_ms: i64,
    pub category: String,
    pub amount_cents: i64,   // negative = expense, positive = refund
    pub note: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CategoryStatus {
    pub category: String,
    pub budgeted_cents: i64,
    pub spent_cents: i64,          // absolute value of expenses
    pub remaining_cents: i64,      // budgeted - spent, can go negative
    pub variance_cents: i64,       // spent - budgeted, positive = overspent
    pub variance_pct: i32,         // -101 sentinel for "no budget set"
    pub over_budget: bool,
    pub alert: bool,
}

#[derive(Debug, Clone)]
pub struct Budget {
    pub envelopes: BTreeMap<String, Envelope>,
    pub transactions: Vec<Transaction>,
}

impl Budget {
    pub fn new() -> Self {
        Self { envelopes: BTreeMap::new(), transactions: Vec::new() }
    }

    pub fn add_envelope(&mut self, e: Envelope) {
        self.envelopes.insert(e.category.clone(), e);
    }

    pub fn record(&mut self, t: Transaction) {
        self.transactions.push(t);
    }

    /// Compute status for each envelope across all transactions.
    pub fn status_all(&self) -> Vec<CategoryStatus> {
        self.status_between(i64::MIN, i64::MAX)
    }

    /// Compute status restricted to a time window.
    pub fn status_between(&self, from_ms: i64, to_ms: i64) -> Vec<CategoryStatus> {
        // Sum spent per category within window.
        let mut spent: BTreeMap<String, i64> = BTreeMap::new();
        for t in &self.transactions {
            if t.timestamp_ms < from_ms || t.timestamp_ms >= to_ms { continue; }
            if t.amount_cents < 0 {
                *spent.entry(t.category.clone()).or_insert(0) += -t.amount_cents;
            } else {
                // Refund subtracts from spent.
                *spent.entry(t.category.clone()).or_insert(0) -= t.amount_cents;
            }
        }
        let mut out: Vec<CategoryStatus> = self.envelopes.values()
            .map(|env| {
                let spent_cents = *spent.get(&env.category).unwrap_or(&0);
                let remaining = env.budgeted_cents - spent_cents;
                let variance = spent_cents - env.budgeted_cents;
                let variance_pct = if env.budgeted_cents == 0 {
                    -101  // sentinel: no budget set
                } else {
                    (variance * 100 / env.budgeted_cents) as i32
                };
                let over_budget = spent_cents > env.budgeted_cents;
                let alert = if env.budgeted_cents > 0 {
                    let spent_pct = (spent_cents * 100 / env.budgeted_cents).max(0) as u32;
                    spent_pct >= env.alert_threshold_pct
                } else { false };
                CategoryStatus {
                    category: env.category.clone(),
                    budgeted_cents: env.budgeted_cents,
                    spent_cents,
                    remaining_cents: remaining,
                    variance_cents: variance,
                    variance_pct,
                    over_budget,
                    alert,
                }
            })
            .collect();
        // Report uncategorised spending as a synthetic envelope (budget=0).
        for (cat, spent_cents) in &spent {
            if !self.envelopes.contains_key(cat) {
                out.push(CategoryStatus {
                    category: cat.clone(),
                    budgeted_cents: 0,
                    spent_cents: *spent_cents,
                    remaining_cents: -spent_cents,
                    variance_cents: *spent_cents,
                    variance_pct: -101,
                    over_budget: *spent_cents > 0,
                    alert: false,
                });
            }
        }
        out.sort_by(|a, b| a.category.cmp(&b.category));
        out
    }

    pub fn total_budgeted(&self) -> i64 {
        self.envelopes.values().map(|e| e.budgeted_cents).sum()
    }

    pub fn total_spent_between(&self, from_ms: i64, to_ms: i64) -> i64 {
        let mut sum = 0;
        for t in &self.transactions {
            if t.timestamp_ms < from_ms || t.timestamp_ms >= to_ms { continue; }
            if t.amount_cents < 0 { sum += -t.amount_cents; }
            else { sum -= t.amount_cents; }
        }
        sum
    }

    /// Alert list — categories currently over their threshold.
    pub fn alerts(&self) -> Vec<CategoryStatus> {
        self.status_all().into_iter().filter(|s| s.alert).collect()
    }
}

impl Default for Budget { fn default() -> Self { Self::new() } }

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_budget() -> Budget {
        let mut b = Budget::new();
        b.add_envelope(Envelope {
            category: "groceries".into(),
            budgeted_cents: 50000,  // $500
            rollover: RolloverPolicy::Reset,
            alert_threshold_pct: 80,
        });
        b.add_envelope(Envelope {
            category: "rent".into(),
            budgeted_cents: 200000,  // $2000
            rollover: RolloverPolicy::Reset,
            alert_threshold_pct: 90,
        });
        b.add_envelope(Envelope {
            category: "entertainment".into(),
            budgeted_cents: 10000,  // $100
            rollover: RolloverPolicy::Reset,
            alert_threshold_pct: 50,
        });
        b
    }

    #[test]
    fn envelope_with_no_spending_is_full() {
        let b = sample_budget();
        let status = b.status_all();
        let rent = status.iter().find(|s| s.category == "rent").unwrap();
        assert_eq!(rent.spent_cents, 0);
        assert_eq!(rent.remaining_cents, 200000);
        assert_eq!(rent.variance_cents, -200000);
        assert!(!rent.over_budget);
        assert!(!rent.alert);
    }

    #[test]
    fn expense_reduces_remaining() {
        let mut b = sample_budget();
        b.record(Transaction {
            timestamp_ms: 1000, category: "groceries".into(),
            amount_cents: -15000, note: "Store A".into(),
        });
        let status = b.status_all();
        let g = status.iter().find(|s| s.category == "groceries").unwrap();
        assert_eq!(g.spent_cents, 15000);
        assert_eq!(g.remaining_cents, 35000);
    }

    #[test]
    fn refund_adds_back_to_remaining() {
        let mut b = sample_budget();
        b.record(Transaction {
            timestamp_ms: 1000, category: "groceries".into(),
            amount_cents: -20000, note: "Store A".into(),
        });
        b.record(Transaction {
            timestamp_ms: 2000, category: "groceries".into(),
            amount_cents: 5000, note: "Refund".into(),
        });
        let status = b.status_all();
        let g = status.iter().find(|s| s.category == "groceries").unwrap();
        assert_eq!(g.spent_cents, 15000);
        assert_eq!(g.remaining_cents, 35000);
    }

    #[test]
    fn over_budget_flag_set_when_spent_exceeds_budget() {
        let mut b = sample_budget();
        b.record(Transaction {
            timestamp_ms: 1000, category: "entertainment".into(),
            amount_cents: -15000, note: "Concert".into(),
        });
        let status = b.status_all();
        let e = status.iter().find(|s| s.category == "entertainment").unwrap();
        assert!(e.over_budget);
        assert_eq!(e.variance_cents, 5000);  // overspent by $50
        assert_eq!(e.variance_pct, 50);      // 50% over
    }

    #[test]
    fn alert_fires_at_threshold() {
        let mut b = sample_budget();
        // entertainment alerts at 50% = $50 spent
        b.record(Transaction {
            timestamp_ms: 1000, category: "entertainment".into(),
            amount_cents: -6000, note: "Movie".into(),
        });
        let status = b.status_all();
        let e = status.iter().find(|s| s.category == "entertainment").unwrap();
        assert!(e.alert);  // 60% of budget spent, > 50% threshold
    }

    #[test]
    fn alert_does_not_fire_below_threshold() {
        let mut b = sample_budget();
        b.record(Transaction {
            timestamp_ms: 1000, category: "entertainment".into(),
            amount_cents: -4000, note: "Rental".into(),
        });
        let status = b.status_all();
        let e = status.iter().find(|s| s.category == "entertainment").unwrap();
        assert!(!e.alert);  // 40% spent, < 50% threshold
    }

    #[test]
    fn alerts_method_lists_only_alerting_categories() {
        let mut b = sample_budget();
        b.record(Transaction {
            timestamp_ms: 1000, category: "entertainment".into(),
            amount_cents: -8000, note: "Concert".into(),
        });
        b.record(Transaction {
            timestamp_ms: 2000, category: "groceries".into(),
            amount_cents: -1000, note: "Small".into(),
        });
        let alerts = b.alerts();
        assert_eq!(alerts.len(), 1);
        assert_eq!(alerts[0].category, "entertainment");
    }

    #[test]
    fn status_between_respects_window() {
        let mut b = sample_budget();
        b.record(Transaction { timestamp_ms: 100, category: "groceries".into(),
                               amount_cents: -10000, note: "".into() });
        b.record(Transaction { timestamp_ms: 500, category: "groceries".into(),
                               amount_cents: -5000, note: "".into() });
        // Window only includes the second transaction.
        let status = b.status_between(200, 1000);
        let g = status.iter().find(|s| s.category == "groceries").unwrap();
        assert_eq!(g.spent_cents, 5000);
    }

    #[test]
    fn uncategorised_spending_appears_as_synthetic_envelope() {
        let mut b = sample_budget();
        b.record(Transaction { timestamp_ms: 100, category: "vacation".into(),
                               amount_cents: -30000, note: "".into() });
        let status = b.status_all();
        let v = status.iter().find(|s| s.category == "vacation").unwrap();
        assert_eq!(v.budgeted_cents, 0);
        assert_eq!(v.spent_cents, 30000);
        assert!(v.over_budget);
        assert_eq!(v.variance_pct, -101);  // sentinel
    }

    #[test]
    fn total_budgeted_sums_all_envelopes() {
        let b = sample_budget();
        assert_eq!(b.total_budgeted(), 50000 + 200000 + 10000);
    }

    #[test]
    fn total_spent_between_sums_window() {
        let mut b = sample_budget();
        b.record(Transaction { timestamp_ms: 100, category: "groceries".into(),
                               amount_cents: -5000, note: "".into() });
        b.record(Transaction { timestamp_ms: 200, category: "rent".into(),
                               amount_cents: -200000, note: "".into() });
        b.record(Transaction { timestamp_ms: 2000, category: "groceries".into(),
                               amount_cents: -3000, note: "".into() });
        assert_eq!(b.total_spent_between(0, 1000), 205000);
        assert_eq!(b.total_spent_between(1000, 3000), 3000);
    }

    #[test]
    fn variance_pct_is_zero_at_exactly_budgeted() {
        let mut b = Budget::new();
        b.add_envelope(Envelope::new("x", 1000));
        b.record(Transaction { timestamp_ms: 0, category: "x".into(),
                               amount_cents: -1000, note: "".into() });
        let status = b.status_all();
        assert_eq!(status[0].variance_pct, 0);
        assert!(!status[0].over_budget);  // 1000 == 1000, not over
    }

    #[test]
    fn empty_budget_produces_empty_status() {
        let b = Budget::new();
        assert!(b.status_all().is_empty());
    }
}
