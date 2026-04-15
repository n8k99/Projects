//! Find PI to the Nth digit using arbitrary precision.
//!
//! Uses the `rug` crate (GMP bindings) for arbitrary precision floats.

use rug::Float;

/// Return PI to `n` decimal places as a String.
///
/// Uses GMP's built-in PI computation (AGM-based, very fast).
pub fn pi(n: u32) -> String {
    if n == 0 {
        return "3".to_string();
    }

    // ~3.32 bits per decimal digit, plus margin
    let precision = ((n as f64) * 3.33 + 64.0) as u32;
    let val = Float::with_val(precision, rug::float::Constant::Pi);

    let formatted = format!("{:.prec$}", val, prec = n as usize);
    formatted
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pi_10() {
        assert_eq!(pi(10), "3.1415926536");
    }

    #[test]
    fn test_pi_0() {
        assert_eq!(pi(0), "3");
    }

    #[test]
    fn test_pi_50() {
        let result = pi(50);
        assert!(result.starts_with("3.14159265358979323846264338327950288419716939937510"));
    }
}
