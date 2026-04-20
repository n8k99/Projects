// Quick Launcher — ranked launchable items with frecency and file round-trip.
package files

import (
	"fmt"
	"math"
	"sort"
	"strconv"
	"strings"
)

// Entry is a launchable item with persistent usage stats.
type Entry struct {
	ID             int64
	Name           string
	Action         string
	Keywords       []string
	LaunchCount    int
	LastLaunchedMs int64
}

// Match is a query result — entry ID plus combined rank score.
type Match struct {
	EntryID int64
	Score   float64
}

// Launcher indexes entries and ranks them for queries.
type Launcher struct {
	Entries         map[int64]*Entry
	FrecencyWeight  float64
}

// NewLauncher returns an empty launcher with default weights.
func NewLauncher() *Launcher {
	return &Launcher{Entries: map[int64]*Entry{}, FrecencyWeight: 0.1}
}

// Add inserts (or replaces) an entry by ID.
func (l *Launcher) Add(e *Entry) {
	l.Entries[e.ID] = e
}

// Get returns the entry with the given ID, or nil.
func (l *Launcher) Get(id int64) *Entry {
	return l.Entries[id]
}

// Len returns entry count.
func (l *Launcher) Len() int { return len(l.Entries) }

// Query returns matches ranked by text_match + frecency_weight * frecency,
// dropping entries with zero text match. Ties break by smaller ID.
func (l *Launcher) Query(text string, nowMs int64) []Match {
	needle := strings.ToLower(strings.TrimSpace(text))
	var out []Match
	for _, e := range l.Entries {
		tm := textMatch(needle, e)
		if tm == 0 {
			continue
		}
		fr := frecency(e, nowMs)
		out = append(out, Match{e.ID, tm + l.FrecencyWeight*fr})
	}
	sort.Slice(out, func(i, j int) bool {
		if out[i].Score != out[j].Score {
			return out[i].Score > out[j].Score
		}
		return out[i].EntryID < out[j].EntryID
	})
	return out
}

// RecordLaunch bumps the count and stamps the timestamp.
func (l *Launcher) RecordLaunch(id int64, nowMs int64) error {
	e, ok := l.Entries[id]
	if !ok {
		return fmt.Errorf("unknown entry %d", id)
	}
	e.LaunchCount++
	e.LastLaunchedMs = nowMs
	return nil
}

// ToText serialises in a line-based format that round-trips.
func (l *Launcher) ToText() string {
	var b strings.Builder
	b.WriteString("LAUNCHER\n")
	ids := make([]int64, 0, len(l.Entries))
	for id := range l.Entries {
		ids = append(ids, id)
	}
	sort.Slice(ids, func(i, j int) bool { return ids[i] < ids[j] })
	for _, id := range ids {
		e := l.Entries[id]
		b.WriteString("---\n")
		fmt.Fprintf(&b, "id: %d\n", e.ID)
		fmt.Fprintf(&b, "name: %s\n", e.Name)
		fmt.Fprintf(&b, "action: %s\n", e.Action)
		for _, k := range e.Keywords {
			fmt.Fprintf(&b, "keyword: %s\n", k)
		}
		fmt.Fprintf(&b, "count: %d\n", e.LaunchCount)
		fmt.Fprintf(&b, "last: %d\n", e.LastLaunchedMs)
	}
	return b.String()
}

// FromText parses a LAUNCHER text dump.
func FromText(text string) (*Launcher, error) {
	lines := strings.Split(strings.TrimRight(text, "\n"), "\n")
	if len(lines) == 0 || lines[0] != "LAUNCHER" {
		return nil, fmt.Errorf("missing LAUNCHER header")
	}
	l := NewLauncher()
	i := 1
	for i < len(lines) {
		if lines[i] != "---" {
			return nil, fmt.Errorf("expected --- separator at line %d", i+1)
		}
		i++
		e, consumed, err := parseEntry(lines[i:])
		if err != nil {
			return nil, err
		}
		l.Add(e)
		i += consumed
	}
	return l, nil
}

func textMatch(needle string, e *Entry) float64 {
	if needle == "" {
		return 0.5
	}
	nameLc := strings.ToLower(e.Name)
	switch {
	case nameLc == needle:
		return 1.0
	case strings.HasPrefix(nameLc, needle):
		return 0.8
	case strings.Contains(nameLc, needle):
		return 0.6
	}
	for _, k := range e.Keywords {
		klc := strings.ToLower(k)
		if klc == needle {
			return 0.5
		}
		if strings.Contains(klc, needle) {
			return 0.4
		}
	}
	return 0
}

func frecency(e *Entry, nowMs int64) float64 {
	if e.LaunchCount == 0 {
		return 0
	}
	ageMs := nowMs - e.LastLaunchedMs
	if ageMs < 0 {
		ageMs = 0
	}
	ageHours := float64(ageMs) / 3_600_000.0
	return math.Log1p(float64(e.LaunchCount)) / (1.0 + ageHours)
}

func parseEntry(lines []string) (*Entry, int, error) {
	var id int64
	var name, action string
	var keywords []string
	var count int64
	var last int64
	var sawID, sawName, sawAction bool
	consumed := 0
	for _, line := range lines {
		if line == "---" {
			break
		}
		consumed++
		idx := strings.Index(line, ": ")
		if idx < 0 {
			return nil, 0, fmt.Errorf("no `: ` in %q", line)
		}
		key, val := line[:idx], line[idx+2:]
		switch key {
		case "id":
			v, err := strconv.ParseInt(val, 10, 64)
			if err != nil {
				return nil, 0, fmt.Errorf("bad id %q", val)
			}
			id = v
			sawID = true
		case "name":
			name = val
			sawName = true
		case "action":
			action = val
			sawAction = true
		case "keyword":
			keywords = append(keywords, val)
		case "count":
			v, err := strconv.ParseInt(val, 10, 64)
			if err != nil {
				return nil, 0, fmt.Errorf("bad count %q", val)
			}
			count = v
		case "last":
			v, err := strconv.ParseInt(val, 10, 64)
			if err != nil {
				return nil, 0, fmt.Errorf("bad last %q", val)
			}
			last = v
		}
	}
	if !sawID || !sawName || !sawAction {
		return nil, 0, fmt.Errorf("missing id/name/action")
	}
	return &Entry{
		ID: id, Name: name, Action: action,
		Keywords: keywords, LaunchCount: int(count), LastLaunchedMs: last,
	}, consumed, nil
}
