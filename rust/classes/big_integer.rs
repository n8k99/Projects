//! Handle Large Numbers — an arbitrary-precision signed integer as a class.
//!
//! The class IS the number. Digits are stored base-10 little-endian; canonical
//! form has no trailing zeros and represents zero as a sign-less empty vector.
//! Operations are the schoolbook algorithms, with carry and borrow.

use std::cmp::Ordering;
use std::fmt;

#[derive(Clone, Debug)]
pub struct BigInt {
    pub negative: bool,
    /// Little-endian base-10 digits (no trailing zeros). Empty = zero.
    pub digits: Vec<u8>,
}

impl BigInt {
    pub fn zero() -> Self {
        Self { negative: false, digits: Vec::new() }
    }

    pub fn from_i64(n: i64) -> Self {
        if n == 0 { return Self::zero(); }
        let negative = n < 0;
        let mut x = if negative { (n as i128).unsigned_abs() } else { n as u128 };
        let mut digits = Vec::new();
        while x > 0 {
            digits.push((x % 10) as u8);
            x /= 10;
        }
        Self { negative, digits }
    }

    pub fn from_str(s: &str) -> Result<Self, String> {
        let s = s.trim();
        if s.is_empty() { return Err("empty string".into()); }
        let (negative, body) = match s.as_bytes()[0] {
            b'-' => (true, &s[1..]),
            b'+' => (false, &s[1..]),
            _ => (false, s),
        };
        if body.is_empty() || !body.bytes().all(|c| c.is_ascii_digit()) {
            return Err(format!("not a decimal integer: {s:?}"));
        }
        let mut digits: Vec<u8> = body.bytes().rev().map(|c| c - b'0').collect();
        while digits.len() > 1 && *digits.last().unwrap() == 0 {
            digits.pop();
        }
        if digits == vec![0] {
            return Ok(Self::zero());
        }
        Ok(Self { negative, digits })
    }

    pub fn is_zero(&self) -> bool {
        self.digits.is_empty()
    }

    pub fn neg(&self) -> Self {
        if self.is_zero() { return self.clone(); }
        Self { negative: !self.negative, digits: self.digits.clone() }
    }

    pub fn abs(&self) -> Self {
        Self { negative: false, digits: self.digits.clone() }
    }

    fn cmp_magnitude(&self, other: &Self) -> Ordering {
        match self.digits.len().cmp(&other.digits.len()) {
            Ordering::Equal => {}
            ord => return ord,
        }
        for (a, b) in self.digits.iter().rev().zip(other.digits.iter().rev()) {
            match a.cmp(b) {
                Ordering::Equal => continue,
                ord => return ord,
            }
        }
        Ordering::Equal
    }

    pub fn add(&self, other: &Self) -> Self {
        if self.negative == other.negative {
            if self.is_zero() && other.is_zero() { return Self::zero(); }
            return Self { negative: self.negative, digits: add_digits(&self.digits, &other.digits) };
        }
        match self.cmp_magnitude(other) {
            Ordering::Equal => Self::zero(),
            Ordering::Greater => Self { negative: self.negative, digits: sub_digits(&self.digits, &other.digits) },
            Ordering::Less => Self { negative: other.negative, digits: sub_digits(&other.digits, &self.digits) },
        }
    }

    pub fn sub(&self, other: &Self) -> Self { self.add(&other.neg()) }

    pub fn mul(&self, other: &Self) -> Self {
        if self.is_zero() || other.is_zero() { return Self::zero(); }
        Self {
            negative: self.negative != other.negative,
            digits: mul_digits(&self.digits, &other.digits),
        }
    }
}

// --- low-level schoolbook digit operations (magnitude only) ---

fn add_digits(a: &[u8], b: &[u8]) -> Vec<u8> {
    if a.is_empty() { return b.to_vec(); }
    if b.is_empty() { return a.to_vec(); }
    let n = a.len().max(b.len());
    let mut out = Vec::with_capacity(n + 1);
    let mut carry: u8 = 0;
    for i in 0..n {
        let da = *a.get(i).unwrap_or(&0);
        let db = *b.get(i).unwrap_or(&0);
        let s = da + db + carry;
        out.push(s % 10);
        carry = s / 10;
    }
    if carry > 0 { out.push(carry); }
    out
}

/// Assumes |a| >= |b|.
fn sub_digits(a: &[u8], b: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(a.len());
    let mut borrow: i16 = 0;
    for i in 0..a.len() {
        let da = a[i] as i16;
        let db = *b.get(i).unwrap_or(&0) as i16;
        let mut d = da - db - borrow;
        if d < 0 { d += 10; borrow = 1; } else { borrow = 0; }
        out.push(d as u8);
    }
    debug_assert_eq!(borrow, 0, "sub_digits called with |a| < |b|");
    while out.len() > 1 && *out.last().unwrap() == 0 { out.pop(); }
    if out == vec![0] { return Vec::new(); }
    out
}

fn mul_digits(a: &[u8], b: &[u8]) -> Vec<u8> {
    let mut out = vec![0u16; a.len() + b.len()];
    for (i, &da) in a.iter().enumerate() {
        let mut carry: u16 = 0;
        for (j, &db) in b.iter().enumerate() {
            let s = out[i + j] + (da as u16) * (db as u16) + carry;
            out[i + j] = s % 10;
            carry = s / 10;
        }
        out[i + b.len()] += carry;
    }
    let mut result: Vec<u8> = out.iter().map(|&x| x as u8).collect();
    while result.len() > 1 && *result.last().unwrap() == 0 { result.pop(); }
    if result == vec![0] { return Vec::new(); }
    result
}

// --- trait impls ---

impl PartialEq for BigInt {
    fn eq(&self, other: &Self) -> bool {
        self.negative == other.negative && self.digits == other.digits
    }
}
impl Eq for BigInt {}

impl PartialOrd for BigInt {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> { Some(self.cmp(other)) }
}

impl Ord for BigInt {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.is_zero() && other.is_zero() { return Ordering::Equal; }
        match (self.negative, other.negative) {
            (true, false) => Ordering::Less,
            (false, true) => Ordering::Greater,
            (false, false) => self.cmp_magnitude(other),
            (true, true) => other.cmp_magnitude(self), // more-negative magnitude is lesser
        }
    }
}

impl fmt::Display for BigInt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_zero() { return write!(f, "0"); }
        if self.negative { write!(f, "-")?; }
        for &d in self.digits.iter().rev() {
            write!(f, "{d}")?;
        }
        Ok(())
    }
}

impl std::ops::Add for BigInt { type Output = BigInt; fn add(self, rhs: Self) -> Self { BigInt::add(&self, &rhs) } }
impl std::ops::Sub for BigInt { type Output = BigInt; fn sub(self, rhs: Self) -> Self { BigInt::sub(&self, &rhs) } }
impl std::ops::Mul for BigInt { type Output = BigInt; fn mul(self, rhs: Self) -> Self { BigInt::mul(&self, &rhs) } }
impl std::ops::Neg for BigInt { type Output = BigInt; fn neg(self) -> Self { BigInt::neg(&self) } }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_and_display_roundtrips() {
        let cases = &["0", "1", "-1", "42", "-42",
                      "12345678901234567890", "-98765432109876543210"];
        for &s in cases {
            assert_eq!(BigInt::from_str(s).unwrap().to_string(), s);
        }
    }

    #[test]
    fn addition_is_commutative() {
        let a = BigInt::from_str("123456789").unwrap();
        let b = BigInt::from_str("987654321").unwrap();
        assert_eq!(a.add(&b), b.add(&a));
    }

    #[test]
    fn subtraction_is_add_of_neg() {
        let a = BigInt::from_str("10000").unwrap();
        let b = BigInt::from_str("3").unwrap();
        assert_eq!(a.sub(&b), a.add(&b.neg()));
    }

    #[test]
    fn mixed_sign_addition_picks_larger_magnitude_sign() {
        let a = BigInt::from_str("100").unwrap();
        let b = BigInt::from_str("-30").unwrap();
        assert_eq!(a.add(&b), BigInt::from_str("70").unwrap());
        let c = BigInt::from_str("30").unwrap();
        let d = BigInt::from_str("-100").unwrap();
        assert_eq!(c.add(&d), BigInt::from_str("-70").unwrap());
    }

    #[test]
    fn multiplication_carries_correctly() {
        // 999 * 999 = 998001 — the carry chain goes all the way up
        let a = BigInt::from_str("999").unwrap();
        assert_eq!(a.mul(&a).to_string(), "998001");
        let big = BigInt::from_str("100000000000").unwrap();
        assert_eq!(big.mul(&big).to_string(), "10000000000000000000000");
    }

    #[test]
    fn factorial_30_matches_known_value() {
        let mut fact = BigInt::from_i64(1);
        for k in 1..=30 {
            fact = fact.mul(&BigInt::from_i64(k));
        }
        assert_eq!(fact.to_string(), "265252859812191058636308480000000");
    }

    #[test]
    fn total_order_handles_signs() {
        let neg_big  = BigInt::from_str("-1000000000").unwrap();
        let neg_small = BigInt::from_str("-1").unwrap();
        let zero = BigInt::zero();
        let pos_small = BigInt::from_str("1").unwrap();
        let pos_big   = BigInt::from_str("1000000000").unwrap();
        let mut v = vec![pos_big.clone(), neg_big.clone(), zero.clone(), neg_small.clone(), pos_small.clone()];
        v.sort();
        assert_eq!(v, vec![neg_big, neg_small, zero, pos_small, pos_big]);
    }
}
