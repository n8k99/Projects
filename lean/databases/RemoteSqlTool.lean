-- Remote SQL Tool — in-memory table engine + execution over parsed queries.

namespace Databases.RST

inductive ColType where | int | text
  deriving Repr, BEq

inductive Cell where
  | int (n : Int)
  | text (s : String)
  | null
  deriving Repr, BEq

def Cell.typeOf : Cell → Option ColType
  | .int _ => some .int
  | .text _ => some .text
  | .null => none

structure Column where
  name : String
  ty : ColType
  deriving Repr, BEq

structure Table where
  name : String
  columns : List Column
  rows : List (List Cell) := []
  deriving Repr

def Table.columnIndex (t : Table) (name : String) : Option Nat :=
  t.columns.findIdx? (fun c => c.name == name)

structure Connection where
  tables : List (String × Table) := []
  pending : Option (List (String × List (List Cell))) := none
  deriving Repr

def Connection.inTransaction (c : Connection) : Bool := c.pending.isSome

def Connection.getTable (c : Connection) (name : String) : Option Table :=
  (c.tables.find? (fun p => p.1 == name)).map Prod.snd

def Connection.createTable (c : Connection) (name : String) (cols : List Column)
    : Except String Connection :=
  if (c.tables.find? (fun p => p.1 == name)).isSome then
    .error s!"table exists: {name}"
  else
    .ok { c with tables := c.tables ++ [(name, { name, columns := cols })] }

def Connection.insertRow (c : Connection) (table : String) (values : List Cell)
    : Except String Connection := do
  let some t := c.getTable table | throw s!"unknown table: {table}"
  if values.length ≠ t.columns.length then
    throw s!"expected {t.columns.length} values, got {values.length}"
  for (col, val) in t.columns.zip values do
    match val.typeOf with
    | some vt => if vt != col.ty then throw s!"type mismatch on {col.name}"
    | none => pure ()
  match c.pending with
  | some p =>
    let updated := match p.find? (fun q => q.1 == table) with
      | some (_, existing) => p.map fun q => if q.1 == table then (table, existing ++ [values]) else q
      | none => p ++ [(table, [values])]
    return { c with pending := some updated }
  | none =>
    let newTables := c.tables.map fun p =>
      if p.1 == table then (table, { p.2 with rows := p.2.rows ++ [values] })
      else p
    return { c with tables := newTables }

def Connection.begin (c : Connection) : Connection :=
  { c with pending := some [] }

def Connection.commit (c : Connection) : Except String Connection := do
  let some p := c.pending | throw "no transaction"
  let mut tables := c.tables
  for (name, rows) in p do
    tables := tables.map fun t =>
      if t.1 == name then (name, { t.2 with rows := t.2.rows ++ rows }) else t
  return { c with tables, pending := none }

def Connection.rollback (c : Connection) : Except String Connection :=
  if c.pending.isNone then .error "no transaction"
  else .ok { c with pending := none }

def Connection.rowsOf (c : Connection) (table : String) : List (List Cell) :=
  let base := (c.getTable table).map Table.rows |>.getD []
  let pending := match c.pending with
    | some p => (p.find? (fun q => q.1 == table)).map Prod.snd |>.getD []
    | none => []
  base ++ pending

def compareCells : Cell → Cell → Int
  | .null, .null => 0
  | .null, _ => -1
  | _, .null => 1
  | .int a, .int b => if a < b then -1 else if a > b then 1 else 0
  | .text a, .text b => if a < b then -1 else if a > b then 1 else 0
  | _, _ => 0

def evalPred (cell : Cell) (op : String) (rhs : Cell) : Bool :=
  let c := compareCells cell rhs
  match op with
  | "=" => c == 0 | "!=" => c != 0
  | "<" => c < 0 | "<=" => c ≤ 0
  | ">" => c > 0 | ">=" => c ≥ 0
  | _ => false

structure WhereExpr where
  column : String
  op : String
  value : Cell
  deriving Repr

structure OrderByExpr where
  column : String
  desc : Bool := false
  deriving Repr

structure SelectQuery where
  projectAll : Bool
  columns : List String := []
  table : String
  whereClause : Option WhereExpr := none
  orderBy : Option OrderByExpr := none
  limit : Option Nat := none
  deriving Repr

structure ResultSet where
  columns : List String
  rows : List (List Cell)
  deriving Repr

def Connection.selectQuery (c : Connection) (q : SelectQuery)
    : Except String ResultSet := do
  let some t := c.getTable q.table | throw s!"unknown table: {q.table}"
  let projIndices ←
    if q.projectAll then pure (List.range t.columns.length)
    else do
      let mut idxs : List Nat := []
      for col in q.columns do
        match t.columnIndex col with
        | some i => idxs := idxs ++ [i]
        | none => throw s!"unknown column: {col}"
      pure idxs
  let whereIdx ← match q.whereClause with
    | some w => do
      match t.columnIndex w.column with
      | some i => pure (some i)
      | none => throw s!"unknown column: {w.column}"
    | none => pure none
  let orderInfo ← match q.orderBy with
    | some o => do
      match t.columnIndex o.column with
      | some i => pure (some (i, o.desc))
      | none => throw s!"unknown column: {o.column}"
    | none => pure none
  let baseRows := c.rowsOf q.table
  let filtered := baseRows.filter fun r =>
    match q.whereClause, whereIdx with
    | some w, some i => evalPred (r.get! i) w.op w.value
    | _, _ => true
  let sorted := match orderInfo with
    | some (i, desc) =>
      filtered.mergeSort fun a b =>
        let cmp := compareCells (a.get! i) (b.get! i)
        if desc then cmp > 0 else cmp < 0
    | none => filtered
  let projected := sorted.map fun r => projIndices.map fun i => r.get! i
  let limited := match q.limit with
    | some n => projected.take n
    | none => projected
  let cols := projIndices.map fun i => (t.columns.get! i).name
  return { columns := cols, rows := limited }

end Databases.RST
