package text

import (
	"strings"
	"unicode"
)

// IsPalindromeExact checks whether a string reads the same forwards and backwards (exact).
func IsPalindromeExact(s string) bool {
	runes := []rune(s)
	for i, j := 0, len(runes)-1; i < j; i, j = i+1, j-1 {
		if runes[i] != runes[j] {
			return false
		}
	}
	return true
}

// IsPalindromeNormalized checks whether a string is a palindrome,
// ignoring case, spaces, and punctuation.
func IsPalindromeNormalized(s string) bool {
	var normalized []rune
	for _, r := range s {
		if unicode.IsLetter(r) || unicode.IsDigit(r) {
			normalized = append(normalized, unicode.ToLower(r))
		}
	}
	for i, j := 0, len(normalized)-1; i < j; i, j = i+1, j-1 {
		if normalized[i] != normalized[j] {
			return false
		}
	}
	return true
}

// LongestPalindromeSubstring finds the longest palindromic substring
// using the expand-around-center algorithm.
// Operates on bytes; assumes ASCII input for substring search.
func LongestPalindromeSubstring(s string) string {
	if len(s) < 2 {
		return s
	}

	start, end := 0, 0

	expand := func(left, right int) (int, int) {
		for left >= 0 && right < len(s) && s[left] == s[right] {
			left--
			right++
		}
		return left + 1, right - 1
	}

	for i := 0; i < len(s); i++ {
		// Odd-length
		l1, r1 := expand(i, i)
		if r1-l1 > end-start {
			start, end = l1, r1
		}

		// Even-length
		l2, r2 := expand(i, i+1)
		if r2-l2 > end-start {
			start, end = l2, r2
		}
	}

	return s[start : end+1]
}

// containsString is a helper — not exported.
func containsString(haystack []string, needle string) bool {
	for _, s := range haystack {
		if strings.EqualFold(s, needle) {
			return true
		}
	}
	return false
}
