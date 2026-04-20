// Event Scheduler and Calendar — intervals, recurrence, overlap detection.
package databases

import "sort"

// ESMsPerDay is 24 * 60 * 60 * 1000.
const ESMsPerDay int64 = 86_400_000

// ESRecurrence is the repeat pattern for an event.
type ESRecurrence int

const (
	ESNone    ESRecurrence = 0
	ESDaily   ESRecurrence = 1
	ESWeekly  ESRecurrence = 2
	ESMonthly ESRecurrence = 3
)

// ESEvent is a calendar event template.
type ESEvent struct {
	ID           int64
	Title        string
	StartMs      int64
	EndMs        int64
	Recurrence   ESRecurrence
	SeriesEndMs  *int64  // nil = no end
}

// Overlaps checks [Start, End) vs [other_start, other_end).
func (e *ESEvent) Overlaps(otherStart, otherEnd int64) bool {
	return e.StartMs < otherEnd && otherStart < e.EndMs
}

// DurationMs returns length of one occurrence.
func (e *ESEvent) DurationMs() int64 { return e.EndMs - e.StartMs }

// ESOccurrence is an expanded event instance within a window.
type ESOccurrence struct {
	EventID int64
	Title   string
	StartMs int64
	EndMs   int64
}

// ESCalendar is a map of event id → event.
type ESCalendar struct {
	Events map[int64]*ESEvent
}

// NewESCalendar returns an empty calendar.
func NewESCalendar() *ESCalendar {
	return &ESCalendar{Events: map[int64]*ESEvent{}}
}

// AddES inserts or replaces an event.
func (c *ESCalendar) AddES(e *ESEvent) { c.Events[e.ID] = e }

func esRecurrenceStep(r ESRecurrence) int64 {
	switch r {
	case ESDaily: return ESMsPerDay
	case ESWeekly: return 7 * ESMsPerDay
	case ESMonthly: return 30 * ESMsPerDay
	}
	return 0
}

func esExpand(e *ESEvent, fromMs, toMs int64) []ESOccurrence {
	var out []ESOccurrence
	duration := e.DurationMs()
	cutoff := int64(1 << 62)
	if e.SeriesEndMs != nil { cutoff = *e.SeriesEndMs }

	if e.Recurrence == ESNone {
		if e.Overlaps(fromMs, toMs) {
			out = append(out, ESOccurrence{EventID: e.ID, Title: e.Title,
				StartMs: e.StartMs, EndMs: e.EndMs})
		}
		return out
	}
	step := esRecurrenceStep(e.Recurrence)
	if step <= 0 { return out }
	start := e.StartMs
	if start < fromMs {
		gap := fromMs - start
		diff := gap - duration
		if diff < 0 { diff = 0 }
		skips := diff / step
		start += skips * step
	}
	for start < toMs && start <= cutoff {
		end := start + duration
		if end > fromMs {
			out = append(out, ESOccurrence{EventID: e.ID, Title: e.Title,
				StartMs: start, EndMs: end})
		}
		start += step
	}
	return out
}

// EventsBetween returns all occurrences overlapping [from, to).
func (c *ESCalendar) EventsBetween(fromMs, toMs int64) []ESOccurrence {
	var all []ESOccurrence
	for _, e := range c.Events {
		all = append(all, esExpand(e, fromMs, toMs)...)
	}
	sort.SliceStable(all, func(i, j int) bool {
		if all[i].StartMs != all[j].StartMs {
			return all[i].StartMs < all[j].StartMs
		}
		return all[i].EventID < all[j].EventID
	})
	return all
}

// FindESConflicts returns event IDs that would conflict with the given window.
func (c *ESCalendar) FindESConflicts(startMs, endMs int64) []int64 {
	var out []int64
	for id, e := range c.Events {
		for _, occ := range esExpand(e, startMs, endMs) {
			if occ.StartMs < endMs && startMs < occ.EndMs {
				out = append(out, id)
				break
			}
		}
	}
	sort.Slice(out, func(i, j int) bool { return out[i] < out[j] })
	return out
}
