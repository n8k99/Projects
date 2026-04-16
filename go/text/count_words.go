// Count Words in a String — word-level text analysis.
package text

import (
	"strings"
	"unicode"
)

// cleanWord strips leading/trailing punctuation and lowercases.
func cleanWord(word string) string {
	return strings.ToLower(strings.TrimFunc(word, func(r rune) bool {
		return unicode.IsPunct(r)
	}))
}

// WordCount counts words in text (split on whitespace).
func WordCount(text string) int {
	if strings.TrimSpace(text) == "" {
		return 0
	}
	return len(strings.Fields(text))
}

// WordFrequency counts occurrences of each word (case-insensitive, strip punctuation).
func WordFrequency(text string) map[string]int {
	counts := make(map[string]int)
	for _, word := range strings.Fields(text) {
		cleaned := cleanWord(word)
		if cleaned != "" {
			counts[cleaned]++
		}
	}
	return counts
}

// UniqueWords counts distinct words (case-insensitive, strip punctuation).
func UniqueWords(text string) int {
	return len(WordFrequency(text))
}

// AverageWordLength returns the mean word length (punctuation stripped).
// Returns 0.0 if there are no words.
func AverageWordLength(text string) float64 {
	var words []string
	for _, word := range strings.Fields(text) {
		cleaned := cleanWord(word)
		if cleaned != "" {
			words = append(words, cleaned)
		}
	}
	if len(words) == 0 {
		return 0.0
	}
	totalLen := 0
	for _, w := range words {
		totalLen += len(w)
	}
	return float64(totalLen) / float64(len(words))
}
