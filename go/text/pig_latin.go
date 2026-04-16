package text

import (
	"strings"
	"unicode"
)

func isVowel(r rune) bool {
	switch unicode.ToLower(r) {
	case 'a', 'e', 'i', 'o', 'u':
		return true
	}
	return false
}

// PigLatinWord converts a single English word to Pig Latin.
//
// Consonant cluster start: move cluster to end, add "ay".
// Vowel start: add "yay".
// Preserves capitalization of original first letter and trailing punctuation.
func PigLatinWord(word string) string {
	if len(word) == 0 {
		return word
	}

	runes := []rune(word)

	// Split trailing punctuation
	alphaEnd := len(runes)
	for alphaEnd > 0 && !unicode.IsLetter(runes[alphaEnd-1]) {
		alphaEnd--
	}
	core := runes[:alphaEnd]
	trail := runes[alphaEnd:]

	if len(core) == 0 {
		return word
	}

	wasCap := unicode.IsUpper(core[0])
	lower := []rune(strings.ToLower(string(core)))

	var result []rune
	if isVowel(lower[0]) {
		result = append(lower, 'y', 'a', 'y')
	} else {
		clusterEnd := 0
		for clusterEnd < len(lower) && !isVowel(lower[clusterEnd]) {
			clusterEnd++
		}
		rest := lower[clusterEnd:]
		cluster := lower[:clusterEnd]
		result = append(rest, cluster...)
		result = append(result, 'a', 'y')
	}

	if wasCap && len(result) > 0 {
		result[0] = unicode.ToUpper(result[0])
	}

	return string(append(result, trail...))
}

// PigLatin converts an English sentence to Pig Latin.
func PigLatin(text string) string {
	words := strings.Split(text, " ")
	translated := make([]string, len(words))
	for i, w := range words {
		translated[i] = PigLatinWord(w)
	}
	return strings.Join(translated, " ")
}
