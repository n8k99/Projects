// Travel Planner System — leg-chained itineraries with integrity validation.
package databases

import "fmt"

// TPTransportMode tags the mode of a leg.
type TPTransportMode int

const (
	TPFlight TPTransportMode = 0
	TPTrain  TPTransportMode = 1
	TPBus    TPTransportMode = 2
	TPCar    TPTransportMode = 3
	TPWalk   TPTransportMode = 4
)

// TPLeg is one segment of a trip.
type TPLeg struct {
	ID          int64
	Mode        TPTransportMode
	Origin      string
	Destination string
	DepartMs    int64
	ArriveMs    int64
	CostCents   int64
	Carrier     string
}

// DurationMs returns the leg's travel duration.
func (l *TPLeg) DurationMs() int64 { return l.ArriveMs - l.DepartMs }

// TPTrip is an ordered sequence of legs with trip-level config.
type TPTrip struct {
	ID                int64
	Title             string
	Legs              []TPLeg
	MinLayoverMinutes int32
}

// NewTPTrip returns an empty trip.
func NewTPTrip(id int64, title string) *TPTrip {
	return &TPTrip{ID: id, Title: title, MinLayoverMinutes: 30}
}

// AddLeg appends a leg.
func (t *TPTrip) AddLeg(l TPLeg) { t.Legs = append(t.Legs, l) }

// TotalSpanMs is last arrive minus first depart.
func (t *TPTrip) TotalSpanMs() int64 {
	if len(t.Legs) == 0 {
		return 0
	}
	return t.Legs[len(t.Legs)-1].ArriveMs - t.Legs[0].DepartMs
}

// InTransitMs sums leg durations.
func (t *TPTrip) InTransitMs() int64 {
	var sum int64
	for _, l := range t.Legs {
		sum += l.DurationMs()
	}
	return sum
}

// TotalCostCents sums leg costs.
func (t *TPTrip) TotalCostCents() int64 {
	var sum int64
	for _, l := range t.Legs {
		sum += l.CostCents
	}
	return sum
}

// TotalLayoverMs is span minus in-transit.
func (t *TPTrip) TotalLayoverMs() int64 {
	return t.TotalSpanMs() - t.InTransitMs()
}

// TPLayover is the gap between two consecutive legs.
type TPLayover struct {
	Location   string
	ArriveMs   int64
	DepartMs   int64
	DurationMs int64
}

// TPLayovers enumerates gaps.
func TPLayovers(t *TPTrip) []TPLayover {
	var out []TPLayover
	for i := 0; i+1 < len(t.Legs); i++ {
		prev := t.Legs[i]
		next := t.Legs[i+1]
		out = append(out, TPLayover{
			Location: prev.Destination, ArriveMs: prev.ArriveMs,
			DepartMs: next.DepartMs, DurationMs: next.DepartMs - prev.ArriveMs,
		})
	}
	return out
}

// TPIssueSeverity ladders the severity.
type TPIssueSeverity int

const (
	TPIInfo    TPIssueSeverity = 0
	TPIWarning TPIssueSeverity = 1
	TPIError   TPIssueSeverity = 2
)

// TPIssueKind identifies the problem.
type TPIssueKind int

const (
	TPIKLocationMismatch    TPIssueKind = 0
	TPIKTimeConflict        TPIssueKind = 1
	TPIKTightLayover        TPIssueKind = 2
	TPIKZeroDurationLeg     TPIssueKind = 3
	TPIKNegativeDurationLeg TPIssueKind = 4
)

// TPIssue is one validation finding.
type TPIssue struct {
	Severity TPIssueSeverity
	Kind     TPIssueKind
	LegIdx   int
	Detail   string
}

// ValidateTPTrip walks the chain and collects issues.
func ValidateTPTrip(trip *TPTrip) []TPIssue {
	var out []TPIssue
	for i, leg := range trip.Legs {
		d := leg.DurationMs()
		if d == 0 {
			out = append(out, TPIssue{Severity: TPIWarning, Kind: TPIKZeroDurationLeg, LegIdx: i})
		} else if d < 0 {
			out = append(out, TPIssue{Severity: TPIError, Kind: TPIKNegativeDurationLeg, LegIdx: i})
		}
	}
	for i := 0; i+1 < len(trip.Legs); i++ {
		prev := trip.Legs[i]
		next := trip.Legs[i+1]
		idx := i + 1
		if prev.Destination != next.Origin {
			out = append(out, TPIssue{
				Severity: TPIError, Kind: TPIKLocationMismatch, LegIdx: idx,
				Detail: fmt.Sprintf("%s -> %s", prev.Destination, next.Origin),
			})
		}
		if next.DepartMs < prev.ArriveMs {
			out = append(out, TPIssue{
				Severity: TPIError, Kind: TPIKTimeConflict, LegIdx: idx,
				Detail: fmt.Sprintf("arrive=%d depart=%d", prev.ArriveMs, next.DepartMs),
			})
		} else {
			layoverMs := next.DepartMs - prev.ArriveMs
			layoverMin := layoverMs / 60000
			if layoverMs > 0 && layoverMin < int64(trip.MinLayoverMinutes) {
				out = append(out, TPIssue{
					Severity: TPIWarning, Kind: TPIKTightLayover, LegIdx: idx,
					Detail: fmt.Sprintf("%d min", layoverMin),
				})
			}
		}
	}
	return out
}
