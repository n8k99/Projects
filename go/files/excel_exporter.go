// Excel Spreadsheet Exporter — cell-addressed grid, formulae, CSV export.
package files

import (
	"fmt"
	"strconv"
	"strings"
)

// XLCellKind tags a spreadsheet cell.
type XLCellKind int

const (
	XLBlank   XLCellKind = 0
	XLNumber  XLCellKind = 1
	XLText    XLCellKind = 2
	XLFormula XLCellKind = 3
)

// XLCell is one spreadsheet cell.
type XLCell struct {
	Kind    XLCellKind
	Number  float64
	Text    string
	Formula string
}

// XLValueKind tags an evaluated value.
type XLValueKind int

const (
	XLValEmpty  XLValueKind = 0
	XLValNumber XLValueKind = 1
	XLValText   XLValueKind = 2
	XLValError  XLValueKind = 3
)

// XLValue is an evaluated value.
type XLValue struct {
	Kind   XLValueKind
	Number float64
	Text   string
}

// AsNumber extracts a float if possible.
func (v XLValue) AsNumber() (float64, bool) {
	if v.Kind == XLValNumber { return v.Number, true }
	if v.Kind == XLValText {
		n, err := strconv.ParseFloat(v.Text, 64)
		return n, err == nil
	}
	return 0, false
}

// ParseAddress parses "A1" into (row, col) 0-indexed.
func ParseAddress(addr string) (int, int, bool) {
	if len(addr) < 2 {
		return 0, 0, false
	}
	letter := addr[0]
	if letter < 'A' || letter > 'Z' {
		letter &^= 0x20  // uppercase
		if letter < 'A' || letter > 'Z' {
			return 0, 0, false
		}
	}
	col := int(letter - 'A')
	row, err := strconv.Atoi(addr[1:])
	if err != nil || row < 1 {
		return 0, 0, false
	}
	return row - 1, col, true
}

// FormatAddress returns "A1" style.
func FormatAddress(row, col int) string {
	return fmt.Sprintf("%c%d", 'A'+col, row+1)
}

// XLSheet is a sparse spreadsheet.
type XLSheet struct {
	Cells map[[2]int]XLCell
}

// NewXLSheet returns empty sheet.
func NewXLSheet() *XLSheet { return &XLSheet{Cells: map[[2]int]XLCell{}} }

// SetNumber sets a numeric cell.
func (s *XLSheet) SetNumber(addr string, n float64) error {
	row, col, ok := ParseAddress(addr)
	if !ok {
		return fmt.Errorf("bad address: %s", addr)
	}
	s.Cells[[2]int{row, col}] = XLCell{Kind: XLNumber, Number: n}
	return nil
}

// SetText sets a text cell.
func (s *XLSheet) SetText(addr, text string) error {
	row, col, ok := ParseAddress(addr)
	if !ok {
		return fmt.Errorf("bad address: %s", addr)
	}
	s.Cells[[2]int{row, col}] = XLCell{Kind: XLText, Text: text}
	return nil
}

// SetFormula sets a formula cell.
func (s *XLSheet) SetFormula(addr, formula string) error {
	row, col, ok := ParseAddress(addr)
	if !ok {
		return fmt.Errorf("bad address: %s", addr)
	}
	s.Cells[[2]int{row, col}] = XLCell{Kind: XLFormula, Formula: formula}
	return nil
}

// Evaluate returns the computed value at an address.
func (s *XLSheet) Evaluate(addr string) XLValue {
	row, col, ok := ParseAddress(addr)
	if !ok {
		return XLValue{Kind: XLValError, Text: "bad address: " + addr}
	}
	return s.evalCell([2]int{row, col}, map[[2]int]bool{})
}

func (s *XLSheet) evalCell(pos [2]int, visiting map[[2]int]bool) XLValue {
	if visiting[pos] {
		return XLValue{Kind: XLValError, Text: "circular reference"}
	}
	visiting[pos] = true
	defer delete(visiting, pos)

	cell, ok := s.Cells[pos]
	if !ok || cell.Kind == XLBlank {
		return XLValue{Kind: XLValEmpty}
	}
	switch cell.Kind {
	case XLNumber:
		return XLValue{Kind: XLValNumber, Number: cell.Number}
	case XLText:
		return XLValue{Kind: XLValText, Text: cell.Text}
	case XLFormula:
		return s.evalFormula(cell.Formula, visiting)
	}
	return XLValue{Kind: XLValEmpty}
}

func (s *XLSheet) evalFormula(f string, visiting map[[2]int]bool) XLValue {
	body := strings.TrimLeft(f, "=")
	body = strings.TrimSpace(body)

	// Function?
	if paren := strings.Index(body, "("); paren > 0 && strings.HasSuffix(body, ")") {
		name := body[:paren]
		args := body[paren+1 : len(body)-1]
		return s.evalFunction(name, args, visiting)
	}
	// Binary op
	for _, op := range "+-*/" {
		if pos := strings.IndexRune(body, op); pos > 0 {
			l := s.evalOperand(strings.TrimSpace(body[:pos]), visiting)
			r := s.evalOperand(strings.TrimSpace(body[pos+1:]), visiting)
			return applyOp(op, l, r)
		}
	}
	return s.evalOperand(body, visiting)
}

func (s *XLSheet) evalFunction(name, args string, visiting map[[2]int]bool) XLValue {
	values := s.expandRange(args, visiting)
	nums := []float64{}
	for _, v := range values {
		if n, ok := v.AsNumber(); ok {
			nums = append(nums, n)
		}
	}
	upper := strings.ToUpper(name)
	switch upper {
	case "SUM":
		sum := 0.0
		for _, n := range nums {
			sum += n
		}
		return XLValue{Kind: XLValNumber, Number: sum}
	case "AVG", "AVERAGE":
		if len(nums) == 0 {
			return XLValue{Kind: XLValError, Text: "empty range for AVG"}
		}
		sum := 0.0
		for _, n := range nums {
			sum += n
		}
		return XLValue{Kind: XLValNumber, Number: sum / float64(len(nums))}
	case "MIN":
		if len(nums) == 0 {
			return XLValue{Kind: XLValError, Text: "empty range"}
		}
		m := nums[0]
		for _, n := range nums[1:] {
			if n < m {
				m = n
			}
		}
		return XLValue{Kind: XLValNumber, Number: m}
	case "MAX":
		if len(nums) == 0 {
			return XLValue{Kind: XLValError, Text: "empty range"}
		}
		m := nums[0]
		for _, n := range nums[1:] {
			if n > m {
				m = n
			}
		}
		return XLValue{Kind: XLValNumber, Number: m}
	case "COUNT":
		return XLValue{Kind: XLValNumber, Number: float64(len(nums))}
	}
	return XLValue{Kind: XLValError, Text: "unknown function: " + name}
}

func (s *XLSheet) expandRange(args string, visiting map[[2]int]bool) []XLValue {
	var out []XLValue
	for _, part := range strings.Split(args, ",") {
		part = strings.TrimSpace(part)
		if colon := strings.Index(part, ":"); colon >= 0 {
			r1, c1, ok1 := ParseAddress(part[:colon])
			r2, c2, ok2 := ParseAddress(part[colon+1:])
			if ok1 && ok2 {
				for r := r1; r <= r2; r++ {
					for c := c1; c <= c2; c++ {
						out = append(out, s.evalCell([2]int{r, c}, visiting))
					}
				}
				continue
			}
		}
		out = append(out, s.evalOperand(part, visiting))
	}
	return out
}

func (s *XLSheet) evalOperand(str string, visiting map[[2]int]bool) XLValue {
	str = strings.TrimSpace(str)
	if n, err := strconv.ParseFloat(str, 64); err == nil {
		return XLValue{Kind: XLValNumber, Number: n}
	}
	if r, c, ok := ParseAddress(str); ok {
		return s.evalCell([2]int{r, c}, visiting)
	}
	return XLValue{Kind: XLValError, Text: "bad operand: " + str}
}

func applyOp(op rune, l, r XLValue) XLValue {
	la, okL := l.AsNumber()
	rb, okR := r.AsNumber()
	if !okL || !okR {
		return XLValue{Kind: XLValError, Text: "non-numeric operand"}
	}
	switch op {
	case '+': return XLValue{Kind: XLValNumber, Number: la + rb}
	case '-': return XLValue{Kind: XLValNumber, Number: la - rb}
	case '*': return XLValue{Kind: XLValNumber, Number: la * rb}
	case '/':
		if rb == 0 {
			return XLValue{Kind: XLValError, Text: "div by zero"}
		}
		return XLValue{Kind: XLValNumber, Number: la / rb}
	}
	return XLValue{Kind: XLValError, Text: "bad op"}
}
