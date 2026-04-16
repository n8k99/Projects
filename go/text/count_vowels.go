// Package text provides text analysis utilities.
//
// Count Vowels — analyze vowel distribution in text.
package text

import (
	"strings"
	"unicode"
)

// isVowel returns true if r is a vowel (optionally including 'y').
func isVowel(r rune, includeY bool) bool {
	lower := unicode.ToLower(r)
	switch lower {
	case 'a', 'e', 'i', 'o', 'u':
		return true
	case 'y':
		return includeY
	}
	return false
}

// CountVowels counts the total number of vowels in text.
// If includeY is true, 'y' is counted as a vowel.
func CountVowels(text string, includeY bool) int {
	count := 0
	for _, r := range text {
		if isVowel(r, includeY) {
			count++
		}
	}
	return count
}

// VowelBreakdown returns a per-vowel frequency map.
// Keys are lowercase vowel runes; all vowels appear even if count is 0.
func VowelBreakdown(text string, includeY bool) map[rune]int {
	keys := []rune{'a', 'e', 'i', 'o', 'u'}
	if includeY {
		keys = append(keys, 'y')
	}
	counts := make(map[rune]int, len(keys))
	for _, k := range keys {
		counts[k] = 0
	}
	for _, r := range strings.ToLower(text) {
		if _, ok := counts[r]; ok {
			counts[r]++
		}
	}
	return counts
}

// VowelRatio returns the ratio of vowels (a, e, i, o, u) to total
// alphabetic characters. Returns 0.0 if there are no alphabetic characters.
func VowelRatio(text string) float64 {
	alphaCount := 0
	for _, r := range text {
		if unicode.IsLetter(r) {
			alphaCount++
		}
	}
	if alphaCount == 0 {
		return 0.0
	}
	return float64(CountVowels(text, false)) / float64(alphaCount)
}
