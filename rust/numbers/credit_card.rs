//! Credit Card Validator — Luhn algorithm and network identification.

/// Validate a credit card number using the Luhn algorithm.
pub fn luhn_check(number: &str) -> bool {
    let digits: Vec<u32> = number.chars().filter_map(|c| c.to_digit(10)).collect();
    if digits.len() < 2 {
        return false;
    }

    let total: u32 = digits
        .iter()
        .rev()
        .enumerate()
        .map(|(i, &d)| {
            if i % 2 == 1 {
                let doubled = d * 2;
                if doubled > 9 { doubled - 9 } else { doubled }
            } else {
                d
            }
        })
        .sum();

    total % 10 == 0
}

/// Identify the card network by prefix and length.
///
/// Returns `Some("Visa")`, `Some("Mastercard")`, `Some("Amex")`,
/// `Some("Discover")`, or `None`.
pub fn identify_network(number: &str) -> Option<&'static str> {
    let digits: String = number.chars().filter(|c| c.is_ascii_digit()).collect();
    let n = digits.len();

    // Amex: 34xx or 37xx, length 15
    if n == 15 {
        let prefix2 = &digits[..2];
        if prefix2 == "34" || prefix2 == "37" {
            return Some("Amex");
        }
    }

    // Visa: starts with 4, length 13 or 16
    if digits.starts_with('4') && (n == 13 || n == 16) {
        return Some("Visa");
    }

    if n == 16 {
        let prefix2: u32 = digits[..2].parse().unwrap_or(0);
        let prefix4: u32 = digits[..4].parse().unwrap_or(0);

        // Mastercard: 51-55 or 2221-2720
        if (51..=55).contains(&prefix2) || (2221..=2720).contains(&prefix4) {
            return Some("Mastercard");
        }

        // Discover: 6011, 644-649, 65xx
        if &digits[..4] == "6011" {
            return Some("Discover");
        }
        let prefix3: u32 = digits[..3].parse().unwrap_or(0);
        if (644..=649).contains(&prefix3) || &digits[..2] == "65" {
            return Some("Discover");
        }
    }

    None
}

/// Result of credit card validation.
pub struct CardValidation {
    pub valid: bool,
    pub network: Option<String>,
    pub display: String,
}

/// Validate a credit card number: Luhn check + network identification.
pub fn validate(number: &str) -> CardValidation {
    let digits: String = number.chars().filter(|c| c.is_ascii_digit()).collect();
    let last4 = if digits.len() >= 4 {
        &digits[digits.len() - 4..]
    } else {
        &digits
    };
    let display = format!("****{}", last4);

    let network = identify_network(&digits);
    let valid = luhn_check(&digits) && network.is_some();

    CardValidation {
        valid,
        network: network.map(String::from),
        display,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_luhn_valid() {
        assert!(luhn_check("4532015112830366"));
        assert!(luhn_check("5425233430109903"));
        assert!(luhn_check("378282246310005"));
        assert!(luhn_check("6011111111111117"));
    }

    #[test]
    fn test_luhn_invalid() {
        assert!(!luhn_check("1234567890123456"));
        assert!(!luhn_check("0000000000000001"));
    }

    #[test]
    fn test_identify_visa() {
        assert_eq!(identify_network("4532015112830366"), Some("Visa"));
    }

    #[test]
    fn test_identify_mastercard() {
        assert_eq!(identify_network("5425233430109903"), Some("Mastercard"));
    }

    #[test]
    fn test_identify_amex() {
        assert_eq!(identify_network("378282246310005"), Some("Amex"));
    }

    #[test]
    fn test_identify_discover() {
        assert_eq!(identify_network("6011111111111117"), Some("Discover"));
    }

    #[test]
    fn test_identify_unknown() {
        assert_eq!(identify_network("1234567890123456"), None);
    }

    #[test]
    fn test_validate_visa() {
        let result = validate("4532015112830366");
        assert!(result.valid);
        assert_eq!(result.network, Some("Visa".to_string()));
        assert_eq!(result.display, "****0366");
    }

    #[test]
    fn test_validate_invalid() {
        let result = validate("1234567890123456");
        assert!(!result.valid);
        assert_eq!(result.network, None);
    }
}
