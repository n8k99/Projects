// Transaction Averages — grouped aggregation over financial records.
package files

import (
	"math"
	"sort"
	"strconv"
	"strings"
)

// Transaction is date,category,amount_cents.
type Transaction struct {
	Date         string
	Category     string
	AmountCents  int64
}

// Month returns YYYY-MM.
func (t Transaction) Month() string {
	if len(t.Date) >= 7 {
		return t.Date[:7]
	}
	return t.Date
}

// Stats holds count/sum/mean/min/max.
type Stats struct {
	Count     int
	SumCents  int64
	MeanCents float64
	MinCents  int64
	MaxCents  int64
}

// NewStats returns an empty Stats.
func NewStats() *Stats {
	return &Stats{MinCents: math.MaxInt64, MaxCents: math.MinInt64}
}

// Observe records one amount.
func (s *Stats) Observe(amount int64) {
	s.Count++
	s.SumCents += amount
	if amount < s.MinCents {
		s.MinCents = amount
	}
	if amount > s.MaxCents {
		s.MaxCents = amount
	}
}

// Finalise computes the mean.
func (s *Stats) Finalise() {
	if s.Count == 0 {
		s.MeanCents = 0
	} else {
		s.MeanCents = float64(s.SumCents) / float64(s.Count)
	}
}

// ParseTransaction parses one line. Returns nil on error.
func ParseTransaction(line string) *Transaction {
	parts := strings.SplitN(line, ",", 3)
	if len(parts) != 3 {
		return nil
	}
	date := strings.TrimSpace(parts[0])
	category := strings.TrimSpace(parts[1])
	if date == "" || category == "" {
		return nil
	}
	amount, err := strconv.ParseInt(strings.TrimSpace(parts[2]), 10, 64)
	if err != nil {
		return nil
	}
	return &Transaction{Date: date, Category: category, AmountCents: amount}
}

// ParseTransactions parses many lines, dropping empty and malformed.
func ParseTransactions(text string) []Transaction {
	out := []Transaction{}
	for _, l := range strings.Split(text, "\n") {
		if l == "" {
			continue
		}
		if t := ParseTransaction(l); t != nil {
			out = append(out, *t)
		}
	}
	return out
}

// Group is one (key, stats) pair.
type Group struct {
	Key   string
	Stats Stats
}

// GroupBy groups transactions by a key function. Returns sorted-by-key.
func GroupBy(txns []Transaction, keyFn func(Transaction) string) []Group {
	m := map[string]*Stats{}
	for _, t := range txns {
		k := keyFn(t)
		if _, ok := m[k]; !ok {
			m[k] = NewStats()
		}
		m[k].Observe(t.AmountCents)
	}
	keys := make([]string, 0, len(m))
	for k := range m {
		keys = append(keys, k)
	}
	sort.Strings(keys)
	out := make([]Group, 0, len(keys))
	for _, k := range keys {
		m[k].Finalise()
		out = append(out, Group{Key: k, Stats: *m[k]})
	}
	return out
}

// ByCategory groups by category.
func ByCategory(txns []Transaction) []Group {
	return GroupBy(txns, func(t Transaction) string { return t.Category })
}

// ByMonth groups by YYYY-MM.
func ByMonth(txns []Transaction) []Group {
	return GroupBy(txns, func(t Transaction) string { return t.Month() })
}

// TotalStats across all transactions.
func TotalStats(txns []Transaction) Stats {
	s := NewStats()
	for _, t := range txns {
		s.Observe(t.AmountCents)
	}
	s.Finalise()
	return *s
}

// TopNByMean returns the n groups with highest mean.
func TopNByMean(groups []Group, n int) []Group {
	out := make([]Group, len(groups))
	copy(out, groups)
	sort.SliceStable(out, func(i, j int) bool {
		return out[i].Stats.MeanCents > out[j].Stats.MeanCents
	})
	if n > len(out) {
		n = len(out)
	}
	return out[:n]
}
