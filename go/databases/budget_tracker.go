// Budget Tracker — envelope allocation, variance analysis, alert thresholds.
package databases

import "sort"

// BTRollover selects what happens to remaining balance at period end.
type BTRollover int

const (
	BTReset BTRollover = 0
	BTCarry BTRollover = 1
)

// BTEnvelope is a category budget.
type BTEnvelope struct {
	Category          string
	BudgetedCents     int64
	Rollover          BTRollover
	AlertThresholdPct int
}

// NewBTEnvelope returns an envelope with a default 80% alert.
func NewBTEnvelope(category string, budgeted int64) BTEnvelope {
	return BTEnvelope{
		Category:          category,
		BudgetedCents:     budgeted,
		AlertThresholdPct: 80,
	}
}

// BTTransaction is one recorded expense or refund.
type BTTransaction struct {
	TimestampMs int64
	Category    string
	AmountCents int64  // negative = expense, positive = refund
	Note        string
}

// BTCategoryStatus is the computed view for one envelope (or uncategorised).
type BTCategoryStatus struct {
	Category       string
	BudgetedCents  int64
	SpentCents     int64
	RemainingCents int64
	VarianceCents  int64
	VariancePct    int  // -101 sentinel when no budget
	OverBudget     bool
	Alert          bool
}

// BTBudget aggregates envelopes + transactions.
type BTBudget struct {
	Envelopes    map[string]*BTEnvelope
	Transactions []BTTransaction
}

// NewBTBudget returns an empty budget.
func NewBTBudget() *BTBudget {
	return &BTBudget{Envelopes: map[string]*BTEnvelope{}}
}

// AddEnvelope inserts an envelope.
func (b *BTBudget) AddEnvelope(e BTEnvelope) {
	e2 := e
	b.Envelopes[e.Category] = &e2
}

// RecordTx appends a transaction.
func (b *BTBudget) RecordTx(t BTTransaction) {
	b.Transactions = append(b.Transactions, t)
}

// TotalBudgeted sums envelope budgets.
func (b *BTBudget) TotalBudgeted() int64 {
	var sum int64
	for _, e := range b.Envelopes {
		sum += e.BudgetedCents
	}
	return sum
}

// TotalSpentBetween sums expenses (negatives converted) within a window.
func (b *BTBudget) TotalSpentBetween(fromMs, toMs int64) int64 {
	var sum int64
	for _, t := range b.Transactions {
		if t.TimestampMs < fromMs || t.TimestampMs >= toMs { continue }
		sum += -t.AmountCents
	}
	return sum
}

// StatusBetween computes per-envelope status for a window.
func (b *BTBudget) StatusBetween(fromMs, toMs int64) []BTCategoryStatus {
	spent := map[string]int64{}
	for _, t := range b.Transactions {
		if t.TimestampMs < fromMs || t.TimestampMs >= toMs { continue }
		spent[t.Category] += -t.AmountCents
	}

	var out []BTCategoryStatus
	for _, env := range b.Envelopes {
		s := spent[env.Category]
		remaining := env.BudgetedCents - s
		variance := s - env.BudgetedCents
		variancePct := -101
		if env.BudgetedCents != 0 {
			variancePct = int(variance * 100 / env.BudgetedCents)
		}
		over := s > env.BudgetedCents
		alert := false
		if env.BudgetedCents > 0 {
			pct := int(s * 100 / env.BudgetedCents)
			if pct < 0 { pct = 0 }
			alert = pct >= env.AlertThresholdPct
		}
		out = append(out, BTCategoryStatus{
			Category: env.Category, BudgetedCents: env.BudgetedCents,
			SpentCents: s, RemainingCents: remaining, VarianceCents: variance,
			VariancePct: variancePct, OverBudget: over, Alert: alert,
		})
	}
	for cat, s := range spent {
		if _, known := b.Envelopes[cat]; known { continue }
		out = append(out, BTCategoryStatus{
			Category: cat, BudgetedCents: 0, SpentCents: s,
			RemainingCents: -s, VarianceCents: s, VariancePct: -101,
			OverBudget: s > 0, Alert: false,
		})
	}
	sort.Slice(out, func(i, j int) bool { return out[i].Category < out[j].Category })
	return out
}

// StatusAll is StatusBetween over all time.
func (b *BTBudget) StatusAll() []BTCategoryStatus {
	return b.StatusBetween(-(1 << 62), 1 << 62)
}

// Alerts filters StatusAll down to alerting categories.
func (b *BTBudget) Alerts() []BTCategoryStatus {
	var out []BTCategoryStatus
	for _, s := range b.StatusAll() {
		if s.Alert { out = append(out, s) }
	}
	return out
}
