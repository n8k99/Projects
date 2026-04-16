//! Tax Calculator — progressive income tax computation.

/// Tax detail for a single bracket.
pub struct BracketTax {
    pub bracket_floor: f64,
    pub rate: f64,
    pub taxable: f64,
    pub tax: f64,
}

/// Result of a progressive tax calculation.
pub struct TaxResult {
    pub total_tax: f64,
    pub effective_rate: f64,
    pub marginal_rate: f64,
    pub breakdown: Vec<BracketTax>,
}

/// 2024 US federal single-filer brackets: (upper_limit, rate).
const BRACKETS: [(f64, f64); 7] = [
    (11_600.0, 0.10),
    (47_150.0, 0.12),
    (100_525.0, 0.22),
    (191_950.0, 0.24),
    (243_725.0, 0.32),
    (609_350.0, 0.35),
    (f64::INFINITY, 0.37),
];

/// Calculate progressive income tax for a given income.
pub fn calculate_tax(income: f64) -> TaxResult {
    if income <= 0.0 {
        return TaxResult {
            total_tax: 0.0,
            effective_rate: 0.0,
            marginal_rate: 0.0,
            breakdown: Vec::new(),
        };
    }

    let mut breakdown = Vec::new();
    let mut total_tax = 0.0;
    let mut prev_limit = 0.0;
    let mut marginal_rate = 0.0;

    for &(limit, rate) in &BRACKETS {
        if income <= prev_limit {
            break;
        }
        let taxable = income.min(limit) - prev_limit;
        let tax = taxable * rate;
        total_tax += tax;
        marginal_rate = rate;
        breakdown.push(BracketTax {
            bracket_floor: prev_limit,
            rate,
            taxable,
            tax,
        });
        prev_limit = limit;
    }

    let effective_rate = total_tax / income;

    TaxResult {
        total_tax,
        effective_rate,
        marginal_rate,
        breakdown,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_50k_total_tax() {
        let result = calculate_tax(50_000.0);
        // 10% on 11,600 = 1,160 + 12% on 35,550 = 4,266 + 22% on 2,850 = 627
        assert!((result.total_tax - 6_053.0).abs() < 1.0);
    }

    #[test]
    fn test_50k_marginal_rate() {
        let result = calculate_tax(50_000.0);
        assert!((result.marginal_rate - 0.22).abs() < 0.001);
    }

    #[test]
    fn test_50k_effective_rate() {
        let result = calculate_tax(50_000.0);
        assert!((result.effective_rate - 0.12106).abs() < 0.001);
    }

    #[test]
    fn test_zero_income() {
        let result = calculate_tax(0.0);
        assert_eq!(result.total_tax, 0.0);
        assert!(result.breakdown.is_empty());
    }

    #[test]
    fn test_bracket_count_50k() {
        let result = calculate_tax(50_000.0);
        assert_eq!(result.breakdown.len(), 3);
    }

    #[test]
    fn test_top_bracket() {
        let result = calculate_tax(750_000.0);
        assert!((result.marginal_rate - 0.37).abs() < 0.001);
        assert_eq!(result.breakdown.len(), 7);
    }
}
