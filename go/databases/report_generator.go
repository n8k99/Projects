// Report Generator — grouped reports with subtotal and grand-total rows.
package databases

import (
	"fmt"
	"sort"
	"strings"
)

// RGValueKind tags a report value.
type RGValueKind int

const (
	RGVInt  RGValueKind = 0
	RGVText RGValueKind = 1
	RGVNull RGValueKind = 2
)

// RGValue is a typed value in a record.
type RGValue struct {
	Kind RGValueKind
	Int  int64
	Text string
}

// RGInt factory.
func RGInt(n int64) RGValue { return RGValue{Kind: RGVInt, Int: n} }

// RGText factory.
func RGText(s string) RGValue { return RGValue{Kind: RGVText, Text: s} }

// RGNull factory.
func RGNull() RGValue { return RGValue{Kind: RGVNull} }

// AsInt returns (value, true) if the underlying kind is int.
func (v RGValue) AsInt() (int64, bool) {
	if v.Kind == RGVInt { return v.Int, true }
	return 0, false
}

// AsText returns a textual representation.
func (v RGValue) AsText() string {
	switch v.Kind {
	case RGVText: return v.Text
	case RGVInt: return fmt.Sprintf("%d", v.Int)
	default: return ""
	}
}

// RGRecord is one row in the input data.
type RGRecord map[string]RGValue

// RGColumnKind tags a report column.
type RGColumnKind int

const (
	RGCPlain        RGColumnKind = 0
	RGCSumAggregate RGColumnKind = 1
)

// RGColumnDef describes one column in the report.
type RGColumnDef struct {
	Name         string
	Field        string
	Kind         RGColumnKind
	FormatCents  bool
}

// RGRowKind tags the role of an output row.
type RGRowKind int

const (
	RGRHeader   RGRowKind = 0
	RGRData     RGRowKind = 1
	RGRSubtotal RGRowKind = 2
	RGRTotal    RGRowKind = 3
)

// RGReportRow is one rendered row.
type RGReportRow struct {
	Kind       RGRowKind
	Cells      []string
	GroupLabel string
}

// RGReportSpec describes how to build the report.
type RGReportSpec struct {
	Title   string
	Columns []RGColumnDef
	GroupBy string  // "" = no grouping
}

// RGReport is the generated output.
type RGReport struct {
	Title string
	Rows  []RGReportRow
}

func rgFormatInt(n int64, asCents bool) string {
	if !asCents {
		return fmt.Sprintf("%d", n)
	}
	sign := ""
	abs := n
	if abs < 0 { sign = "-"; abs = -abs }
	return fmt.Sprintf("$%s%d.%02d", sign, abs/100, abs%100)
}

func rgFormatCell(v RGValue, col RGColumnDef) string {
	if v.Kind == RGVNull { return "" }
	if v.Kind == RGVInt { return rgFormatInt(v.Int, col.FormatCents) }
	return v.Text
}

// RGGenerate builds the report from records.
func RGGenerate(records []RGRecord, spec *RGReportSpec) *RGReport {
	var rows []RGReportRow
	headerCells := make([]string, len(spec.Columns))
	for i, c := range spec.Columns { headerCells[i] = c.Name }
	rows = append(rows, RGReportRow{Kind: RGRHeader, Cells: headerCells})

	type group struct {
		Key     string
		Records []RGRecord
	}
	var groups []group
	if spec.GroupBy != "" {
		m := map[string][]RGRecord{}
		for _, r := range records {
			key := "(none)"
			if v, ok := r[spec.GroupBy]; ok {
				if t := v.AsText(); t != "" { key = t }
			}
			m[key] = append(m[key], r)
		}
		keys := make([]string, 0, len(m))
		for k := range m { keys = append(keys, k) }
		sort.Strings(keys)
		for _, k := range keys { groups = append(groups, group{Key: k, Records: m[k]}) }
	} else {
		groups = []group{{Key: "__all__", Records: records}}
	}

	grandTotals := map[string]int64{}

	for _, g := range groups {
		for _, record := range g.Records {
			cells := make([]string, len(spec.Columns))
			for i, col := range spec.Columns {
				cells[i] = rgFormatCell(record[col.Field], col)
			}
			label := ""
			if spec.GroupBy != "" { label = g.Key }
			rows = append(rows, RGReportRow{Kind: RGRData, Cells: cells, GroupLabel: label})
		}
		if spec.GroupBy != "" {
			subtotals := map[string]int64{}
			for _, record := range g.Records {
				for _, col := range spec.Columns {
					if col.Kind == RGCSumAggregate {
						if n, ok := record[col.Field].AsInt(); ok {
							subtotals[col.Field] += n
							grandTotals[col.Field] += n
						}
					}
				}
			}
			cells := make([]string, len(spec.Columns))
			for i, col := range spec.Columns {
				switch {
				case i == 0:
					cells[i] = g.Key + " subtotal"
				case col.Kind == RGCSumAggregate:
					cells[i] = rgFormatInt(subtotals[col.Field], col.FormatCents)
				default:
					cells[i] = ""
				}
			}
			rows = append(rows, RGReportRow{Kind: RGRSubtotal, Cells: cells, GroupLabel: g.Key})
		}
	}

	hasAggregate := false
	for _, c := range spec.Columns {
		if c.Kind == RGCSumAggregate { hasAggregate = true; break }
	}
	if hasAggregate {
		if spec.GroupBy == "" {
			for _, record := range records {
				for _, col := range spec.Columns {
					if col.Kind == RGCSumAggregate {
						if n, ok := record[col.Field].AsInt(); ok {
							grandTotals[col.Field] += n
						}
					}
				}
			}
		}
		cells := make([]string, len(spec.Columns))
		for i, col := range spec.Columns {
			switch {
			case i == 0:
				cells[i] = "TOTAL"
			case col.Kind == RGCSumAggregate:
				cells[i] = rgFormatInt(grandTotals[col.Field], col.FormatCents)
			default:
				cells[i] = ""
			}
		}
		rows = append(rows, RGReportRow{Kind: RGRTotal, Cells: cells})
	}

	return &RGReport{Title: spec.Title, Rows: rows}
}

// RGRender emits a tab-delimited text table prefixed by kind markers.
func RGRender(r *RGReport) string {
	var b strings.Builder
	if r.Title != "" {
		b.WriteString(r.Title)
		b.WriteByte('\n')
	}
	for _, row := range r.Rows {
		var prefix string
		switch row.Kind {
		case RGRHeader: prefix = "#"
		case RGRData: prefix = " "
		case RGRSubtotal: prefix = "~"
		case RGRTotal: prefix = "="
		}
		b.WriteString(prefix)
		b.WriteByte('\t')
		b.WriteString(strings.Join(row.Cells, "\t"))
		b.WriteByte('\n')
	}
	return b.String()
}
