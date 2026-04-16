package numbers

import (
	"fmt"
	"strings"
)

// ToBinary converts a non-negative integer to its binary string.
func ToBinary(n uint64) string {
	if n == 0 {
		return "0"
	}
	var bits []byte
	for n > 0 {
		if n&1 == 1 {
			bits = append(bits, '1')
		} else {
			bits = append(bits, '0')
		}
		n >>= 1
	}
	// reverse
	for i, j := 0, len(bits)-1; i < j; i, j = i+1, j-1 {
		bits[i], bits[j] = bits[j], bits[i]
	}
	return string(bits)
}

// ToDecimal converts a binary string to its decimal value.
func ToDecimal(binary string) (uint64, error) {
	if binary == "" {
		return 0, fmt.Errorf("empty string")
	}
	var result uint64
	for _, c := range binary {
		switch c {
		case '0', '1':
			result = (result << 1) | uint64(c-'0')
		default:
			return 0, fmt.Errorf("invalid binary digit: %c", c)
		}
	}
	return result, nil
}

const baseDigits = "0123456789abcdefghijklmnopqrstuvwxyz"

// ToBase converts a non-negative integer to a string in the given base (2-36).
func ToBase(n uint64, base int) (string, error) {
	if base < 2 || base > 36 {
		return "", fmt.Errorf("base must be between 2 and 36")
	}
	if n == 0 {
		return "0", nil
	}
	var result []byte
	b := uint64(base)
	for n > 0 {
		result = append(result, baseDigits[n%b])
		n /= b
	}
	for i, j := 0, len(result)-1; i < j; i, j = i+1, j-1 {
		result[i], result[j] = result[j], result[i]
	}
	return string(result), nil
}

// FromBase converts a string in the given base (2-36) to decimal.
func FromBase(s string, base int) (uint64, error) {
	if base < 2 || base > 36 {
		return 0, fmt.Errorf("base must be between 2 and 36")
	}
	if s == "" {
		return 0, fmt.Errorf("empty string")
	}
	var result uint64
	for _, c := range strings.ToLower(s) {
		idx := strings.IndexRune(baseDigits, c)
		if idx < 0 || idx >= base {
			return 0, fmt.Errorf("digit '%c' invalid for base %d", c, base)
		}
		result = result*uint64(base) + uint64(idx)
	}
	return result, nil
}
