package numbers

// PrimeFactors returns the prime factors of n in ascending order, with repetition.
func PrimeFactors(n uint64) []uint64 {
	if n < 2 {
		return nil
	}

	var factors []uint64
	var d uint64 = 2
	for d*d <= n {
		for n%d == 0 {
			factors = append(factors, d)
			n /= d
		}
		d++
	}
	if n > 1 {
		factors = append(factors, n)
	}
	return factors
}

// PrimeFactorization returns a map of {prime: exponent}.
func PrimeFactorization(n uint64) map[uint64]int {
	result := make(map[uint64]int)
	for _, f := range PrimeFactors(n) {
		result[f]++
	}
	return result
}
