-- Report Generator — grouped reports with subtotal and grand-total rows.

namespace Databases.RG

inductive Value where
  | int (n : Int)
  | text (s : String)
  | null
  deriving Repr

def Value.asInt : Value → Option Int
  | .int n => some n
  | _ => none

def Value.asText : Value → String
  | .text s => s
  | .int n => toString n
  | .null => ""

inductive ColumnKind where | plain | sumAggregate
  deriving Repr, BEq

structure ColumnDef where
  name : String
  field : String
  kind : ColumnKind := .plain
  formatCents : Bool := false
  deriving Repr

inductive RowKind where | header | data | subtotal | total
  deriving Repr, BEq

structure ReportRow where
  kind : RowKind
  cells : List String
  groupLabel : Option String := none
  deriving Repr

structure ReportSpec where
  title : String
  columns : List ColumnDef
  groupBy : Option String := none
  deriving Repr

structure Report where
  title : String
  rows : List ReportRow
  deriving Repr

def formatInt (n : Int) (asCents : Bool) : String :=
  if !asCents then toString n
  else
    let sign := if n < 0 then "-" else ""
    let abs := if n < 0 then -n else n
    let dollars := abs / 100
    let cents := abs % 100
    let centsStr :=
      if cents < 10 then "0" ++ toString cents else toString cents
    s!"${sign}{dollars}.{centsStr}"

def formatCell (v : Option Value) (col : ColumnDef) : String :=
  match v with
  | none | some .null => ""
  | some (.int n) => formatInt n col.formatCents
  | some (.text s) => s

abbrev Record := List (String × Value)

def recordGet (r : Record) (field : String) : Option Value :=
  (r.find? (fun p => p.1 == field)).map Prod.snd

def generate (records : List Record) (spec : ReportSpec) : Report := Id.run do
  let mut rows : List ReportRow := []
  let header : ReportRow :=
    { kind := .header, cells := spec.columns.map ColumnDef.name }
  rows := rows ++ [header]

  let groups : List (String × List Record) :=
    match spec.groupBy with
    | some gf =>
      let keyed := records.map fun r =>
        let key := match recordGet r gf with
          | some v => let t := v.asText; if t == "" then "(none)" else t
          | none => "(none)"
        (key, r)
      let keys := (keyed.map Prod.fst).eraseDups.mergeSort (· < ·)
      keys.map fun k => (k, (keyed.filter (fun p => p.1 == k)).map Prod.snd)
    | none => [("__all__", records)]

  let mut grandTotals : List (String × Int) := []

  for (groupKey, groupRecords) in groups do
    for record in groupRecords do
      let cells := spec.columns.map fun c => formatCell (recordGet record c.field) c
      let label := match spec.groupBy with | some _ => some groupKey | none => none
      rows := rows ++ [{ kind := .data, cells, groupLabel := label }]
    if spec.groupBy.isSome then
      let mut subtotals : List (String × Int) := []
      for record in groupRecords do
        for c in spec.columns do
          if c.kind == .sumAggregate then
            match recordGet record c.field with
            | some v =>
              match v.asInt with
              | some n =>
                let newSub := (subtotals.find? (fun p => p.1 == c.field)).map Prod.snd |>.getD 0
                subtotals := (subtotals.filter (fun p => p.1 ≠ c.field)) ++ [(c.field, newSub + n)]
                let newGT := (grandTotals.find? (fun p => p.1 == c.field)).map Prod.snd |>.getD 0
                grandTotals := (grandTotals.filter (fun p => p.1 ≠ c.field)) ++ [(c.field, newGT + n)]
              | none => pure ()
            | none => pure ()
      let cells := spec.columns.mapIdx fun i c =>
        if i == 0 then s!"{groupKey} subtotal"
        else if c.kind == .sumAggregate then
          let sub := (subtotals.find? (fun p => p.1 == c.field)).map Prod.snd |>.getD 0
          formatInt sub c.formatCents
        else ""
      rows := rows ++ [{ kind := .subtotal, cells, groupLabel := some groupKey }]

  let hasAggregate := spec.columns.any (fun c => c.kind == .sumAggregate)
  if hasAggregate then
    if spec.groupBy.isNone then
      for record in records do
        for c in spec.columns do
          if c.kind == .sumAggregate then
            match recordGet record c.field with
            | some v =>
              match v.asInt with
              | some n =>
                let newGT := (grandTotals.find? (fun p => p.1 == c.field)).map Prod.snd |>.getD 0
                grandTotals := (grandTotals.filter (fun p => p.1 ≠ c.field)) ++ [(c.field, newGT + n)]
              | none => pure ()
            | none => pure ()
    let cells := spec.columns.mapIdx fun i c =>
      if i == 0 then "TOTAL"
      else if c.kind == .sumAggregate then
        let gt := (grandTotals.find? (fun p => p.1 == c.field)).map Prod.snd |>.getD 0
        formatInt gt c.formatCents
      else ""
    rows := rows ++ [{ kind := .total, cells }]

  return { title := spec.title, rows }

def render (r : Report) : String := Id.run do
  let mut out := ""
  if r.title.length > 0 then out := out ++ r.title ++ "\n"
  for row in r.rows do
    let prefix := match row.kind with
      | .header => "#" | .data => " "
      | .subtotal => "~" | .total => "="
    out := out ++ prefix ++ "\t" ++ "\t".intercalate row.cells ++ "\n"
  return out

end Databases.RG
