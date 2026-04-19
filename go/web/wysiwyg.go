// WYSIWYG Editor — document model where edit and render are the same structure.
//
// The document is a list of runs, each a contiguous span of runes sharing an
// attribute set. Operations reshape runs — splitting where attrs change,
// merging where adjacent runs end up identical. Positions are character
// offsets; runs are the implementation detail.
package web

import (
	"sort"
	"strings"
)

// Attr is a formatting attribute — bold, italic, a heading level, etc.
type Attr string

const (
	AttrBold      Attr = "b"
	AttrItalic    Attr = "i"
	AttrUnderline Attr = "u"
	AttrH1        Attr = "h1"
	AttrH2        Attr = "h2"
)

// Run is a contiguous span of runes sharing the same attribute set.
type Run struct {
	Text  string
	Attrs []Attr // sorted for stable equality
}

// Document is a formatted text document.
type Document struct {
	Runs []Run
}

// NewDocument returns an empty document.
func NewDocument() *Document { return &Document{} }

// Len returns the total number of runes in the document.
func (d *Document) Len() int {
	n := 0
	for _, r := range d.Runs {
		n += runeCount(r.Text)
	}
	return n
}

// Insert inserts text at rune offset pos with the given attributes.
func (d *Document) Insert(pos int, text string, attrs []Attr) {
	left, right := splitRuns(d.Runs, pos)
	attrs = canonAttrs(attrs)
	d.Runs = normalize(append(append(left, Run{Text: text, Attrs: attrs}), right...))
}

// Delete removes the half-open range [start, end).
func (d *Document) Delete(start, end int) {
	if start >= end {
		return
	}
	left, rest := splitRuns(d.Runs, start)
	_, right := splitRuns(rest, end-start)
	d.Runs = normalize(append(left, right...))
}

// ApplyStyle adds attr to every rune in [start, end).
func (d *Document) ApplyStyle(start, end int, attr Attr) {
	d.modify(start, end, func(attrs []Attr) []Attr {
		for _, a := range attrs {
			if a == attr {
				return attrs
			}
		}
		return canonAttrs(append(append([]Attr{}, attrs...), attr))
	})
}

// RemoveStyle removes attr from every rune in [start, end).
func (d *Document) RemoveStyle(start, end int, attr Attr) {
	d.modify(start, end, func(attrs []Attr) []Attr {
		out := make([]Attr, 0, len(attrs))
		for _, a := range attrs {
			if a != attr {
				out = append(out, a)
			}
		}
		return out
	})
}

func (d *Document) modify(start, end int, fn func([]Attr) []Attr) {
	if start >= end {
		return
	}
	left, rest := splitRuns(d.Runs, start)
	middle, right := splitRuns(rest, end-start)
	for i := range middle {
		middle[i].Attrs = fn(middle[i].Attrs)
	}
	d.Runs = normalize(append(append(left, middle...), right...))
}

// ToPlain returns the document's text with formatting stripped.
func (d *Document) ToPlain() string {
	var sb strings.Builder
	for _, r := range d.Runs {
		sb.WriteString(r.Text)
	}
	return sb.String()
}

// ToHTML renders the document as HTML-like markup.
func (d *Document) ToHTML() string {
	blockOrder := []Attr{AttrH1, AttrH2}
	inlineOrder := []Attr{AttrBold, AttrItalic, AttrUnderline}
	var sb strings.Builder
	for _, r := range d.Runs {
		var open, close strings.Builder
		attrSet := make(map[Attr]bool, len(r.Attrs))
		for _, a := range r.Attrs {
			attrSet[a] = true
		}
		for _, a := range blockOrder {
			if attrSet[a] {
				open.WriteString("<" + string(a) + ">")
				close.WriteString("</" + string(a) + ">")
			}
		}
		for _, a := range inlineOrder {
			if attrSet[a] {
				open.WriteString("<" + string(a) + ">")
				close.WriteString("</" + string(a) + ">")
			}
		}
		// close is reversed because we appended in opening order but need
		// to close in reverse.
		closeStr := reverseTags(close.String())
		sb.WriteString(open.String())
		sb.WriteString(escapeHTML(r.Text))
		sb.WriteString(closeStr)
	}
	return sb.String()
}

func reverseTags(s string) string {
	// Split on "</...>" boundaries and reverse.
	parts := []string{}
	i := 0
	for i < len(s) {
		j := strings.IndexByte(s[i:], '>')
		if j < 0 {
			parts = append(parts, s[i:])
			break
		}
		parts = append(parts, s[i:i+j+1])
		i += j + 1
	}
	var sb strings.Builder
	for k := len(parts) - 1; k >= 0; k-- {
		sb.WriteString(parts[k])
	}
	return sb.String()
}

func splitRuns(runs []Run, pos int) ([]Run, []Run) {
	left := []Run{}
	right := []Run{}
	seen := 0
	for _, r := range runs {
		n := runeCount(r.Text)
		switch {
		case pos >= seen+n:
			left = append(left, r)
		case pos <= seen:
			right = append(right, r)
		default:
			split := pos - seen
			l, rt := splitString(r.Text, split)
			left = append(left, Run{Text: l, Attrs: append([]Attr{}, r.Attrs...)})
			right = append(right, Run{Text: rt, Attrs: append([]Attr{}, r.Attrs...)})
		}
		seen += n
	}
	return left, right
}

func normalize(runs []Run) []Run {
	out := make([]Run, 0, len(runs))
	for _, r := range runs {
		if r.Text == "" {
			continue
		}
		if len(out) > 0 && attrsEqual(out[len(out)-1].Attrs, r.Attrs) {
			out[len(out)-1].Text += r.Text
		} else {
			out = append(out, r)
		}
	}
	return out
}

func canonAttrs(attrs []Attr) []Attr {
	seen := make(map[Attr]bool, len(attrs))
	out := make([]Attr, 0, len(attrs))
	for _, a := range attrs {
		if !seen[a] {
			seen[a] = true
			out = append(out, a)
		}
	}
	sort.Slice(out, func(i, j int) bool { return out[i] < out[j] })
	return out
}

func attrsEqual(a, b []Attr) bool {
	if len(a) != len(b) {
		return false
	}
	for i := range a {
		if a[i] != b[i] {
			return false
		}
	}
	return true
}

func runeCount(s string) int {
	n := 0
	for range s {
		n++
	}
	return n
}

func splitString(s string, runePos int) (string, string) {
	i := 0
	for idx := range s {
		if i == runePos {
			return s[:idx], s[idx:]
		}
		i++
	}
	return s, ""
}

func escapeHTML(s string) string {
	r := strings.NewReplacer("&", "&amp;", "<", "&lt;", ">", "&gt;")
	return r.Replace(s)
}
