//! Change return program — break an amount into denominations.

use std::collections::BTreeMap;

/// US coin denominations in cents.
pub const US_COINS: &[u32] = &[25, 10, 5, 1];

/// Break amount (in cents) into fewest coins possible.
pub fn make_change(amount: u32, denominations: &[u32]) -> BTreeMap<u32, u32> {
    let mut remaining = amount;
    let mut denoms = denominations.to_vec();
    denoms.sort_unstable_by(|a, b| b.cmp(a));

    let mut result = BTreeMap::new();
    for denom in denoms {
        let count = remaining / denom;
        if count > 0 {
            result.insert(denom, count);
            remaining -= count * denom;
        }
    }
    result
}

/// An obligation with a name, amount, and due day.
#[derive(Clone)]
pub struct Obligation {
    pub name: String,
    pub amount: f64,
    pub due_day: u32,
}

/// Result of covering obligations.
pub struct CoverResult {
    pub covered: Vec<Obligation>,
    pub uncovered: Vec<(Obligation, f64)>,  // (obligation, shortfall)
    pub remaining: f64,
    pub all_covered: bool,
}

/// Cover obligations in chronological order with available funds.
pub fn cover_obligations(available: f64, obligations: &[Obligation]) -> CoverResult {
    let mut sorted = obligations.to_vec();
    sorted.sort_by_key(|o| o.due_day);

    let mut remaining = available;
    let mut covered = Vec::new();
    let mut uncovered = Vec::new();

    for ob in sorted {
        if remaining >= ob.amount {
            remaining -= ob.amount;
            covered.push(ob);
        } else {
            let shortfall = ob.amount - remaining;
            uncovered.push((ob, shortfall));
        }
    }

    let all_covered = uncovered.is_empty();
    CoverResult { covered, uncovered, remaining, all_covered }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_make_change() {
        let result = make_change(87, US_COINS);
        assert_eq!(result[&25], 3);
        assert_eq!(result[&10], 1);
        assert_eq!(result[&1], 2);
    }
}
