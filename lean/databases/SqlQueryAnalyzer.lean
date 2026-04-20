-- SQL Query Analyzer — recursive-descent parser + static analysis.

namespace Databases.SQL

inductive TokenKind where | word | number | str | op | comma | star | semi
  deriving Repr, BEq

structure Token where
  kind : TokenKind
  word : String := ""
  num : Int := 0
  str : String := ""
  op : String := ""
  deriving Repr

def Char.isIdentStart (c : Char) : Bool := c.isAlpha ∨ c == '_'
def Char.isIdentCont (c : Char) : Bool := c.isAlphanum ∨ c == '_'

partial def tokenise (input : String) : Except String (List Token) :=
  let rec go (chars : List Char) (acc : List Token) : Except String (List Token) :=
    match chars with
    | [] => .ok acc.reverse
    | c :: rest =>
      if c.isWhitespace then go rest acc
      else if c == ',' then go rest ({ kind := .comma : Token } :: acc)
      else if c == ';' then go rest ({ kind := .semi : Token } :: acc)
      else if c == '*' then go rest ({ kind := .star : Token } :: acc)
      else if c == '\'' then
        let (str, rest') := rest.span (· != '\'')
        match rest' with
        | [] => .error "unterminated string"
        | _ :: rest'' => go rest'' ({ kind := .str, str := String.mk str : Token } :: acc)
      else if c == '=' then go rest ({ kind := .op, op := "=" : Token } :: acc)
      else if c == '<' ∨ c == '>' ∨ c == '!' then
        match rest with
        | '=' :: rest' =>
          go rest' ({ kind := .op, op := String.mk [c, '='] : Token } :: acc)
        | _ =>
          if c == '!' then .error "bad operator: !"
          else go rest ({ kind := .op, op := String.mk [c] : Token } :: acc)
      else if c.isDigit ∨ (c == '-' ∧ (rest.head?.map Char.isDigit).getD false) then
        let (digits, rest') :=
          if c == '-' then
            let (ds, rs) := rest.span Char.isDigit
            (c :: ds, rs)
          else
            let (ds, rs) := (c :: rest).span Char.isDigit
            (ds, rs)
        match (String.mk digits).toInt? with
        | some n => go rest' ({ kind := .number, num := n : Token } :: acc)
        | none => .error "bad number"
      else if c.isIdentStart then
        let (ident, rest') := (c :: rest).span Char.isIdentCont
        go rest' ({ kind := .word, word := (String.mk ident).toLower : Token } :: acc)
      else .error s!"unexpected char: {c}"
  go input.toList []

inductive SqlOrder where | asc | desc
  deriving Repr, BEq

structure SqlValue where
  isNum : Bool
  num : Int := 0
  text : String := ""
  deriving Repr

structure WhereClause where
  column : String
  op : String
  value : SqlValue
  deriving Repr

structure OrderBy where
  column : String
  order : SqlOrder
  deriving Repr

structure SqlQuery where
  projectionAll : Bool := false
  projectionColumns : List String := []
  table : String
  whereClause : Option WhereClause := none
  orderBy : Option OrderBy := none
  limit : Option Nat := none
  deriving Repr

inductive Severity where | info | warning | error
  deriving Repr, BEq

structure Finding where
  severity : Severity
  code : String
  message : String
  deriving Repr

structure Analysis where
  tables : List String
  columns : List String
  findings : List Finding
  deriving Repr

structure SqlSchema where
  tables : List (String × List String) := []
  deriving Repr

def SqlSchema.addTable (s : SqlSchema) (name : String) (columns : List String) : SqlSchema :=
  { tables := s.tables ++ [(name, columns)] }

def SqlSchema.lookup (s : SqlSchema) (name : String) : Option (List String) :=
  (s.tables.find? (fun p => p.1 == name)).map Prod.snd

def analyse (q : SqlQuery) (schema : SqlSchema) : Analysis :=
  let known := schema.lookup q.table
  let mut findings : List Finding := []
  if known.isNone then
    findings := findings ++ [{ severity := .error, code := "UNKNOWN_TABLE",
                                message := s!"unknown table: {q.table}" }]
  let mut columns : List String := []
  if !q.projectionAll then columns := q.projectionColumns
  match q.whereClause with | some w => columns := columns ++ [w.column] | none => ()
  match q.orderBy with | some o => columns := columns ++ [o.column] | none => ()
  columns := columns.eraseDups
  match known with
  | some cols =>
    for c in columns do
      if !cols.contains c then
        findings := findings ++ [{ severity := .error, code := "UNKNOWN_COLUMN",
                                    message := s!"unknown column {c} in table {q.table}" }]
  | none => ()
  if q.projectionAll then
    findings := findings ++ [{ severity := .warning, code := "SELECT_STAR",
                                message := "SELECT * pulls every column; name them explicitly" }]
  if q.limit.isSome ∧ q.orderBy.isNone then
    findings := findings ++ [{ severity := .warning, code := "LIMIT_WITHOUT_ORDER",
                                message := "LIMIT without ORDER BY returns arbitrary rows" }]
  if q.whereClause.isNone ∧ q.limit.isNone then
    findings := findings ++ [{ severity := .info, code := "FULL_SCAN",
                                message := "no WHERE or LIMIT — query will scan every row" }]
  { tables := [q.table], columns := columns.mergeSort (· < ·), findings }

end Databases.SQL
