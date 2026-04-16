package text

import (
	"fmt"
	"math/rand"
	"strings"
	"time"
)

const (
	charset    = "ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789"
	groupLen   = 5
	dataGroups = 4
	prime      = 65521
)

func init() {
	rand.Seed(time.Now().UnixNano())
}

func computeChecksum(groups []string) string {
	raw := []byte(strings.Join(groups, ""))
	var h uint64
	for i, b := range raw {
		h += uint64(b) * uint64(i+1)
	}
	h %= prime
	chars := make([]byte, groupLen)
	for i := 0; i < groupLen; i++ {
		chars[i] = charset[h%uint64(len(charset))]
		h /= uint64(len(charset))
	}
	return string(chars)
}

// GenerateKey produces a CD key in XXXXX-XXXXX-XXXXX-XXXXX-XXXXX format.
func GenerateKey() string {
	groups := make([]string, dataGroups)
	for g := 0; g < dataGroups; g++ {
		chars := make([]byte, groupLen)
		for i := 0; i < groupLen; i++ {
			chars[i] = charset[rand.Intn(len(charset))]
		}
		groups[g] = string(chars)
	}
	checksum := computeChecksum(groups)
	return strings.Join(append(groups, checksum), "-")
}

// ValidateKey checks whether a CD key has a valid checksum.
func ValidateKey(key string) bool {
	parts := strings.Split(strings.TrimSpace(strings.ToUpper(key)), "-")
	if len(parts) != dataGroups+1 {
		return false
	}
	for _, p := range parts {
		if len(p) != groupLen {
			return false
		}
		for _, c := range p {
			if !strings.ContainsRune(charset, c) {
				return false
			}
		}
	}
	expected := computeChecksum(parts[:dataGroups])
	return parts[dataGroups] == expected
}

// GenerateBatch generates n CD keys.
func GenerateBatch(n int) []string {
	keys := make([]string, n)
	for i := 0; i < n; i++ {
		keys[i] = GenerateKey()
	}
	return keys
}

// DemoCdKeyGenerator runs a demonstration of key generation and validation.
func DemoCdKeyGenerator() {
	fmt.Println("=== CD Key Generator ===")
	fmt.Println()

	keys := GenerateBatch(5)
	for _, key := range keys {
		fmt.Printf("  %s  valid=%t\n", key, ValidateKey(key))
	}

	tampered := keys[0]
	if tampered[0] == 'Z' {
		tampered = "A" + tampered[1:]
	} else {
		tampered = "Z" + tampered[1:]
	}
	fmt.Printf("\n  Tampered: %s  valid=%t\n", tampered, ValidateKey(tampered))
}
