package numbers

import "math"

// Payment represents a single period in an amortization schedule.
type Payment struct {
	Period    int
	Payment   float64
	Principal float64
	Interest  float64
	Balance   float64
}

// MonthlyPayment calculates the fixed monthly payment for a mortgage.
func MonthlyPayment(principal, annualRate float64, years int) float64 {
	if annualRate == 0 {
		return principal / float64(years*12)
	}
	r := annualRate / 100.0 / 12.0
	n := float64(years * 12)
	factor := math.Pow(1+r, n)
	return principal * r * factor / (factor - 1)
}

// AmortizationSchedule generates the full schedule.
func AmortizationSchedule(principal, annualRate float64, years int) []Payment {
	pmt := MonthlyPayment(principal, annualRate, years)
	r := annualRate / 100.0 / 12.0
	balance := principal
	n := years * 12
	schedule := make([]Payment, 0, n)

	for period := 1; period <= n; period++ {
		interest := balance * r
		princ := pmt - interest
		balance -= princ
		if period == n {
			princ += balance
			balance = 0
		}
		schedule = append(schedule, Payment{period, pmt, princ, interest, balance})
	}
	return schedule
}

// PaceResult holds the result of a pace check.
type PaceResult struct {
	ExpectedByNow float64
	Actual        float64
	Delta         float64
	PacePercent   float64
	OnTrack       bool
}

// PaceCheck checks if cumulative payments are on pace to meet target.
func PaceCheck(target float64, periods, currentPeriod int, cumulative float64) PaceResult {
	expected := target * float64(currentPeriod) / float64(periods)
	delta := cumulative - expected
	pacePct := 0.0
	if expected > 0 {
		pacePct = cumulative / expected * 100
	}
	return PaceResult{
		ExpectedByNow: expected,
		Actual:        cumulative,
		Delta:         delta,
		PacePercent:   pacePct,
		OnTrack:       delta >= 0,
	}
}
