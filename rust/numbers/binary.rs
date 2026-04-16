//! Binary to Decimal and Back — convert between number systems.

/// Convert a non-negative integer to its binary string.
pub fn to_binary(n: u64) -> String {
    if n == 0 {
        return "0".to_string();
    }
    let mut bits = Vec::new();
    let mut val = n;
    while val > 0 {
        bits.push(if val & 1 == 1 { '1' } else { '0' });
        val >>= 1;
    }
    bits.into_iter().rev().collect()
}

/// Convert a binary string to its decimal value.
pub fn to_decimal(binary: &str) -> Result<u64, String> {
    if binary.is_empty() {
        return Err("empty string".to_string());
    }
    let mut result: u64 = 0;
    for c in binary.chars() {
        match c {
            '0' | '1' => result = (result << 1) | (c as u64 - '0' as u64),
            _ => return Err(format!("invalid binary digit: {c}")),
        }
    }
    Ok(result)
}

/// Convert a non-negative integer to a string in the given base (2-36).
pub fn to_base(n: u64, base: u64) -> Result<String, String> {
    if !(2..=36).contains(&base) {
        return Err("base must be between 2 and 36".to_string());
    }
    if n == 0 {
        return Ok("0".to_string());
    }
    let digits: Vec<char> = "0123456789abcdefghijklmnopqrstuvwxyz".chars().collect();
    let mut result = Vec::new();
    let mut val = n;
    while val > 0 {
        result.push(digits[(val % base) as usize]);
        val /= base;
    }
    Ok(result.into_iter().rev().collect())
}

/// Convert a string in the given base (2-36) to decimal.
pub fn from_base(s: &str, base: u64) -> Result<u64, String> {
    if !(2..=36).contains(&base) {
        return Err("base must be between 2 and 36".to_string());
    }
    if s.is_empty() {
        return Err("empty string".to_string());
    }
    let mut result: u64 = 0;
    for c in s.to_lowercase().chars() {
        let val = match c {
            '0'..='9' => c as u64 - '0' as u64,
            'a'..='z' => c as u64 - 'a' as u64 + 10,
            _ => return Err(format!("invalid digit: {c}")),
        };
        if val >= base {
            return Err(format!("digit '{c}' invalid for base {base}"));
        }
        result = result * base + val;
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_binary() {
        assert_eq!(to_decimal(&to_binary(42)).unwrap(), 42);
    }

    #[test]
    fn roundtrip_hex() {
        assert_eq!(from_base(&to_base(255, 16).unwrap(), 16).unwrap(), 255);
    }

    #[test]
    fn zero() {
        assert_eq!(to_binary(0), "0");
        assert_eq!(to_decimal("0").unwrap(), 0);
    }
}
