-- Excel Spreadsheet Exporter — cell-addressed grid, formulae, CSV export.

namespace Files.XL

inductive CellKind where | blank | number | text | formula
  deriving Repr, BEq

structure Cell where
  kind : CellKind
  number : Float := 0.0
  text : String := ""
  formula : String := ""
  deriving Repr

inductive Value where
  | empty
  | number (n : Float)
  | text (s : String)
  | err (msg : String)
  deriving Repr

def Value.asNumber : Value → Option Float
  | .number n => some n
  | .text s => s.toFloat?
  | _ => none

def parseAddress (addr : String) : Option (Nat × Nat) :=
  if addr.length < 2 then none
  else
    let first := addr.get 0
    if !first.isAlpha then none
    else
      let col := (first.toUpper.toNat - 'A'.toNat)
      match (addr.drop 1).toNat? with
      | some r => if r ≥ 1 then some (r - 1, col) else none
      | none => none

def formatAddress (row col : Nat) : String :=
  String.mk [Char.ofNat ('A'.toNat + col)] ++ toString (row + 1)

structure Sheet where
  cells : List ((Nat × Nat) × Cell) := []

def Sheet.set (s : Sheet) (addr : String) (cell : Cell) : Sheet :=
  match parseAddress addr with
  | none => s
  | some pos =>
    let filtered := s.cells.filter fun p => p.1 ≠ pos
    { cells := filtered ++ [(pos, cell)] }

def Sheet.setNumber (s : Sheet) (addr : String) (n : Float) : Sheet :=
  s.set addr { kind := .number, number := n }

def Sheet.setText (s : Sheet) (addr : String) (t : String) : Sheet :=
  s.set addr { kind := .text, text := t }

def Sheet.setFormula (s : Sheet) (addr : String) (f : String) : Sheet :=
  s.set addr { kind := .formula, formula := f }

partial def evalCell (s : Sheet) (pos : Nat × Nat) (visiting : List (Nat × Nat))
    : Value :=
  if visiting.contains pos then .err "circular reference"
  else
    match s.cells.find? (fun p => p.1 == pos) with
    | none => .empty
    | some (_, c) =>
      match c.kind with
      | .blank => .empty
      | .number => .number c.number
      | .text => .text c.text
      | .formula => evalFormula s c.formula (pos :: visiting)

where evalFormula (s : Sheet) (f : String) (visiting : List (Nat × Nat)) : Value :=
  let body := (f.dropWhile (· == '=')).trim
  match body.indexOf? '+', body.indexOf? '-', body.indexOf? '*', body.indexOf? '/' with
  | some p, _, _, _ =>
    let l := evalOperand s ((body.extract 0 p).trim) visiting
    let r := evalOperand s ((body.extract (p + ⟨1⟩) ⟨body.length⟩).trim) visiting
    applyOp '+' l r
  | _, some p, _, _ =>
    if p.byteIdx = 0 then evalOperand s body visiting
    else
      let l := evalOperand s ((body.extract 0 p).trim) visiting
      let r := evalOperand s ((body.extract (p + ⟨1⟩) ⟨body.length⟩).trim) visiting
      applyOp '-' l r
  | _, _, some p, _ =>
    let l := evalOperand s ((body.extract 0 p).trim) visiting
    let r := evalOperand s ((body.extract (p + ⟨1⟩) ⟨body.length⟩).trim) visiting
    applyOp '*' l r
  | _, _, _, some p =>
    let l := evalOperand s ((body.extract 0 p).trim) visiting
    let r := evalOperand s ((body.extract (p + ⟨1⟩) ⟨body.length⟩).trim) visiting
    applyOp '/' l r
  | _, _, _, _ => evalOperand s body visiting

and evalOperand (s : Sheet) (str : String) (visiting : List (Nat × Nat)) : Value :=
  let t := str.trim
  match t.toFloat? with
  | some n => .number n
  | none =>
    match parseAddress t with
    | some pos => evalCell s pos visiting
    | none => .err s!"bad operand: {t}"

and applyOp (op : Char) (l r : Value) : Value :=
  match l.asNumber, r.asNumber with
  | some a, some b =>
    match op with
    | '+' => .number (a + b)
    | '-' => .number (a - b)
    | '*' => .number (a * b)
    | '/' => if b == 0.0 then .err "div by zero" else .number (a / b)
    | _ => .err "bad op"
  | _, _ => .err "non-numeric operand"

def Sheet.evaluate (s : Sheet) (addr : String) : Value :=
  match parseAddress addr with
  | none => .err s!"bad address: {addr}"
  | some pos => evalCell s pos []

end Files.XL
