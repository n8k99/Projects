// Handle Large Numbers — an arbitrary-precision signed integer as a class.
//
// The class IS the number. Digits are stored base-10 little-endian; canonical
// form has no trailing zeros and zero is the empty slice with Negative=false.
package classes

import (
	"errors"
	"fmt"
	"strings"
)

// BigInt is an arbitrary-precision signed integer.
type BigInt struct {
	Negative bool
	Digits   []uint8 // little-endian base-10; empty = zero
}

// BigZero returns 0.
func BigZero() BigInt { return BigInt{} }

// BigFromInt64 constructs a BigInt from a 64-bit integer.
func BigFromInt64(n int64) BigInt {
	if n == 0 {
		return BigZero()
	}
	negative := n < 0
	var x uint64
	if negative {
		x = uint64(-n)
	} else {
		x = uint64(n)
	}
	var digits []uint8
	for x > 0 {
		digits = append(digits, uint8(x%10))
		x /= 10
	}
	return BigInt{Negative: negative, Digits: digits}
}

// BigFromString parses a decimal string with optional sign.
func BigFromString(s string) (BigInt, error) {
	s = strings.TrimSpace(s)
	if s == "" {
		return BigInt{}, errors.New("empty string")
	}
	negative := false
	switch s[0] {
	case '-':
		negative = true
		s = s[1:]
	case '+':
		s = s[1:]
	}
	if s == "" {
		return BigInt{}, fmt.Errorf("not a decimal integer")
	}
	digits := make([]uint8, len(s))
	for i := 0; i < len(s); i++ {
		c := s[len(s)-1-i]
		if c < '0' || c > '9' {
			return BigInt{}, fmt.Errorf("not a decimal integer: %q", s)
		}
		digits[i] = c - '0'
	}
	for len(digits) > 1 && digits[len(digits)-1] == 0 {
		digits = digits[:len(digits)-1]
	}
	if len(digits) == 1 && digits[0] == 0 {
		return BigZero(), nil
	}
	return BigInt{Negative: negative, Digits: digits}, nil
}

// IsZero reports whether the value is exactly zero.
func (x BigInt) IsZero() bool { return len(x.Digits) == 0 }

// Neg returns -x.
func (x BigInt) Neg() BigInt {
	if x.IsZero() {
		return x
	}
	return BigInt{Negative: !x.Negative, Digits: append([]uint8(nil), x.Digits...)}
}

// Abs returns |x|.
func (x BigInt) Abs() BigInt {
	return BigInt{Negative: false, Digits: append([]uint8(nil), x.Digits...)}
}

// cmpMagnitude returns -1/0/1 comparing absolute values.
func (x BigInt) cmpMagnitude(y BigInt) int {
	if len(x.Digits) != len(y.Digits) {
		if len(x.Digits) < len(y.Digits) {
			return -1
		}
		return 1
	}
	for i := len(x.Digits) - 1; i >= 0; i-- {
		if x.Digits[i] != y.Digits[i] {
			if x.Digits[i] < y.Digits[i] {
				return -1
			}
			return 1
		}
	}
	return 0
}

// Cmp returns -1/0/1 as the total order: -big < -small < 0 < +small < +big.
func (x BigInt) Cmp(y BigInt) int {
	if x.IsZero() && y.IsZero() {
		return 0
	}
	switch {
	case x.Negative && !y.Negative:
		return -1
	case !x.Negative && y.Negative:
		return 1
	case !x.Negative && !y.Negative:
		return x.cmpMagnitude(y)
	default: // both negative
		return y.cmpMagnitude(x)
	}
}

// Equal reports value equality.
func (x BigInt) Equal(y BigInt) bool { return x.Cmp(y) == 0 }

// Add returns x + y.
func (x BigInt) Add(y BigInt) BigInt {
	if x.Negative == y.Negative {
		if x.IsZero() && y.IsZero() {
			return BigZero()
		}
		return BigInt{Negative: x.Negative, Digits: addDigits(x.Digits, y.Digits)}
	}
	switch x.cmpMagnitude(y) {
	case 0:
		return BigZero()
	case 1:
		return BigInt{Negative: x.Negative, Digits: subDigits(x.Digits, y.Digits)}
	default:
		return BigInt{Negative: y.Negative, Digits: subDigits(y.Digits, x.Digits)}
	}
}

// Sub returns x - y.
func (x BigInt) Sub(y BigInt) BigInt { return x.Add(y.Neg()) }

// Mul returns x * y.
func (x BigInt) Mul(y BigInt) BigInt {
	if x.IsZero() || y.IsZero() {
		return BigZero()
	}
	return BigInt{
		Negative: x.Negative != y.Negative,
		Digits:   mulDigits(x.Digits, y.Digits),
	}
}

// String formats as decimal with optional leading minus.
func (x BigInt) String() string {
	if x.IsZero() {
		return "0"
	}
	var b strings.Builder
	if x.Negative {
		b.WriteByte('-')
	}
	for i := len(x.Digits) - 1; i >= 0; i-- {
		b.WriteByte('0' + x.Digits[i])
	}
	return b.String()
}

// --- schoolbook digit operations (magnitude only) ---

func addDigits(a, b []uint8) []uint8 {
	if len(a) == 0 {
		return append([]uint8(nil), b...)
	}
	if len(b) == 0 {
		return append([]uint8(nil), a...)
	}
	n := len(a)
	if len(b) > n {
		n = len(b)
	}
	out := make([]uint8, 0, n+1)
	var carry uint8
	for i := 0; i < n; i++ {
		var da, db uint8
		if i < len(a) {
			da = a[i]
		}
		if i < len(b) {
			db = b[i]
		}
		s := da + db + carry
		out = append(out, s%10)
		carry = s / 10
	}
	if carry > 0 {
		out = append(out, carry)
	}
	return out
}

func subDigits(a, b []uint8) []uint8 {
	out := make([]uint8, 0, len(a))
	var borrow int16
	for i := 0; i < len(a); i++ {
		da := int16(a[i])
		var db int16
		if i < len(b) {
			db = int16(b[i])
		}
		d := da - db - borrow
		if d < 0 {
			d += 10
			borrow = 1
		} else {
			borrow = 0
		}
		out = append(out, uint8(d))
	}
	// Strip trailing zeros.
	for len(out) > 1 && out[len(out)-1] == 0 {
		out = out[:len(out)-1]
	}
	if len(out) == 1 && out[0] == 0 {
		return nil
	}
	return out
}

func mulDigits(a, b []uint8) []uint8 {
	acc := make([]uint16, len(a)+len(b))
	for i, da := range a {
		var carry uint16
		for j, db := range b {
			s := acc[i+j] + uint16(da)*uint16(db) + carry
			acc[i+j] = s % 10
			carry = s / 10
		}
		acc[i+len(b)] += carry
	}
	out := make([]uint8, len(acc))
	for i, v := range acc {
		out[i] = uint8(v)
	}
	for len(out) > 1 && out[len(out)-1] == 0 {
		out = out[:len(out)-1]
	}
	if len(out) == 1 && out[0] == 0 {
		return nil
	}
	return out
}
