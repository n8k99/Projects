// Sort File Records Utility — parse delimited records, multi-key sort.
package files

import (
	"sort"
	"strconv"
	"strings"
)

// Order of a sort key.
type Order int

const (
	Asc  Order = 0
	Desc Order = 1
)

// FieldKind is the typed comparison kind for a field.
type FieldKind int

const (
	FKStr FieldKind = 0
	FKInt FieldKind = 1
)

// SortKey describes one sort axis.
type SortKey struct {
	Field int
	Order Order
	Kind  FieldKind
}

// FieldVal is a column value — either an int or a string.
type FieldVal struct {
	IsInt bool
	I     int64
	S     string
}

// ParseField infers int vs string from a raw token.
func ParseField(s string) FieldVal {
	s = strings.TrimSpace(s)
	if n, err := strconv.ParseInt(s, 10, 64); err == nil {
		return FieldVal{IsInt: true, I: n}
	}
	return FieldVal{S: s}
}

// AsStr returns the canonical string form.
func (f FieldVal) AsStr() string {
	if f.IsInt {
		return strconv.FormatInt(f.I, 10)
	}
	return f.S
}

// Record is one row.
type Record struct {
	Fields []FieldVal
}

// ParseRecord parses one comma-separated line.
func ParseRecord(line string) Record {
	parts := strings.Split(line, ",")
	fields := make([]FieldVal, len(parts))
	for i, p := range parts {
		fields[i] = ParseField(p)
	}
	return Record{Fields: fields}
}

// Serialise emits the canonical text form.
func (r Record) Serialise() string {
	parts := make([]string, len(r.Fields))
	for i, f := range r.Fields {
		parts[i] = f.AsStr()
	}
	return strings.Join(parts, ",")
}

// Table is a list of rows with optional header.
type Table struct {
	Header []string
	Rows   []Record
}

// ParseTable parses a delimited text file.
func ParseTable(text string, hasHeader bool) Table {
	lines := []string{}
	for _, l := range strings.Split(text, "\n") {
		if l != "" {
			lines = append(lines, l)
		}
	}
	t := Table{}
	if hasHeader && len(lines) > 0 {
		cols := strings.Split(lines[0], ",")
		for i, c := range cols {
			cols[i] = strings.TrimSpace(c)
		}
		t.Header = cols
		lines = lines[1:]
	}
	t.Rows = make([]Record, len(lines))
	for i, l := range lines {
		t.Rows[i] = ParseRecord(l)
	}
	return t
}

// Serialise emits the canonical text (header + rows + trailing newline).
func (t Table) Serialise() string {
	var b strings.Builder
	if t.Header != nil {
		b.WriteString(strings.Join(t.Header, ","))
		b.WriteByte('\n')
	}
	for _, r := range t.Rows {
		b.WriteString(r.Serialise())
		b.WriteByte('\n')
	}
	return b.String()
}

// FieldIndex resolves a header name (case-insensitive).
func (t Table) FieldIndex(name string) int {
	for i, h := range t.Header {
		if strings.EqualFold(h, name) {
			return i
		}
	}
	return -1
}

func fieldAt(r Record, i int) (FieldVal, bool) {
	if i < len(r.Fields) {
		return r.Fields[i], true
	}
	return FieldVal{}, false
}

func compareFields(a, b FieldVal, ok1, ok2 bool, kind FieldKind) int {
	if !ok1 && !ok2 {
		return 0
	}
	if !ok1 {
		return -1
	}
	if !ok2 {
		return 1
	}
	if kind == FKInt {
		ai := a.I
		if !a.IsInt {
			v, _ := strconv.ParseInt(a.S, 10, 64)
			ai = v
		}
		bi := b.I
		if !b.IsInt {
			v, _ := strconv.ParseInt(b.S, 10, 64)
			bi = v
		}
		switch {
		case ai < bi:
			return -1
		case ai > bi:
			return 1
		default:
			return 0
		}
	}
	as := a.AsStr()
	bs := b.AsStr()
	switch {
	case as < bs:
		return -1
	case as > bs:
		return 1
	default:
		return 0
	}
}

func compareRecords(a, b Record, spec []SortKey) int {
	for _, k := range spec {
		av, aok := fieldAt(a, k.Field)
		bv, bok := fieldAt(b, k.Field)
		c := compareFields(av, bv, aok, bok, k.Kind)
		if k.Order == Desc {
			c = -c
		}
		if c != 0 {
			return c
		}
	}
	return 0
}

// SortBy sorts t.Rows in place by the spec. Stable.
func (t *Table) SortBy(spec []SortKey) {
	sort.SliceStable(t.Rows, func(i, j int) bool {
		return compareRecords(t.Rows[i], t.Rows[j], spec) < 0
	})
}
