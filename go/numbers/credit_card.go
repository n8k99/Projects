package numbers

import (
	"strconv"
	"strings"
	"unicode"
)

// CardValidation holds the result of credit card validation.
type CardValidation struct {
	Valid   bool
	Network string
	Display string
}

// LuhnCheck validates a credit card number using the Luhn algorithm.
func LuhnCheck(number string) bool {
	digits := onlyDigits(number)
	if len(digits) < 2 {
		return false
	}

	total := 0
	for i, d := range reverseBytes(digits) {
		v := int(d - '0')
		if i%2 == 1 {
			v *= 2
			if v > 9 {
				v -= 9
			}
		}
		total += v
	}
	return total%10 == 0
}

// IdentifyNetwork identifies the card network by prefix and length.
// Returns "Visa", "Mastercard", "Amex", "Discover", or "" for unknown.
func IdentifyNetwork(number string) string {
	digits := onlyDigits(number)
	n := len(digits)

	// Amex: 34xx or 37xx, length 15
	if n == 15 {
		prefix2 := digits[:2]
		if prefix2 == "34" || prefix2 == "37" {
			return "Amex"
		}
	}

	// Visa: starts with 4, length 13 or 16
	if digits[0] == '4' && (n == 13 || n == 16) {
		return "Visa"
	}

	if n == 16 {
		prefix2, _ := strconv.Atoi(digits[:2])
		prefix4, _ := strconv.Atoi(digits[:4])

		// Mastercard: 51-55 or 2221-2720
		if prefix2 >= 51 && prefix2 <= 55 {
			return "Mastercard"
		}
		if prefix4 >= 2221 && prefix4 <= 2720 {
			return "Mastercard"
		}

		// Discover: 6011, 644-649, 65xx
		if digits[:4] == "6011" {
			return "Discover"
		}
		prefix3, _ := strconv.Atoi(digits[:3])
		if prefix3 >= 644 && prefix3 <= 649 {
			return "Discover"
		}
		if digits[:2] == "65" {
			return "Discover"
		}
	}

	return ""
}

// ValidateCard validates a credit card number: Luhn check + network identification.
func ValidateCard(number string) CardValidation {
	digits := onlyDigits(number)
	last4 := digits
	if len(digits) >= 4 {
		last4 = digits[len(digits)-4:]
	}
	display := "****" + last4

	network := IdentifyNetwork(digits)
	valid := LuhnCheck(digits) && network != ""

	return CardValidation{
		Valid:   valid,
		Network: network,
		Display: display,
	}
}

func onlyDigits(s string) string {
	var b strings.Builder
	for _, r := range s {
		if unicode.IsDigit(r) {
			b.WriteRune(r)
		}
	}
	return b.String()
}

func reverseBytes(s string) []byte {
	b := []byte(s)
	for i, j := 0, len(b)-1; i < j; i, j = i+1, j-1 {
		b[i], b[j] = b[j], b[i]
	}
	return b
}
