package numbers

import "math/big"

// Fibonacci returns the first n Fibonacci numbers.
func Fibonacci(n int) []*big.Int {
	if n <= 0 {
		return nil
	}

	seq := make([]*big.Int, n)
	seq[0] = big.NewInt(0)
	if n == 1 {
		return seq
	}
	seq[1] = big.NewInt(1)

	for i := 2; i < n; i++ {
		seq[i] = new(big.Int).Add(seq[i-1], seq[i-2])
	}
	return seq
}

// FibonacciUpTo returns all Fibonacci numbers up to and including limit.
func FibonacciUpTo(limit *big.Int) []*big.Int {
	if limit.Sign() < 0 {
		return nil
	}

	seq := []*big.Int{big.NewInt(0)}
	a := big.NewInt(0)
	b := big.NewInt(1)

	for b.Cmp(limit) <= 0 {
		seq = append(seq, new(big.Int).Set(b))
		next := new(big.Int).Add(a, b)
		a.Set(b)
		b.Set(next)
	}
	return seq
}

// Fib returns the nth Fibonacci number (0-indexed).
func Fib(n int) *big.Int {
	if n < 0 {
		panic("n must be non-negative")
	}

	a := big.NewInt(0)
	b := big.NewInt(1)

	for i := 0; i < n; i++ {
		next := new(big.Int).Add(a, b)
		a.Set(b)
		b.Set(next)
	}
	return a
}
