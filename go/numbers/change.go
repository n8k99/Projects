package numbers

import "sort"

// MakeChange breaks amount (in cents) into fewest coins possible.
func MakeChange(amount int, denominations []int) map[int]int {
	denoms := make([]int, len(denominations))
	copy(denoms, denominations)
	sort.Sort(sort.Reverse(sort.IntSlice(denoms)))

	remaining := amount
	result := make(map[int]int)

	for _, denom := range denoms {
		count := remaining / denom
		if count > 0 {
			result[denom] = count
			remaining -= count * denom
		}
	}
	return result
}

// Obligation represents a bill with a name, amount, and due day.
type Obligation struct {
	Name   string
	Amount float64
	DueDay int
}

// CoverResult holds the result of covering obligations.
type CoverResult struct {
	Covered    []Obligation
	Uncovered  []Obligation
	Shortfalls []float64
	Remaining  float64
	AllCovered bool
}

// CoverObligations covers obligations in chronological order.
func CoverObligations(available float64, obligations []Obligation) CoverResult {
	sorted := make([]Obligation, len(obligations))
	copy(sorted, obligations)
	sort.Slice(sorted, func(i, j int) bool {
		return sorted[i].DueDay < sorted[j].DueDay
	})

	remaining := available
	var covered, uncovered []Obligation
	var shortfalls []float64

	for _, ob := range sorted {
		if remaining >= ob.Amount {
			remaining -= ob.Amount
			covered = append(covered, ob)
		} else {
			uncovered = append(uncovered, ob)
			shortfalls = append(shortfalls, ob.Amount-remaining)
		}
	}

	return CoverResult{
		Covered:    covered,
		Uncovered:  uncovered,
		Shortfalls: shortfalls,
		Remaining:  remaining,
		AllCovered: len(uncovered) == 0,
	}
}
