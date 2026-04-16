package numbers

import "math"

// TaxBracketDetail holds the tax computation for a single bracket.
type TaxBracketDetail struct {
	BracketFloor float64
	Rate         float64
	Taxable      float64
	Tax          float64
}

// TaxResult holds the full result of a progressive tax calculation.
type TaxResult struct {
	TotalTax      float64
	EffectiveRate float64
	MarginalRate  float64
	Breakdown     []TaxBracketDetail
}

// 2024 US federal single-filer brackets: (upper limit, rate).
var taxBrackets = [][2]float64{
	{11_600, 0.10},
	{47_150, 0.12},
	{100_525, 0.22},
	{191_950, 0.24},
	{243_725, 0.32},
	{609_350, 0.35},
	{math.Inf(1), 0.37},
}

// CalculateTax computes progressive income tax for a given income.
func CalculateTax(income float64) TaxResult {
	if income <= 0 {
		return TaxResult{}
	}

	var breakdown []TaxBracketDetail
	totalTax := 0.0
	prevLimit := 0.0
	marginalRate := 0.0

	for _, bracket := range taxBrackets {
		limit, rate := bracket[0], bracket[1]
		if income <= prevLimit {
			break
		}
		taxable := math.Min(income, limit) - prevLimit
		tax := taxable * rate
		totalTax += tax
		marginalRate = rate
		breakdown = append(breakdown, TaxBracketDetail{
			BracketFloor: prevLimit,
			Rate:         rate,
			Taxable:      taxable,
			Tax:          tax,
		})
		prevLimit = limit
	}

	effectiveRate := totalTax / income

	return TaxResult{
		TotalTax:      totalTax,
		EffectiveRate: effectiveRate,
		MarginalRate:  marginalRate,
		Breakdown:     breakdown,
	}
}
