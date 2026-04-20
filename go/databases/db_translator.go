// Database Translation — dialect-aware DDL emission.
package databases

import (
	"fmt"
	"strings"
)

// DTDialect selects the target SQL dialect.
type DTDialect int

const (
	DTMySQL    DTDialect = 0
	DTPostgres DTDialect = 1
	DTSQLite   DTDialect = 2
)

// DTColumnKind tags the abstract column type.
type DTColumnKind int

const (
	DTKSmallInt  DTColumnKind = 0
	DTKInteger   DTColumnKind = 1
	DTKBigInt    DTColumnKind = 2
	DTKText      DTColumnKind = 3
	DTKVarchar   DTColumnKind = 4
	DTKBoolean   DTColumnKind = 5
	DTKTimestamp DTColumnKind = 6
	DTKDate      DTColumnKind = 7
	DTKFloat     DTColumnKind = 8
	DTKDecimal   DTColumnKind = 9
)

// DTColumnType is the abstract (dialect-free) column type.
type DTColumnType struct {
	Kind      DTColumnKind
	Length    int   // VARCHAR length
	Precision int   // DECIMAL precision
	Scale     int   // DECIMAL scale
}

// Render emits the dialect-specific type name.
func (c DTColumnType) Render(d DTDialect) string {
	switch c.Kind {
	case DTKSmallInt:
		switch d {
		case DTMySQL:    return "TINYINT"
		case DTPostgres: return "SMALLINT"
		default:         return "INTEGER"
		}
	case DTKInteger:
		return "INTEGER"
	case DTKBigInt:
		if d == DTSQLite { return "INTEGER" }
		return "BIGINT"
	case DTKText:
		return "TEXT"
	case DTKVarchar:
		return fmt.Sprintf("VARCHAR(%d)", c.Length)
	case DTKBoolean:
		switch d {
		case DTMySQL:    return "TINYINT(1)"
		case DTPostgres: return "BOOLEAN"
		default:         return "INTEGER"
		}
	case DTKTimestamp:
		switch d {
		case DTMySQL:    return "DATETIME"
		case DTPostgres: return "TIMESTAMP"
		default:         return "TEXT"
		}
	case DTKDate:
		if d == DTSQLite { return "TEXT" }
		return "DATE"
	case DTKFloat:
		return "REAL"
	case DTKDecimal:
		return fmt.Sprintf("DECIMAL(%d,%d)", c.Precision, c.Scale)
	}
	return "UNKNOWN"
}

// DTDefaultKind selects the default-value flavour.
type DTDefaultKind int

const (
	DTDLiteral DTDefaultKind = 0
	DTDNow     DTDefaultKind = 1
)

// DTDefaultValue is a default clause value.
type DTDefaultValue struct {
	Kind    DTDefaultKind
	Literal string
}

// Render emits the dialect-specific default expression.
func (d DTDefaultValue) Render(dialect DTDialect) string {
	if d.Kind == DTDLiteral { return d.Literal }
	if dialect == DTMySQL { return "NOW()" }
	return "CURRENT_TIMESTAMP"
}

// DTDbColumn is one column declaration.
type DTDbColumn struct {
	Name          string
	Ty            DTColumnType
	Nullable      bool
	PrimaryKey    bool
	Unique        bool
	AutoIncrement bool
	Default       *DTDefaultValue
}

// DTTable holds columns for one table.
type DTTable struct {
	Name    string
	Columns []DTDbColumn
}

func autoIncrementSyntax(d DTDialect) string {
	switch d {
	case DTMySQL:    return "AUTO_INCREMENT"
	case DTPostgres: return "GENERATED ALWAYS AS IDENTITY"
	default:         return "AUTOINCREMENT"
	}
}

func renderColumn(col DTDbColumn, d DTDialect) string {
	parts := []string{col.Name, col.Ty.Render(d)}
	if col.PrimaryKey {
		parts = append(parts, "PRIMARY KEY")
	}
	if !col.Nullable && !col.PrimaryKey {
		parts = append(parts, "NOT NULL")
	}
	if col.Unique && !col.PrimaryKey {
		parts = append(parts, "UNIQUE")
	}
	if col.AutoIncrement {
		if d == DTSQLite && col.PrimaryKey {
			parts = append(parts, "AUTOINCREMENT")
		} else if d != DTSQLite {
			parts = append(parts, autoIncrementSyntax(d))
		}
	}
	if col.Default != nil {
		parts = append(parts, "DEFAULT "+col.Default.Render(d))
	}
	return strings.Join(parts, " ")
}

// DTEmitCreateTable emits a CREATE TABLE statement.
func DTEmitCreateTable(t *DTTable, d DTDialect) string {
	var b strings.Builder
	fmt.Fprintf(&b, "CREATE TABLE %s (\n", t.Name)
	rows := make([]string, len(t.Columns))
	for i, col := range t.Columns {
		rows[i] = "  " + renderColumn(col, d)
	}
	b.WriteString(strings.Join(rows, ",\n"))
	b.WriteString("\n);\n")
	return b.String()
}

// DTEmitSchema concatenates CREATE TABLE for all tables.
func DTEmitSchema(tables []*DTTable, d DTDialect) string {
	var b strings.Builder
	for _, t := range tables {
		b.WriteString(DTEmitCreateTable(t, d))
	}
	return b.String()
}
