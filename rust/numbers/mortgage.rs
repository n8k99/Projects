//! Mortgage calculator — monthly payments, amortization, pace tracking.

/// A single payment in an amortization schedule.
pub struct Payment {
    pub period: u32,
    pub payment: f64,
    pub principal: f64,
    pub interest: f64,
    pub balance: f64,
}

/// Calculate fixed monthly payment for a mortgage.
pub fn monthly_payment(principal: f64, annual_rate: f64, years: u32) -> f64 {
    if annual_rate == 0.0 {
        return principal / (years as f64 * 12.0);
    }
    let r = annual_rate / 100.0 / 12.0;
    let n = (years * 12) as f64;
    let factor = (1.0 + r).powf(n);
    principal * r * factor / (factor - 1.0)
}

/// Generate full amortization schedule.
pub fn amortization_schedule(principal: f64, annual_rate: f64, years: u32) -> Vec<Payment> {
    let pmt = monthly_payment(principal, annual_rate, years);
    let r = annual_rate / 100.0 / 12.0;
    let mut balance = principal;
    let n = years * 12;
    let mut schedule = Vec::with_capacity(n as usize);

    for period in 1..=n {
        let interest = balance * r;
        let mut princ = pmt - interest;
        balance -= princ;
        if period == n {
            princ += balance;
            balance = 0.0;
        }
        schedule.push(Payment { period, payment: pmt, principal: princ, interest, balance });
    }
    schedule
}

/// Result of a pace check.
pub struct PaceResult {
    pub expected_by_now: f64,
    pub actual: f64,
    pub delta: f64,
    pub pace_percent: f64,
    pub on_track: bool,
}

/// Check if cumulative payments are on pace to meet target.
pub fn pace_check(target: f64, periods: u32, current_period: u32, cumulative: f64) -> PaceResult {
    let expected = target * current_period as f64 / periods as f64;
    let delta = cumulative - expected;
    let pace_percent = if expected > 0.0 { cumulative / expected * 100.0 } else { 0.0 };

    PaceResult {
        expected_by_now: expected,
        actual: cumulative,
        delta,
        pace_percent,
        on_track: delta >= 0.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_monthly_payment() {
        let pmt = monthly_payment(200_000.0, 6.0, 30);
        assert!((pmt - 1199.10).abs() < 1.0);
    }

    #[test]
    fn test_pace_ahead() {
        let r = pace_check(300.0, 30, 10, 120.0);
        assert!(r.on_track);
    }

    #[test]
    fn test_pace_behind() {
        let r = pace_check(300.0, 30, 10, 80.0);
        assert!(!r.on_track);
    }
}
