// Package text provides text processing utilities.
// Vigenere / Vernam / Caesar Ciphers — classical text encryption.
package text

import (
	"errors"
	"strings"
	"unicode"
)

// CaesarEncrypt encrypts text using Caesar cipher with the given shift.
func CaesarEncrypt(text string, shift int) string {
	s := ((shift % 26) + 26) % 26
	var b strings.Builder
	for _, ch := range text {
		if unicode.IsLetter(ch) && ch < 128 {
			base := 'a'
			if unicode.IsUpper(ch) {
				base = 'A'
			}
			b.WriteRune(rune((int(ch-base)+s)%26) + base)
		} else {
			b.WriteRune(ch)
		}
	}
	return b.String()
}

// CaesarDecrypt decrypts Caesar cipher by shifting in the opposite direction.
func CaesarDecrypt(text string, shift int) string {
	return CaesarEncrypt(text, -shift)
}

// VigenereEncrypt encrypts text using Vigenere cipher with the given keyword.
func VigenereEncrypt(text string, key string) (string, error) {
	if len(key) == 0 {
		return "", errors.New("key must be a non-empty alphabetic string")
	}
	keyUpper := strings.ToUpper(key)
	for _, c := range keyUpper {
		if c < 'A' || c > 'Z' {
			return "", errors.New("key must be a non-empty alphabetic string")
		}
	}
	keyBytes := []byte(keyUpper)
	var b strings.Builder
	ki := 0
	for _, ch := range text {
		if unicode.IsLetter(ch) && ch < 128 {
			shift := int(keyBytes[ki%len(keyBytes)] - 'A')
			base := 'a'
			if unicode.IsUpper(ch) {
				base = 'A'
			}
			b.WriteRune(rune((int(ch-base)+shift)%26) + base)
			ki++
		} else {
			b.WriteRune(ch)
		}
	}
	return b.String(), nil
}

// VigenereDecrypt decrypts Vigenere cipher by inverting each key letter's shift.
func VigenereDecrypt(text string, key string) (string, error) {
	if len(key) == 0 {
		return "", errors.New("key must be a non-empty alphabetic string")
	}
	keyUpper := strings.ToUpper(key)
	inverted := make([]byte, len(keyUpper))
	for i, c := range []byte(keyUpper) {
		if c < 'A' || c > 'Z' {
			return "", errors.New("key must be a non-empty alphabetic string")
		}
		inverted[i] = byte((26-int(c-'A'))%26) + 'A'
	}
	return VigenereEncrypt(text, string(inverted))
}

// VernamEncrypt encrypts data using Vernam (one-time pad) XOR cipher.
// Key must be at least as long as data.
func VernamEncrypt(data, key []byte) ([]byte, error) {
	if len(key) < len(data) {
		return nil, errors.New("key must be at least as long as the data")
	}
	result := make([]byte, len(data))
	for i, d := range data {
		result[i] = d ^ key[i]
	}
	return result, nil
}

// VernamDecrypt decrypts Vernam cipher (XOR is its own inverse).
func VernamDecrypt(data, key []byte) ([]byte, error) {
	return VernamEncrypt(data, key)
}
