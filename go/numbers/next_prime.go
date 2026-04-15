package numbers

// IsPrime tests whether n is prime.
func IsPrime(n uint64) bool {
	if n < 2 {
		return false
	}
	if n < 4 {
		return true
	}
	if n%2 == 0 || n%3 == 0 {
		return false
	}
	for d := uint64(5); d*d <= n; d += 6 {
		if n%d == 0 || n%(d+2) == 0 {
			return false
		}
	}
	return true
}

// NextPrime returns the smallest prime greater than n.
func NextPrime(n uint64) uint64 {
	candidate := n + 1
	if n < 2 {
		candidate = 2
	}
	if candidate == 2 {
		return 2
	}
	if candidate%2 == 0 {
		candidate++
	}
	for !IsPrime(candidate) {
		candidate += 2
	}
	return candidate
}

// NthPrime returns the nth prime (1-indexed).
func NthPrime(n int) uint64 {
	if n < 1 {
		panic("n must be positive")
	}
	count := 0
	candidate := uint64(2)
	for {
		if IsPrime(candidate) {
			count++
			if count == n {
				return candidate
			}
		}
		if candidate == 2 {
			candidate++
		} else {
			candidate += 2
		}
	}
}
