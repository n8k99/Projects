-- Sort File Records Utility — parse delimited records, multi-key sort.

namespace Files.SFR

inductive FieldVal where
  | int (n : Int)
  | str (s : String)
  deriving Repr, BEq

def FieldVal.asStr : FieldVal → String
  | .int n => toString n
  | .str s => s

def parseField (s : String) : FieldVal :=
  let t := s.trim
  match t.toInt? with
  | some n => .int n
  | none => .str t

structure Record where
  fields : List FieldVal
  deriving Repr

def Record.parse (line : String) : Record :=
  { fields := (line.splitOn ",").map parseField }

def Record.serialise (r : Record) : String :=
  ",".intercalate (r.fields.map FieldVal.asStr)

inductive Order where | asc | desc
  deriving Repr, BEq

inductive Kind where | str | int
  deriving Repr, BEq

structure SortKey where
  field : Nat
  order : Order := .asc
  kind : Kind := .str
  deriving Repr

structure Table where
  header : Option (List String) := none
  rows : List Record := []
  deriving Repr

def Table.parse (text : String) (hasHeader : Bool) : Table :=
  let lines := (text.splitOn "\n").filter (fun l => l != "")
  if hasHeader then
    match lines with
    | [] => { header := none, rows := [] }
    | h :: rest =>
      let cols := (h.splitOn ",").map String.trim
      { header := some cols, rows := rest.map Record.parse }
  else
    { header := none, rows := lines.map Record.parse }

def Record.fieldAt (r : Record) (i : Nat) : Option FieldVal :=
  r.fields.get? i

def compareField (a b : Option FieldVal) (kind : Kind) : Int :=
  match a, b with
  | none, none => 0
  | none, _    => -1
  | _,    none => 1
  | some x, some y =>
    match kind with
    | .int =>
      let xi := match x with | .int n => n | .str s => s.toInt?.getD 0
      let yi := match y with | .int n => n | .str s => s.toInt?.getD 0
      if xi < yi then -1 else if xi > yi then 1 else 0
    | .str =>
      let xs := x.asStr
      let ys := y.asStr
      if xs < ys then -1 else if xs > ys then 1 else 0

def compareRecords (a b : Record) (spec : List SortKey) : Int :=
  spec.foldl (init := 0) fun acc k =>
    if acc ≠ 0 then acc
    else
      let c := compareField (a.fieldAt k.field) (b.fieldAt k.field) k.kind
      if k.order == .desc then -c else c

def Table.sortBy (t : Table) (spec : List SortKey) : Table :=
  { t with rows := t.rows.mergeSort fun a b => compareRecords a b spec < 0 }

end Files.SFR
