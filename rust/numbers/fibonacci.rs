//! Fibonacci sequence — generate to a count or up to a value.

/// Return the first `n` Fibonacci numbers.
pub fn fibonacci(n: usize) -> Vec<u128> {
    if n == 0 {
        return vec![];
    }
    if n == 1 {
        return vec![0];
    }

    let mut seq = Vec::with_capacity(n);
    seq.push(0);
    seq.push(1);
    for i in 2..n {
        seq.push(seq[i - 1] + seq[i - 2]);
    }
    seq
}

/// Return all Fibonacci numbers up to and including `limit`.
pub fn fibonacci_up_to(limit: u128) -> Vec<u128> {
    let mut seq = vec![0];
    let (mut a, mut b) = (0u128, 1u128);
    while b <= limit {
        seq.push(b);
        let next = a + b;
        a = b;
        b = next;
    }
    seq
}

/// Return the nth Fibonacci number (0-indexed).
pub fn fib(n: usize) -> u128 {
    let (mut a, mut b) = (0u128, 1u128);
    for _ in 0..n {
        let next = a + b;
        a = b;
        b = next;
    }
    a
}

/// Infinite Fibonacci iterator.
pub struct FibIter {
    a: u128,
    b: u128,
}

impl FibIter {
    pub fn new() -> Self {
        Self { a: 0, b: 1 }
    }
}

impl Iterator for FibIter {
    type Item = u128;

    fn next(&mut self) -> Option<Self::Item> {
        let val = self.a;
        let next = self.a + self.b;
        self.a = self.b;
        self.b = next;
        Some(val)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fibonacci_10() {
        assert_eq!(fibonacci(10), vec![0, 1, 1, 2, 3, 5, 8, 13, 21, 34]);
    }

    #[test]
    fn test_fib_nth() {
        assert_eq!(fib(0), 0);
        assert_eq!(fib(1), 1);
        assert_eq!(fib(10), 55);
    }

    #[test]
    fn test_up_to() {
        assert_eq!(fibonacci_up_to(10), vec![0, 1, 1, 2, 3, 5, 8]);
    }
}
