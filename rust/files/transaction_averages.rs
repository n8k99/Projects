//! Transaction Averages — grouped aggregation over a stream of records.
//!
//! Fifth Files project. G088 gave us tabular records and sort; G089 adds
//! **group-by aggregation**. Every transaction has (date, category, amount);
//! the aggregator groups by a key function and computes per-group stats
//! (count, sum, mean, min, max).
//!
//! Core pattern: one pass over records, a hash map accumulates per-group
//! running sums, a second pass finalises the means. Deterministic output
//! order comes from sorting group keys before emission — maps don't
//! iterate in a stable order, but tests and diffs need one.
//!
//! Amounts are integer cents throughout — no floats until the final mean,
//! where we compute `sum_cents / count` as a float. This keeps the
//! accumulation exact; only the reporting step loses precision, and
//! only to a bounded amount.

use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Transaction {
    pub date: String,     // ISO: YYYY-MM-DD
    pub category: String,
    pub amount_cents: i64,
}

impl Transaction {
    pub fn month(&self) -> String {
        if self.date.len() >= 7 { self.date[..7].to_string() }
        else { self.date.clone() }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Stats {
    pub count: u32,
    pub sum_cents: i64,
    pub mean_cents: f64,
    pub min_cents: i64,
    pub max_cents: i64,
}

impl Stats {
    pub fn new() -> Self {
        Self { count: 0, sum_cents: 0, mean_cents: 0.0,
               min_cents: i64::MAX, max_cents: i64::MIN }
    }

    pub fn observe(&mut self, amount: i64) {
        self.count += 1;
        self.sum_cents += amount;
        if amount < self.min_cents { self.min_cents = amount; }
        if amount > self.max_cents { self.max_cents = amount; }
    }

    pub fn finalise(&mut self) {
        self.mean_cents = if self.count == 0 { 0.0 }
            else { self.sum_cents as f64 / self.count as f64 };
    }
}

impl Default for Stats { fn default() -> Self { Self::new() } }

/// Parse a line of `date,category,amount_cents`.
pub fn parse_transaction(line: &str) -> Option<Transaction> {
    let mut parts = line.splitn(3, ',');
    let date = parts.next()?.trim().to_string();
    let category = parts.next()?.trim().to_string();
    let amount_cents: i64 = parts.next()?.trim().parse().ok()?;
    if date.is_empty() || category.is_empty() { return None; }
    Some(Transaction { date, category, amount_cents })
}

pub fn parse_transactions(text: &str) -> Vec<Transaction> {
    text.lines()
        .filter(|l| !l.is_empty())
        .filter_map(parse_transaction)
        .collect()
}

/// Group transactions by a key function, returning sorted-by-key group stats.
pub fn group_by<F>(txns: &[Transaction], key_fn: F) -> Vec<(String, Stats)>
where F: Fn(&Transaction) -> String {
    let mut groups: HashMap<String, Stats> = HashMap::new();
    for t in txns {
        let k = key_fn(t);
        groups.entry(k).or_default().observe(t.amount_cents);
    }
    let mut out: Vec<(String, Stats)> = groups.into_iter()
        .map(|(k, mut v)| { v.finalise(); (k, v) })
        .collect();
    out.sort_by(|a, b| a.0.cmp(&b.0));
    out
}

/// Group by category.
pub fn by_category(txns: &[Transaction]) -> Vec<(String, Stats)> {
    group_by(txns, |t| t.category.clone())
}

/// Group by YYYY-MM month prefix.
pub fn by_month(txns: &[Transaction]) -> Vec<(String, Stats)> {
    group_by(txns, |t| t.month())
}

/// Total stats across all transactions.
pub fn total(txns: &[Transaction]) -> Stats {
    let mut s = Stats::new();
    for t in txns { s.observe(t.amount_cents); }
    s.finalise();
    s
}

/// Top-N groups by mean (descending).
pub fn top_n_by_mean(groups: &[(String, Stats)], n: usize) -> Vec<(String, Stats)> {
    let mut sorted: Vec<(String, Stats)> = groups.iter().cloned().collect();
    sorted.sort_by(|a, b| b.1.mean_cents.partial_cmp(&a.1.mean_cents)
        .unwrap_or(std::cmp::Ordering::Equal));
    sorted.into_iter().take(n).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Vec<Transaction> {
        parse_transactions(
            "2026-01-03,groceries,4500\n\
             2026-01-10,groceries,3200\n\
             2026-01-12,rent,120000\n\
             2026-02-01,rent,120000\n\
             2026-02-14,groceries,5500\n\
             2026-02-22,books,2200\n"
        )
    }

    #[test]
    fn parse_skips_empty_lines() {
        let t = parse_transactions("\n\n2026-01-01,x,100\n\n");
        assert_eq!(t.len(), 1);
    }

    #[test]
    fn parse_rejects_malformed() {
        assert!(parse_transaction("nope").is_none());
        assert!(parse_transaction("2026,only-two").is_none());
        assert!(parse_transaction("2026-01-01,cat,not_a_number").is_none());
    }

    #[test]
    fn stats_for_single_observation() {
        let mut s = Stats::new();
        s.observe(500);
        s.finalise();
        assert_eq!(s.count, 1);
        assert_eq!(s.sum_cents, 500);
        assert_eq!(s.mean_cents, 500.0);
        assert_eq!(s.min_cents, 500);
        assert_eq!(s.max_cents, 500);
    }

    #[test]
    fn by_category_groups_correctly() {
        let t = sample();
        let g = by_category(&t);
        let names: Vec<&String> = g.iter().map(|(k, _)| k).collect();
        assert_eq!(names, vec!["books", "groceries", "rent"]);
        let groceries = g.iter().find(|(k, _)| k == "groceries").unwrap();
        assert_eq!(groceries.1.count, 3);
        assert_eq!(groceries.1.sum_cents, 13200);
        assert!((groceries.1.mean_cents - 4400.0).abs() < 0.01);
    }

    #[test]
    fn by_month_groups_correctly() {
        let t = sample();
        let g = by_month(&t);
        assert_eq!(g.len(), 2);
        let jan = g.iter().find(|(k, _)| k == "2026-01").unwrap();
        assert_eq!(jan.1.count, 3);
        assert_eq!(jan.1.sum_cents, 127700);
    }

    #[test]
    fn group_output_is_sorted_by_key() {
        let t = sample();
        let g = by_category(&t);
        for window in g.windows(2) {
            assert!(window[0].0 < window[1].0);
        }
    }

    #[test]
    fn total_across_all() {
        let t = sample();
        let s = total(&t);
        assert_eq!(s.count, 6);
        assert_eq!(s.sum_cents, 255400);
    }

    #[test]
    fn top_n_by_mean_returns_highest_means() {
        let t = sample();
        let g = by_category(&t);
        let top = top_n_by_mean(&g, 2);
        assert_eq!(top.len(), 2);
        // rent (120000 mean) > groceries (4400) > books (2200)
        assert_eq!(top[0].0, "rent");
        assert_eq!(top[1].0, "groceries");
    }

    #[test]
    fn min_max_over_group() {
        let t = sample();
        let g = by_category(&t);
        let groceries = g.iter().find(|(k, _)| k == "groceries").unwrap();
        assert_eq!(groceries.1.min_cents, 3200);
        assert_eq!(groceries.1.max_cents, 5500);
    }

    #[test]
    fn empty_input_produces_empty_groups() {
        let t: Vec<Transaction> = vec![];
        assert!(by_category(&t).is_empty());
        assert!(by_month(&t).is_empty());
        let s = total(&t);
        assert_eq!(s.count, 0);
    }

    #[test]
    fn single_transaction_appears_in_category_and_month() {
        let t = parse_transactions("2026-03-01,salary,500000\n");
        let cats = by_category(&t);
        let months = by_month(&t);
        assert_eq!(cats.len(), 1);
        assert_eq!(months.len(), 1);
        assert_eq!(cats[0].1.count, 1);
        assert_eq!(months[0].1.count, 1);
    }

    #[test]
    fn month_extraction_from_date() {
        let tx = Transaction { date: "2026-04-20".into(), category: "x".into(), amount_cents: 100 };
        assert_eq!(tx.month(), "2026-04");
    }

    #[test]
    fn custom_group_key() {
        let t = sample();
        // Group by "year" (first 4 chars).
        let by_year = group_by(&t, |tx| tx.date[..4].to_string());
        assert_eq!(by_year.len(), 1);  // all 2026
        assert_eq!(by_year[0].0, "2026");
        assert_eq!(by_year[0].1.count, 6);
    }
}
