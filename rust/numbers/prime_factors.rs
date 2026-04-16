//! Prime factorization — find all prime factors of a number.

use std::collections::BTreeMap;

/// Return the prime factors of `n` in ascending order, with repetition.
pub fn prime_factors(mut n: u64) -> Vec<u64> {
    if n < 2 {
        return vec![];
    }

    let mut factors = Vec::new();
    let mut d = 2u64;
    while d * d <= n {
        while n % d == 0 {
            factors.push(d);
            n /= d;
        }
        d += 1;
    }
    if n > 1 {
        factors.push(n);
    }
    factors
}

/// Return prime factorization as a sorted map of {prime: exponent}.
pub fn prime_factorization(n: u64) -> BTreeMap<u64, u32> {
    let mut map = BTreeMap::new();
    for f in prime_factors(n) {
        *map.entry(f).or_insert(0) += 1;
    }
    map
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prime_factors() {
        assert_eq!(prime_factors(12), vec![2, 2, 3]);
        assert_eq!(prime_factors(100), vec![2, 2, 5, 5]);
        assert_eq!(prime_factors(17), vec![17]);
        let empty: Vec<u64> = vec![];
        assert_eq!(prime_factors(1), empty);
    }
}
