// SQL Query Analyzer — recursive-descent parser + static analysis.
package databases

import (
	"fmt"
	"sort"
	"strconv"
	"strings"
	"unicode"
)

// TokKind tags a token.
type TokKind int

const (
	TkWord     TokKind = 0
	TkNumber   TokKind = 1
	TkString   TokKind = 2
	TkOp       TokKind = 3
	TkComma    TokKind = 4
	TkStar     TokKind = 5
	TkSemi     TokKind = 6
)

// Token is one lexeme.
type Token struct {
	Kind   TokKind
	Word   string
	Number int64
	String string
	Op     string
}

// Tokenise produces a token stream.
func Tokenise(s string) ([]Token, error) {
	var out []Token
	runes := []rune(s)
	i, n := 0, len(runes)
	for i < n {
		c := runes[i]
		switch {
		case unicode.IsSpace(c):
			i++
		case c == ',':
			out = append(out, Token{Kind: TkComma}); i++
		case c == ';':
			out = append(out, Token{Kind: TkSemi}); i++
		case c == '*':
			out = append(out, Token{Kind: TkStar}); i++
		case c == '\'':
			i++
			start := i
			for i < n && runes[i] != '\'' { i++ }
			if i >= n { return nil, fmt.Errorf("unterminated string") }
			out = append(out, Token{Kind: TkString, String: string(runes[start:i])})
			i++
		case c == '=':
			out = append(out, Token{Kind: TkOp, Op: "="}); i++
		case c == '<' || c == '>' || c == '!':
			op := string(c)
			i++
			if i < n && runes[i] == '=' { op += "="; i++ }
			if op == "!" { return nil, fmt.Errorf("bad operator: !") }
			out = append(out, Token{Kind: TkOp, Op: op})
		case unicode.IsDigit(c) || (c == '-' && i+1 < n && unicode.IsDigit(runes[i+1])):
			start := i
			if c == '-' { i++ }
			for i < n && unicode.IsDigit(runes[i]) { i++ }
			num, _ := strconv.ParseInt(string(runes[start:i]), 10, 64)
			out = append(out, Token{Kind: TkNumber, Number: num})
		case unicode.IsLetter(c) || c == '_':
			start := i
			for i < n && (unicode.IsLetter(runes[i]) || unicode.IsDigit(runes[i]) || runes[i] == '_') {
				i++
			}
			out = append(out, Token{Kind: TkWord, Word: strings.ToLower(string(runes[start:i]))})
		default:
			return nil, fmt.Errorf("unexpected char: %c", c)
		}
	}
	return out, nil
}

// SqlValue is a typed value.
type SqlValue struct {
	IsNum bool
	Num   int64
	Text  string
}

// WhereClause is column OP value.
type WhereClause struct {
	Column string
	Op     string
	Value  SqlValue
}

// SqlOrder is ascending or descending.
type SqlOrder int

const (
	OrderAsc  SqlOrder = 0
	OrderDesc SqlOrder = 1
)

// OrderBy holds an ORDER BY clause.
type OrderBy struct {
	Column string
	Order  SqlOrder
}

// SqlQuery is a parsed SELECT.
type SqlQuery struct {
	ProjectionAll     bool
	ProjectionColumns []string
	Table             string
	Where             *WhereClause
	OrderBy           *OrderBy
	Limit             *int32
}

type sqlParser struct {
	tokens []Token
	pos    int
}

func (p *sqlParser) peek() *Token {
	if p.pos >= len(p.tokens) { return nil }
	return &p.tokens[p.pos]
}

func (p *sqlParser) bump() *Token {
	t := p.peek()
	p.pos++
	return t
}

func (p *sqlParser) expectWord(w string) error {
	t := p.bump()
	if t == nil || t.Kind != TkWord || t.Word != w {
		return fmt.Errorf("expected '%s'", w)
	}
	return nil
}

func (p *sqlParser) peekWord() string {
	t := p.peek()
	if t != nil && t.Kind == TkWord { return t.Word }
	return ""
}

func (p *sqlParser) parseIdent() (string, error) {
	t := p.bump()
	if t == nil || t.Kind != TkWord {
		return "", fmt.Errorf("expected identifier")
	}
	return t.Word, nil
}

func (p *sqlParser) parseProjection() (bool, []string, error) {
	t := p.peek()
	if t != nil && t.Kind == TkStar {
		p.bump()
		return true, nil, nil
	}
	first, err := p.parseIdent()
	if err != nil { return false, nil, err }
	cols := []string{first}
	for {
		t := p.peek()
		if t == nil || t.Kind != TkComma { break }
		p.bump()
		c, err := p.parseIdent()
		if err != nil { return false, nil, err }
		cols = append(cols, c)
	}
	return false, cols, nil
}

func (p *sqlParser) parseWhere() (*WhereClause, error) {
	col, err := p.parseIdent()
	if err != nil { return nil, err }
	op := p.bump()
	if op == nil || op.Kind != TkOp {
		return nil, fmt.Errorf("expected operator")
	}
	v := p.bump()
	if v == nil {
		return nil, fmt.Errorf("expected value")
	}
	var val SqlValue
	switch v.Kind {
	case TkNumber:
		val = SqlValue{IsNum: true, Num: v.Number}
	case TkString:
		val = SqlValue{IsNum: false, Text: v.String}
	default:
		return nil, fmt.Errorf("expected value")
	}
	return &WhereClause{Column: col, Op: op.Op, Value: val}, nil
}

func (p *sqlParser) parseOrderBy() (*OrderBy, error) {
	col, err := p.parseIdent()
	if err != nil { return nil, err }
	order := OrderAsc
	switch p.peekWord() {
	case "asc": p.bump()
	case "desc": p.bump(); order = OrderDesc
	}
	return &OrderBy{Column: col, Order: order}, nil
}

func (p *sqlParser) parseQuery() (*SqlQuery, error) {
	if err := p.expectWord("select"); err != nil { return nil, err }
	all, cols, err := p.parseProjection()
	if err != nil { return nil, err }
	if err := p.expectWord("from"); err != nil { return nil, err }
	table, err := p.parseIdent()
	if err != nil { return nil, err }
	q := &SqlQuery{ProjectionAll: all, ProjectionColumns: cols, Table: table}
	if p.peekWord() == "where" {
		p.bump()
		w, err := p.parseWhere()
		if err != nil { return nil, err }
		q.Where = w
	}
	if p.peekWord() == "order" {
		p.bump()
		if err := p.expectWord("by"); err != nil { return nil, err }
		ob, err := p.parseOrderBy()
		if err != nil { return nil, err }
		q.OrderBy = ob
	}
	if p.peekWord() == "limit" {
		p.bump()
		t := p.bump()
		if t == nil || t.Kind != TkNumber || t.Number < 0 {
			return nil, fmt.Errorf("expected positive number after LIMIT")
		}
		n := int32(t.Number)
		q.Limit = &n
	}
	return q, nil
}

// ParseSql is the entry point.
func ParseSql(s string) (*SqlQuery, error) {
	tokens, err := Tokenise(s)
	if err != nil { return nil, err }
	if len(tokens) > 0 && tokens[len(tokens)-1].Kind == TkSemi {
		tokens = tokens[:len(tokens)-1]
	}
	p := &sqlParser{tokens: tokens}
	q, err := p.parseQuery()
	if err != nil { return nil, err }
	if p.pos < len(p.tokens) {
		return nil, fmt.Errorf("trailing tokens after query")
	}
	return q, nil
}

// Severity of a finding.
type Severity int

const (
	SevInfo    Severity = 0
	SevWarning Severity = 1
	SevError   Severity = 2
)

// Finding is one analysis result.
type Finding struct {
	Severity Severity
	Code     string
	Message  string
}

// Analysis aggregates findings + references.
type Analysis struct {
	Tables   []string
	Columns  []string
	Findings []Finding
}

// SqlSchema is a known database shape.
type SqlSchema struct {
	Tables map[string]map[string]bool
}

// NewSqlSchema creates an empty schema.
func NewSqlSchema() *SqlSchema {
	return &SqlSchema{Tables: map[string]map[string]bool{}}
}

// AddTable registers columns for a table.
func (s *SqlSchema) AddTable(name string, columns []string) {
	cols := map[string]bool{}
	for _, c := range columns { cols[c] = true }
	s.Tables[name] = cols
}

// AnalyseSql runs static checks over a query.
func AnalyseSql(q *SqlQuery, schema *SqlSchema) Analysis {
	tables := []string{q.Table}
	var columns []string
	var findings []Finding

	known, tableKnown := schema.Tables[q.Table]
	if !tableKnown {
		findings = append(findings, Finding{
			Severity: SevError, Code: "UNKNOWN_TABLE",
			Message: "unknown table: " + q.Table,
		})
	}

	seen := map[string]bool{}
	addCol := func(c string) {
		if !seen[c] {
			columns = append(columns, c)
			seen[c] = true
		}
	}
	if !q.ProjectionAll {
		for _, c := range q.ProjectionColumns { addCol(c) }
	}
	if q.Where != nil { addCol(q.Where.Column) }
	if q.OrderBy != nil { addCol(q.OrderBy.Column) }

	if tableKnown {
		for _, c := range columns {
			if !known[c] {
				findings = append(findings, Finding{
					Severity: SevError, Code: "UNKNOWN_COLUMN",
					Message: "unknown column " + c + " in table " + q.Table,
				})
			}
		}
	}
	if q.ProjectionAll {
		findings = append(findings, Finding{
			Severity: SevWarning, Code: "SELECT_STAR",
			Message: "SELECT * pulls every column; name them explicitly",
		})
	}
	if q.Limit != nil && q.OrderBy == nil {
		findings = append(findings, Finding{
			Severity: SevWarning, Code: "LIMIT_WITHOUT_ORDER",
			Message: "LIMIT without ORDER BY returns arbitrary rows",
		})
	}
	if q.Where == nil && q.Limit == nil {
		findings = append(findings, Finding{
			Severity: SevInfo, Code: "FULL_SCAN",
			Message: "no WHERE or LIMIT — query will scan every row",
		})
	}

	sort.Strings(columns)
	sort.Strings(tables)
	return Analysis{Tables: tables, Columns: columns, Findings: findings}
}
