// Code Snippet Manager — templated code fragments with tab-stop expansion.
package files

import (
	"sort"
	"strconv"
	"strings"
)

// SnipTabStop is one cursor position in an expansion.
type SnipTabStop struct {
	Number int
	Offset int
	Length int
}

// SnipExpansion is the output of expanding a snippet.
type SnipExpansion struct {
	Text  string
	Stops []SnipTabStop
}

// SnipSnippet is a code-fragment template.
type SnipSnippet struct {
	Trigger     string
	Language    string
	Tags        []string
	Body        string
	Description string
}

// Expand parses body and returns text + sorted tab stops.
func (s *SnipSnippet) Expand() SnipExpansion {
	return parseAndExpand(s.Body)
}

// SnipLibrary is a searchable snippet store.
type SnipLibrary struct {
	Snippets []SnipSnippet
}

// NewSnipLibrary returns empty library.
func NewSnipLibrary() *SnipLibrary { return &SnipLibrary{} }

// Add appends a snippet.
func (l *SnipLibrary) Add(s SnipSnippet) { l.Snippets = append(l.Snippets, s) }

// FindByTrigger returns the snippet matching trigger + language, or nil.
func (l *SnipLibrary) FindByTrigger(trigger, language string) *SnipSnippet {
	for i := range l.Snippets {
		if l.Snippets[i].Trigger == trigger && l.Snippets[i].Language == language {
			return &l.Snippets[i]
		}
	}
	return nil
}

// FindByLanguage returns all snippets for a language, sorted by trigger.
func (l *SnipLibrary) FindByLanguage(language string) []*SnipSnippet {
	var out []*SnipSnippet
	for i := range l.Snippets {
		if l.Snippets[i].Language == language {
			out = append(out, &l.Snippets[i])
		}
	}
	sort.Slice(out, func(i, j int) bool { return out[i].Trigger < out[j].Trigger })
	return out
}

// FindByTag returns all snippets with the given tag, sorted by trigger.
func (l *SnipLibrary) FindByTag(tag string) []*SnipSnippet {
	var out []*SnipSnippet
	for i := range l.Snippets {
		for _, t := range l.Snippets[i].Tags {
			if t == tag {
				out = append(out, &l.Snippets[i])
				break
			}
		}
	}
	sort.Slice(out, func(i, j int) bool { return out[i].Trigger < out[j].Trigger })
	return out
}

func parseAndExpand(body string) SnipExpansion {
	runes := []rune(body)
	n := len(runes)
	var out strings.Builder
	var stops []SnipTabStop
	i := 0
	for i < n {
		if runes[i] == '$' && i+1 < n {
			if runes[i+1] == '{' {
				if close := findMatching(runes, i+1); close >= 0 {
					inside := string(runes[i+2 : close])
					var numStr, def string
					if colon := strings.Index(inside, ":"); colon >= 0 {
						numStr = inside[:colon]
						def = inside[colon+1:]
					} else {
						numStr = inside
					}
					number, _ := strconv.Atoi(numStr)
					offset := strCharCount(out.String())
					stops = append(stops, SnipTabStop{Number: number, Offset: offset, Length: strCharCount(def)})
					out.WriteString(def)
					i = close + 1
					continue
				}
			} else if runes[i+1] >= '0' && runes[i+1] <= '9' {
				j := i + 1
				for j < n && runes[j] >= '0' && runes[j] <= '9' {
					j++
				}
				number, _ := strconv.Atoi(string(runes[i+1 : j]))
				offset := strCharCount(out.String())
				stops = append(stops, SnipTabStop{Number: number, Offset: offset, Length: 0})
				i = j
				continue
			}
		}
		out.WriteRune(runes[i])
		i++
	}
	sort.SliceStable(stops, func(i, j int) bool {
		a, b := stops[i].Number, stops[j].Number
		if a == 0 && b == 0 { return false }
		if a == 0 { return false }
		if b == 0 { return true }
		return a < b
	})
	return SnipExpansion{Text: out.String(), Stops: stops}
}

func findMatching(runes []rune, open int) int {
	depth := 1
	for i := open + 1; i < len(runes); i++ {
		switch runes[i] {
		case '{':
			depth++
		case '}':
			depth--
			if depth == 0 {
				return i
			}
		}
	}
	return -1
}

func strCharCount(s string) int { return len([]rune(s)) }

// SnipGroupStopsByNumber groups stops by their number for mirroring.
func SnipGroupStopsByNumber(stops []SnipTabStop) map[int][]SnipTabStop {
	out := map[int][]SnipTabStop{}
	for _, s := range stops {
		out[s.Number] = append(out[s.Number], s)
	}
	return out
}
