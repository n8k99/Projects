//! Next prime number — continuously find primes on demand.

/// Test whether `n` is prime.
pub fn is_prime(n: u64) -> bool {
    if n < 2 { return false; }
    if n < 4 { return true; }
    if n % 2 == 0 || n % 3 == 0 { return false; }
    let mut d = 5u64;
    while d * d <= n {
        if n % d == 0 || n % (d + 2) == 0 { return false; }
        d += 6;
    }
    true
}

/// Return the smallest prime greater than `n`.
pub fn next_prime(n: u64) -> u64 {
    let mut candidate = if n >= 2 { n + 1 } else { 2 };
    if candidate == 2 { return 2; }
    if candidate % 2 == 0 { candidate += 1; }
    while !is_prime(candidate) {
        candidate += 2;
    }
    candidate
}

/// Return the nth prime (1-indexed).
pub fn nth_prime(n: usize) -> u64 {
    assert!(n >= 1, "n must be positive");
    let mut count = 0usize;
    let mut candidate = 2u64;
    loop {
        if is_prime(candidate) {
            count += 1;
            if count == n { return candidate; }
        }
        candidate += if candidate == 2 { 1 } else { 2 };
    }
}

/// Infinite prime iterator.
pub struct Primes {
    candidate: u64,
}

impl Primes {
    pub fn new() -> Self { Self { candidate: 2 } }
}

impl Iterator for Primes {
    type Item = u64;
    fn next(&mut self) -> Option<u64> {
        while !is_prime(self.candidate) {
            self.candidate += if self.candidate == 2 { 1 } else { 2 };
        }
        let val = self.candidate;
        self.candidate += if self.candidate == 2 { 1 } else { 2 };
        Some(val)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_next_prime() {
        assert_eq!(next_prime(0), 2);
        assert_eq!(next_prime(2), 3);
        assert_eq!(next_prime(10), 11);
        assert_eq!(next_prime(13), 17);
    }

    #[test]
    fn test_nth_prime() {
        assert_eq!(nth_prime(1), 2);
        assert_eq!(nth_prime(5), 11);
        assert_eq!(nth_prime(100), 541);
    }
}
