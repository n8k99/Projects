// Log File Maker — structured append-only log with level filtering and rotation.
package files

import (
	"fmt"
	"strconv"
	"strings"
)

// LogLevel encodes severity as an int (higher = more severe).
type LogLevel int

const (
	LLDebug LogLevel = 0
	LLInfo  LogLevel = 1
	LLWarn  LogLevel = 2
	LLError LogLevel = 3
)

// AsStr returns the canonical level name.
func (l LogLevel) AsStr() string {
	switch l {
	case LLDebug: return "DEBUG"
	case LLInfo:  return "INFO"
	case LLWarn:  return "WARN"
	case LLError: return "ERROR"
	}
	return "UNKNOWN"
}

// ParseLogLevel parses a level name, or returns -1 invalid.
func ParseLogLevel(s string) LogLevel {
	switch s {
	case "DEBUG": return LLDebug
	case "INFO":  return LLInfo
	case "WARN":  return LLWarn
	case "ERROR": return LLError
	}
	return -1
}

// LogEntry is one line in the log.
type LogEntry struct {
	TimestampMs int64
	Level       LogLevel
	Category    string
	Message     string
}

// String renders tab-delimited.
func (e LogEntry) String() string {
	return fmt.Sprintf("%d\t%s\t%s\t%s", e.TimestampMs, e.Level.AsStr(), e.Category, e.Message)
}

// Logger is an append-only buffer with rotation.
type Logger struct {
	LevelThreshold LogLevel
	MaxEntries     int
	Active         []LogEntry
	Archive        []LogEntry
}

// NewLogger returns an empty logger.
func NewLogger(threshold LogLevel, maxEntries int) *Logger {
	return &Logger{LevelThreshold: threshold, MaxEntries: maxEntries}
}

// Log appends an entry if it passes the threshold, rotating if needed.
func (l *Logger) Log(ts int64, level LogLevel, category, message string) {
	if level < l.LevelThreshold {
		return
	}
	l.Active = append(l.Active, LogEntry{ts, level, category, message})
	for len(l.Active) > l.MaxEntries {
		l.Archive = append(l.Archive, l.Active[0])
		l.Active = l.Active[1:]
	}
}

// QueryLevel returns active entries at or above min.
func (l *Logger) QueryLevel(min LogLevel) []LogEntry {
	var out []LogEntry
	for _, e := range l.Active {
		if e.Level >= min {
			out = append(out, e)
		}
	}
	return out
}

// QueryCategory returns active entries with matching category.
func (l *Logger) QueryCategory(cat string) []LogEntry {
	var out []LogEntry
	for _, e := range l.Active {
		if e.Category == cat {
			out = append(out, e)
		}
	}
	return out
}

// QueryMessageContains returns active entries whose message contains the needle (case-insensitive).
func (l *Logger) QueryMessageContains(needle string) []LogEntry {
	lc := strings.ToLower(needle)
	var out []LogEntry
	for _, e := range l.Active {
		if strings.Contains(strings.ToLower(e.Message), lc) {
			out = append(out, e)
		}
	}
	return out
}

// QueryTimeRange returns entries in [from, to] inclusive.
func (l *Logger) QueryTimeRange(from, to int64) []LogEntry {
	var out []LogEntry
	for _, e := range l.Active {
		if e.TimestampMs >= from && e.TimestampMs <= to {
			out = append(out, e)
		}
	}
	return out
}

// ToText serialises active entries one per line.
func (l *Logger) ToText() string {
	if len(l.Active) == 0 {
		return ""
	}
	var b strings.Builder
	for i, e := range l.Active {
		if i > 0 {
			b.WriteByte('\n')
		}
		b.WriteString(e.String())
	}
	b.WriteByte('\n')
	return b.String()
}

// ParseLogEntries parses tab-delimited log text. Returns nil on any malformed line.
func ParseLogEntries(text string) []LogEntry {
	var out []LogEntry
	for _, line := range strings.Split(text, "\n") {
		if line == "" {
			continue
		}
		parts := strings.SplitN(line, "\t", 4)
		if len(parts) != 4 {
			return nil
		}
		ts, err := strconv.ParseInt(parts[0], 10, 64)
		if err != nil {
			return nil
		}
		level := ParseLogLevel(parts[1])
		if level < 0 {
			return nil
		}
		out = append(out, LogEntry{ts, level, parts[2], parts[3]})
	}
	return out
}
