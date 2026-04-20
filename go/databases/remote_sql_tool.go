// Remote SQL Tool — in-memory table engine + execution over parsed queries.
package databases

import (
	"fmt"
	"sort"
)

// ColType classifies a column's type.
type ColType int

const (
	CTInt  ColType = 0
	CTText ColType = 1
)

// Cell is a typed nullable value.
type Cell struct {
	IsNull bool
	IsInt  bool
	Int    int64
	Text   string
}

// CellInt factory.
func CellInt(n int64) Cell { return Cell{IsInt: true, Int: n} }

// CellText factory.
func CellText(s string) Cell { return Cell{Text: s} }

// CellNull factory.
func CellNull() Cell { return Cell{IsNull: true} }

// TypeOf returns the cell's column type, or -1 for NULL.
func (c Cell) TypeOf() ColType {
	if c.IsNull { return -1 }
	if c.IsInt { return CTInt }
	return CTText
}

// DbColumn is a named typed column.
type DbColumn struct {
	Name string
	Ty   ColType
}

// DbTable holds column schema + rows.
type DbTable struct {
	Name    string
	Columns []DbColumn
	Rows    [][]Cell
}

// ColumnIndex finds the position of a named column, or -1.
func (t *DbTable) ColumnIndex(name string) int {
	for i, c := range t.Columns {
		if c.Name == name { return i }
	}
	return -1
}

// DbConnection is a mutable set of tables with optional pending-transaction buffer.
type DbConnection struct {
	Tables  map[string]*DbTable
	pending map[string][][]Cell  // nil = no active txn
}

// NewDbConnection returns an empty connection.
func NewDbConnection() *DbConnection {
	return &DbConnection{Tables: map[string]*DbTable{}}
}

// InTransaction reports whether a txn is open.
func (c *DbConnection) InTransaction() bool { return c.pending != nil }

// CreateDbTable adds a new table.
func (c *DbConnection) CreateDbTable(name string, cols []DbColumn) error {
	if _, exists := c.Tables[name]; exists {
		return fmt.Errorf("table exists: %s", name)
	}
	c.Tables[name] = &DbTable{Name: name, Columns: cols}
	return nil
}

// InsertRow validates and appends a row. If a txn is open, the row is buffered.
func (c *DbConnection) InsertRow(table string, values []Cell) error {
	t, ok := c.Tables[table]
	if !ok { return fmt.Errorf("unknown table: %s", table) }
	if len(values) != len(t.Columns) {
		return fmt.Errorf("expected %d values, got %d", len(t.Columns), len(values))
	}
	for i, col := range t.Columns {
		vt := values[i].TypeOf()
		if vt != -1 && vt != col.Ty {
			return fmt.Errorf("type mismatch on column %s", col.Name)
		}
	}
	if c.pending != nil {
		c.pending[table] = append(c.pending[table], values)
	} else {
		t.Rows = append(t.Rows, values)
	}
	return nil
}

// Begin opens a transaction.
func (c *DbConnection) Begin() {
	c.pending = map[string][][]Cell{}
}

// Commit persists pending rows.
func (c *DbConnection) Commit() error {
	if c.pending == nil { return fmt.Errorf("no transaction") }
	for name, rows := range c.pending {
		if t, ok := c.Tables[name]; ok {
			t.Rows = append(t.Rows, rows...)
		}
	}
	c.pending = nil
	return nil
}

// Rollback discards pending rows.
func (c *DbConnection) Rollback() error {
	if c.pending == nil { return fmt.Errorf("no transaction") }
	c.pending = nil
	return nil
}

func (c *DbConnection) rowsOf(table string) [][]Cell {
	var out [][]Cell
	if t, ok := c.Tables[table]; ok {
		for _, r := range t.Rows {
			cp := append([]Cell{}, r...)
			out = append(out, cp)
		}
	}
	if c.pending != nil {
		if p, ok := c.pending[table]; ok {
			for _, r := range p {
				cp := append([]Cell{}, r...)
				out = append(out, cp)
			}
		}
	}
	return out
}

// WhereExpression is a simple column-op-value predicate.
type WhereExpression struct {
	Column string
	Op     string
	Value  Cell
}

// OrderByExpression sorts on a column.
type OrderByExpression struct {
	Column string
	Desc   bool
}

// DbSelectQuery parameterises an execution.
type DbSelectQuery struct {
	ProjectAll bool
	Columns    []string
	Table      string
	Where      *WhereExpression
	OrderBy    *OrderByExpression
	Limit      *int32
}

// DbResultSet is the output of a select.
type DbResultSet struct {
	Columns []string
	Rows    [][]Cell
}

// Select executes the query.
func (c *DbConnection) Select(q *DbSelectQuery) (*DbResultSet, error) {
	t, ok := c.Tables[q.Table]
	if !ok { return nil, fmt.Errorf("unknown table: %s", q.Table) }

	var proj []int
	if q.ProjectAll {
		proj = make([]int, len(t.Columns))
		for i := range t.Columns { proj[i] = i }
	} else {
		proj = make([]int, 0, len(q.Columns))
		for _, c := range q.Columns {
			i := t.ColumnIndex(c)
			if i < 0 { return nil, fmt.Errorf("unknown column: %s", c) }
			proj = append(proj, i)
		}
	}

	whereIdx := -1
	if q.Where != nil {
		whereIdx = t.ColumnIndex(q.Where.Column)
		if whereIdx < 0 { return nil, fmt.Errorf("unknown column: %s", q.Where.Column) }
	}

	orderIdx := -1
	orderDesc := false
	if q.OrderBy != nil {
		orderIdx = t.ColumnIndex(q.OrderBy.Column)
		if orderIdx < 0 { return nil, fmt.Errorf("unknown column: %s", q.OrderBy.Column) }
		orderDesc = q.OrderBy.Desc
	}

	baseRows := c.rowsOf(q.Table)
	filtered := make([][]Cell, 0, len(baseRows))
	for _, r := range baseRows {
		if q.Where == nil || evalPred(r[whereIdx], q.Where.Op, q.Where.Value) {
			filtered = append(filtered, r)
		}
	}

	if orderIdx >= 0 {
		sort.SliceStable(filtered, func(i, j int) bool {
			cmp := compareCells(filtered[i][orderIdx], filtered[j][orderIdx])
			if orderDesc { return cmp > 0 }
			return cmp < 0
		})
	}

	projected := make([][]Cell, 0, len(filtered))
	for _, r := range filtered {
		row := make([]Cell, len(proj))
		for i, idx := range proj { row[i] = r[idx] }
		projected = append(projected, row)
	}
	if q.Limit != nil && int(*q.Limit) < len(projected) {
		projected = projected[:*q.Limit]
	}
	cols := make([]string, len(proj))
	for i, idx := range proj { cols[i] = t.Columns[idx].Name }
	return &DbResultSet{Columns: cols, Rows: projected}, nil
}

func evalPred(cell Cell, op string, rhs Cell) bool {
	c := compareCells(cell, rhs)
	switch op {
	case "=": return c == 0
	case "!=": return c != 0
	case "<": return c < 0
	case "<=": return c <= 0
	case ">": return c > 0
	case ">=": return c >= 0
	}
	return false
}

func compareCells(a, b Cell) int {
	if a.IsNull && b.IsNull { return 0 }
	if a.IsNull { return -1 }
	if b.IsNull { return 1 }
	if a.IsInt && b.IsInt {
		switch {
		case a.Int < b.Int: return -1
		case a.Int > b.Int: return 1
		default: return 0
		}
	}
	if !a.IsInt && !b.IsInt {
		switch {
		case a.Text < b.Text: return -1
		case a.Text > b.Text: return 1
		default: return 0
		}
	}
	return 0
}
