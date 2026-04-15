// Package numbers provides math, algorithms, and converters.
package numbers

import (
	"fmt"
	"math/big"
	"strings"
)

// Pi returns PI to n decimal places as a string.
// Uses the Chudnovsky algorithm with math/big for arbitrary precision.
func Pi(n int) string {
	if n < 0 {
		panic("n must be non-negative")
	}
	if n == 0 {
		return "3"
	}

	// Precision in bits: ~3.32 bits per decimal digit, plus margin
	prec := uint(float64(n)*3.33) + 128

	one := new(big.Float).SetPrec(prec).SetInt64(1)
	c := new(big.Float).SetPrec(prec).SetInt64(426880)
	sqrt10005 := new(big.Float).SetPrec(prec).SetInt64(10005)
	sqrt10005.Sqrt(sqrt10005)
	c.Mul(c, sqrt10005)

	k := new(big.Float).SetPrec(prec).SetInt64(0)
	m := new(big.Float).SetPrec(prec).SetInt64(1)
	x := new(big.Float).SetPrec(prec).SetInt64(1)
	l := new(big.Float).SetPrec(prec).SetInt64(13591409)
	s := new(big.Float).SetPrec(prec).SetInt64(13591409)

	twelve := new(big.Float).SetPrec(prec).SetInt64(12)
	lInc := new(big.Float).SetPrec(prec).SetInt64(545140134)
	xMul := new(big.Float).SetPrec(prec).SetInt64(-262537412640768000)
	sixteen := new(big.Float).SetPrec(prec).SetInt64(16)

	for i := int64(1); int(i) <= n; i++ {
		// M = M * (K^3 - 16*K) / i^3
		k3 := new(big.Float).SetPrec(prec).Mul(k, k)
		k3.Mul(k3, k)
		k16 := new(big.Float).SetPrec(prec).Mul(sixteen, k)
		num := new(big.Float).SetPrec(prec).Sub(k3, k16)
		m.Mul(m, num)
		iCubed := new(big.Float).SetPrec(prec).SetInt64(i * i * i)
		m.Quo(m, iCubed)

		k.Add(k, twelve)
		l.Add(l, lInc)
		x.Mul(x, xMul)

		term := new(big.Float).SetPrec(prec).Mul(m, l)
		term.Quo(term, x)
		s.Add(s, term)
	}

	pi := new(big.Float).SetPrec(prec).Quo(c, s)

	// Ignore the exact-flag from big.Float.Text — we just want the string
	_ = one // suppress unused
	result := pi.Text('f', n)

	// Trim trailing zeros if any crept in beyond precision
	if strings.Contains(result, ".") {
		parts := strings.SplitN(result, ".", 2)
		dec := parts[1]
		if len(dec) > n {
			dec = dec[:n]
		}
		return fmt.Sprintf("%s.%s", parts[0], dec)
	}
	return result
}
