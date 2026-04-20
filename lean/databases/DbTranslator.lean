-- Database Translation — dialect-aware DDL emission.

namespace Databases.DT

inductive Dialect where | mysql | postgres | sqlite
  deriving Repr, BEq

inductive ColumnKind where
  | smallInt | integer | bigInt | text | varchar | boolean
  | timestamp | date | float | decimal
  deriving Repr, BEq

structure ColumnType where
  kind : ColumnKind
  length : Nat := 0
  precision : Nat := 0
  scale : Nat := 0
  deriving Repr

def ColumnType.render (c : ColumnType) (d : Dialect) : String :=
  match c.kind with
  | .smallInt =>
    match d with
    | .mysql => "TINYINT" | .postgres => "SMALLINT" | .sqlite => "INTEGER"
  | .integer => "INTEGER"
  | .bigInt => if d == .sqlite then "INTEGER" else "BIGINT"
  | .text => "TEXT"
  | .varchar => s!"VARCHAR({c.length})"
  | .boolean =>
    match d with
    | .mysql => "TINYINT(1)" | .postgres => "BOOLEAN" | .sqlite => "INTEGER"
  | .timestamp =>
    match d with
    | .mysql => "DATETIME" | .postgres => "TIMESTAMP" | .sqlite => "TEXT"
  | .date => if d == .sqlite then "TEXT" else "DATE"
  | .float => "REAL"
  | .decimal => s!"DECIMAL({c.precision},{c.scale})"

inductive DefaultKind where | literal | now
  deriving Repr, BEq

structure DefaultValue where
  kind : DefaultKind
  literal : String := ""
  deriving Repr

def DefaultValue.render (v : DefaultValue) (d : Dialect) : String :=
  match v.kind with
  | .literal => v.literal
  | .now => if d == .mysql then "NOW()" else "CURRENT_TIMESTAMP"

structure DbColumn where
  name : String
  ty : ColumnType
  nullable : Bool := true
  primaryKey : Bool := false
  unique : Bool := false
  autoIncrement : Bool := false
  default : Option DefaultValue := none
  deriving Repr

structure Table where
  name : String
  columns : List DbColumn := []
  deriving Repr

def autoIncrementSyntax (d : Dialect) : String :=
  match d with
  | .mysql => "AUTO_INCREMENT"
  | .postgres => "GENERATED ALWAYS AS IDENTITY"
  | .sqlite => "AUTOINCREMENT"

def renderColumn (col : DbColumn) (d : Dialect) : String :=
  let mut parts := [col.name, col.ty.render d]
  if col.primaryKey then parts := parts ++ ["PRIMARY KEY"]
  if !col.nullable ∧ !col.primaryKey then parts := parts ++ ["NOT NULL"]
  if col.unique ∧ !col.primaryKey then parts := parts ++ ["UNIQUE"]
  if col.autoIncrement then
    if d == .sqlite ∧ col.primaryKey then
      parts := parts ++ ["AUTOINCREMENT"]
    else if d ≠ .sqlite then
      parts := parts ++ [autoIncrementSyntax d]
  match col.default with
  | some dv => parts := parts ++ [s!"DEFAULT {dv.render d}"]
  | none => pure ()
  " ".intercalate parts

def emitCreateTable (t : Table) (d : Dialect) : String :=
  let header := s!"CREATE TABLE {t.name} (\n"
  let cols := t.columns.map fun c => "  " ++ renderColumn c d
  let body := ",\n".intercalate cols
  header ++ body ++ "\n);\n"

def emitSchema (tables : List Table) (d : Dialect) : String :=
  tables.foldl (init := "") fun acc t => acc ++ emitCreateTable t d

end Databases.DT
