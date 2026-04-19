// Password Safe — encrypted-at-rest credential store with session FSM.
// Not real crypto; byte-sum + XOR demonstrates structure.
package web

import (
	"strings"
)

// PSStrength categorises password strength.
type PSStrength int

const (
	PSWeak PSStrength = iota
	PSFair
	PSStrong
	PSVeryStrong
)

// PSGeneratePolicy configures password generation.
type PSGeneratePolicy struct {
	Length  int
	Lower   bool
	Upper   bool
	Digits  bool
	Symbols bool
}

// PSEntrySummary is a view over an entry that omits the password.
type PSEntrySummary struct {
	ID             int64
	Service        string
	Username       string
	Notes          string
	CreatedAtMs    int64
	ModifiedAtMs   int64
}

type psEntry struct {
	id                int64
	service, username string
	encrypted         []byte
	notes             string
	createdAtMs       int64
	modifiedAtMs      int64
}

// PSSafe is an encrypted credential store.
type PSSafe struct {
	salt              []byte
	verifier          []byte
	entries           []*psEntry
	nextID            int64
	key               []byte
	lastActivityMs    int64
	autoLockAfterMs   int64
}

// NewPSSafe creates a new locked safe with the given master password.
func NewPSSafe(master string, autoLockAfterMs int64) *PSSafe {
	salt := make([]byte, 16)
	for i := range salt {
		salt[i] = byte(i)
	}
	return &PSSafe{
		salt:            salt,
		verifier:        psVerifierHash(master, salt),
		nextID:          1,
		autoLockAfterMs: autoLockAfterMs,
	}
}

// IsLocked returns true when the safe is locked.
func (s *PSSafe) IsLocked() bool { return s.key == nil }

// Unlock attempts to unlock the safe; returns success.
func (s *PSSafe) Unlock(master string, nowMs int64) bool {
	if !bytesEqual(psVerifierHash(master, s.salt), s.verifier) {
		return false
	}
	s.key = psDeriveKey(master, s.salt)
	s.lastActivityMs = nowMs
	return true
}

// Lock locks the safe.
func (s *PSSafe) Lock() { s.key = nil }

// CheckAutoLock locks the safe if inactivity exceeds the auto-lock window.
func (s *PSSafe) CheckAutoLock(nowMs int64) {
	if s.key != nil && nowMs >= s.lastActivityMs+s.autoLockAfterMs {
		s.Lock()
	}
}

func (s *PSSafe) touch(nowMs int64) []byte {
	if s.key == nil {
		return nil
	}
	s.lastActivityMs = nowMs
	return s.key
}

// AddEntry adds a new entry. Returns id or 0 if locked.
func (s *PSSafe) AddEntry(service, username, password, notes string, nowMs int64) int64 {
	key := s.touch(nowMs)
	if key == nil {
		return 0
	}
	id := s.nextID
	s.nextID++
	s.entries = append(s.entries, &psEntry{
		id: id, service: service, username: username,
		encrypted: psXOR([]byte(password), key),
		notes:     notes,
		createdAtMs:  nowMs,
		modifiedAtMs: nowMs,
	})
	return id
}

// GetPassword decrypts and returns the password; "" if locked or not found.
func (s *PSSafe) GetPassword(id int64, nowMs int64) string {
	key := s.touch(nowMs)
	if key == nil {
		return ""
	}
	for _, e := range s.entries {
		if e.id == id {
			return string(psXOR(e.encrypted, key))
		}
	}
	return ""
}

// GetSummary returns a summary (no password) for the given id.
func (s *PSSafe) GetSummary(id int64) *PSEntrySummary {
	for _, e := range s.entries {
		if e.id == id {
			return &PSEntrySummary{
				ID: e.id, Service: e.service, Username: e.username,
				Notes: e.notes,
				CreatedAtMs: e.createdAtMs, ModifiedAtMs: e.modifiedAtMs,
			}
		}
	}
	return nil
}

// Search returns summaries matching the query (case-insensitive).
func (s *PSSafe) Search(query string, nowMs int64) []*PSEntrySummary {
	if s.touch(nowMs) == nil {
		return nil
	}
	q := strings.ToLower(query)
	var out []*PSEntrySummary
	for _, e := range s.entries {
		if strings.Contains(strings.ToLower(e.service), q) ||
			strings.Contains(strings.ToLower(e.username), q) ||
			strings.Contains(strings.ToLower(e.notes), q) {
			out = append(out, &PSEntrySummary{
				ID: e.id, Service: e.service, Username: e.username,
				Notes: e.notes,
				CreatedAtMs: e.createdAtMs, ModifiedAtMs: e.modifiedAtMs,
			})
		}
	}
	return out
}

// EntryCount returns the number of entries in the safe.
func (s *PSSafe) EntryCount() int { return len(s.entries) }

// PSGeneratePassword generates a deterministic pseudo-random password.
func PSGeneratePassword(p PSGeneratePolicy, seed uint64) string {
	var sb strings.Builder
	if p.Lower {
		sb.WriteString("abcdefghijklmnopqrstuvwxyz")
	}
	if p.Upper {
		sb.WriteString("ABCDEFGHIJKLMNOPQRSTUVWXYZ")
	}
	if p.Digits {
		sb.WriteString("0123456789")
	}
	if p.Symbols {
		sb.WriteString("!@#$%^&*()-_=+[]{}")
	}
	alphabet := sb.String()
	if alphabet == "" {
		return ""
	}
	runes := []rune(alphabet)
	state := seed*6364136223846793005 + 1442695040888963407
	var out strings.Builder
	for i := 0; i < p.Length; i++ {
		state = state*6364136223846793005 + 1442695040888963407
		out.WriteRune(runes[(state>>32)%uint64(len(runes))])
	}
	return out.String()
}

// PSPasswordStrength classifies a password.
func PSPasswordStrength(pw string) PSStrength {
	classes := 0
	hasLower, hasUpper, hasDigit, hasSymbol := false, false, false, false
	for _, c := range pw {
		switch {
		case c >= 'a' && c <= 'z':
			hasLower = true
		case c >= 'A' && c <= 'Z':
			hasUpper = true
		case c >= '0' && c <= '9':
			hasDigit = true
		default:
			if c != ' ' {
				hasSymbol = true
			}
		}
	}
	if hasLower {
		classes++
	}
	if hasUpper {
		classes++
	}
	if hasDigit {
		classes++
	}
	if hasSymbol {
		classes++
	}
	length := len([]rune(pw))
	switch {
	case length < 8:
		return PSWeak
	case length < 12 && classes < 3:
		return PSWeak
	case length >= 20:
		return PSVeryStrong
	case length >= 14 && classes >= 3:
		return PSStrong
	case classes >= 4:
		return PSStrong
	default:
		return PSFair
	}
}

func psVerifierHash(master string, salt []byte) []byte {
	out := make([]byte, 32)
	m := []byte(master)
	for r := uint64(0); r < 32; r++ {
		h := r
		for _, b := range m {
			h = (h * 131) + uint64(b)
		}
		for _, b := range salt {
			h = (h * 131) + uint64(b)
		}
		out[r] = byte(h & 0xFF)
	}
	return out
}

func psDeriveKey(master string, salt []byte) []byte {
	out := make([]byte, 64)
	m := []byte(master)
	for i := uint64(0); i < 64; i++ {
		h := 128 + i
		for _, b := range m {
			h = (h * 197) + uint64(b)
		}
		for _, b := range salt {
			h = (h * 197) + uint64(b)
		}
		out[i] = byte(h & 0xFF)
	}
	return out
}

func psXOR(data, key []byte) []byte {
	out := make([]byte, len(data))
	for i, b := range data {
		out[i] = b ^ key[i%len(key)]
	}
	return out
}

func bytesEqual(a, b []byte) bool {
	if len(a) != len(b) {
		return false
	}
	for i := range a {
		if a[i] != b[i] {
			return false
		}
	}
	return true
}
