// CAPTCHA Maker — challenge-response with expiration and attempt limits.
package web

import (
	"fmt"
	"strings"
)

// CapKind classifies a challenge type.
type CapKind int

const (
	CapMath CapKind = iota
	CapWordRecall
	CapReverseString
	CapSequenceCount
)

// CapChallenge is an issued CAPTCHA challenge.
type CapChallenge struct {
	ID            int64
	Kind          CapKind
	Prompt        string
	Expected      string
	CreatedAtMs   int64
	ExpiresAtMs   int64
	AttemptsUsed  int
	MaxAttempts   int
	Solved        bool
}

// CapVerifyKind is the outcome of verifying an answer.
type CapVerifyKind int

const (
	CapSuccess CapVerifyKind = iota
	CapWrongAnswer
	CapExpired
	CapAlreadySolved
	CapTooManyAttempts
	CapUnknown
)

// CapVerifyResult carries the outcome + remaining attempts when wrong.
type CapVerifyResult struct {
	Kind               CapVerifyKind
	AttemptsRemaining  int // only meaningful when Kind == CapWrongAnswer
}

// CapService owns the challenge registry.
type CapService struct {
	challenges    []*CapChallenge
	nextID        int64
	defaultTTLMs  int64
	maxAttempts   int
	rngState      uint64
}

// NewCapService builds a new service with the given TTL, attempt cap, and seed.
func NewCapService(defaultTTLMs int64, maxAttempts int, seed uint64) *CapService {
	return &CapService{
		nextID: 1, defaultTTLMs: defaultTTLMs,
		maxAttempts: maxAttempts, rngState: seed,
	}
}

func (s *CapService) nextRandom() uint64 {
	s.rngState = s.rngState*6364136223846793005 + 1442695040888963407
	return s.rngState >> 32
}

func (s *CapService) generate(kind CapKind) (prompt, expected string) {
	switch kind {
	case CapMath:
		a := s.nextRandom()%20 + 1
		b := s.nextRandom()%20 + 1
		op := s.nextRandom() % 3
		switch op {
		case 0:
			return fmt.Sprintf("What is %d + %d?", a, b), fmt.Sprintf("%d", a+b)
		case 1:
			hi, lo := a, b
			if b > a {
				hi, lo = b, a
			}
			return fmt.Sprintf("What is %d - %d?", hi, lo), fmt.Sprintf("%d", hi-lo)
		default:
			return fmt.Sprintf("What is %d × %d?", a, b), fmt.Sprintf("%d", a*b)
		}
	case CapWordRecall:
		words := []string{"butterfly", "chronicle", "lighthouse", "symphony", "mountain"}
		w := words[s.nextRandom()%uint64(len(words))]
		return fmt.Sprintf("Type this word: %s", w), w
	case CapReverseString:
		inputs := []string{"hello", "rosetta", "captcha", "innate"}
		w := inputs[s.nextRandom()%uint64(len(inputs))]
		rev := reverseStringRunes(w)
		return fmt.Sprintf("Reverse this: %s", w), rev
	case CapSequenceCount:
		length := s.nextRandom()%6 + 5
		var seq strings.Builder
		digits := 0
		for i := uint64(0); i < length; i++ {
			r := s.nextRandom() % 36
			if r < 10 {
				seq.WriteByte(byte('0' + r))
				digits++
			} else {
				seq.WriteByte(byte('A' + (r - 10)))
			}
		}
		return fmt.Sprintf("How many digits in: %s?", seq.String()), fmt.Sprintf("%d", digits)
	}
	return "", ""
}

func reverseStringRunes(s string) string {
	runes := []rune(s)
	for i, j := 0, len(runes)-1; i < j; i, j = i+1, j-1 {
		runes[i], runes[j] = runes[j], runes[i]
	}
	return string(runes)
}

// Issue creates and returns a new challenge.
func (s *CapService) Issue(kind CapKind, nowMs int64) *CapChallenge {
	id := s.nextID
	s.nextID++
	prompt, expected := s.generate(kind)
	c := &CapChallenge{
		ID: id, Kind: kind, Prompt: prompt, Expected: expected,
		CreatedAtMs: nowMs,
		ExpiresAtMs: nowMs + s.defaultTTLMs,
		MaxAttempts: s.maxAttempts,
	}
	s.challenges = append(s.challenges, c)
	return c
}

// Verify checks a user's answer against a challenge.
func (s *CapService) Verify(id int64, userAnswer string, nowMs int64) CapVerifyResult {
	var c *CapChallenge
	for _, ch := range s.challenges {
		if ch.ID == id {
			c = ch
			break
		}
	}
	if c == nil {
		return CapVerifyResult{Kind: CapUnknown}
	}
	if c.Solved {
		return CapVerifyResult{Kind: CapAlreadySolved}
	}
	if nowMs > c.ExpiresAtMs {
		return CapVerifyResult{Kind: CapExpired}
	}
	if c.AttemptsUsed >= c.MaxAttempts {
		return CapVerifyResult{Kind: CapTooManyAttempts}
	}
	c.AttemptsUsed++
	if strings.EqualFold(strings.TrimSpace(c.Expected), strings.TrimSpace(userAnswer)) {
		c.Solved = true
		return CapVerifyResult{Kind: CapSuccess}
	}
	return CapVerifyResult{
		Kind:              CapWrongAnswer,
		AttemptsRemaining: c.MaxAttempts - c.AttemptsUsed,
	}
}

// Challenge returns the challenge with the given id, or nil.
func (s *CapService) Challenge(id int64) *CapChallenge {
	for _, c := range s.challenges {
		if c.ID == id {
			return c
		}
	}
	return nil
}

// Cleanup removes solved or expired challenges; returns count removed.
func (s *CapService) Cleanup(nowMs int64) int {
	before := len(s.challenges)
	kept := s.challenges[:0]
	for _, c := range s.challenges {
		if !c.Solved && nowMs <= c.ExpiresAtMs {
			kept = append(kept, c)
		}
	}
	s.challenges = kept
	return before - len(s.challenges)
}

// ActiveCount returns the number of live challenges.
func (s *CapService) ActiveCount() int { return len(s.challenges) }
