-- Matrix Class — a 2D value class whose shape constrains its operations.

namespace Classes

structure Matrix where
  rows : List (List Float)
  deriving Repr

namespace Matrix

def nrows (m : Matrix) : Nat := m.rows.length

def ncols (m : Matrix) : Nat :=
  match m.rows.head? with
  | some row => row.length
  | none => 0

def shape (m : Matrix) : Nat × Nat := (m.nrows, m.ncols)

def isSquare (m : Matrix) : Bool := m.nrows == m.ncols

/-- Build a matrix, validating rectangularity. -/
def build (rows : List (List Float)) : Except String Matrix :=
  match rows with
  | [] => .error "matrix must have at least one row"
  | r :: _ =>
    if r.isEmpty then .error "matrix must have at least one column"
    else
      let cols := r.length
      if rows.any (fun x => x.length ≠ cols)
      then .error "row length mismatch"
      else .ok { rows }

def zeros (nrows ncols : Nat) : Except String Matrix :=
  if nrows == 0 || ncols == 0 then .error "non-positive dimension"
  else .ok { rows := List.replicate nrows (List.replicate ncols 0.0) }

def identity (n : Nat) : Except String Matrix := do
  let z ← zeros n n
  let rows := (List.range n).map fun i =>
    (List.range n).map fun j => if i == j then 1.0 else 0.0
  pure { rows }

def get (m : Matrix) (i j : Nat) : Float :=
  ((m.rows.get? i).getD []).get? j |>.getD 0.0

private def elementwise (a b : Matrix) (op : Float → Float → Float)
    : Except String Matrix :=
  if a.shape ≠ b.shape then
    .error s!"shape mismatch: {a.shape} vs {b.shape}"
  else
    .ok { rows := a.rows.zipWith (fun ra rb => ra.zipWith op rb) b.rows }

def add (a b : Matrix) : Except String Matrix := elementwise a b (· + ·)
def sub (a b : Matrix) : Except String Matrix := elementwise a b (· - ·)

def scalarMul (m : Matrix) (k : Float) : Matrix :=
  { rows := m.rows.map fun r => r.map (k * ·) }

/-- Matrix multiplication. Inner dimensions must match. -/
def matmul (a b : Matrix) : Except String Matrix :=
  if a.ncols ≠ b.nrows then
    .error s!"cannot multiply: inner dims differ {a.shape} * {b.shape}"
  else
    let m := a.nrows
    let p := b.ncols
    let n := a.ncols
    let rows := (List.range m).map fun i =>
      (List.range p).map fun j =>
        ((List.range n).foldl (fun s k =>
          s + ((a.rows.get? i).getD [] |>.get? k |>.getD 0.0)
              * ((b.rows.get? k).getD [] |>.get? j |>.getD 0.0)) 0.0)
    .ok { rows }

def transpose (m : Matrix) : Matrix :=
  let r := m.nrows
  let c := m.ncols
  let rows := (List.range c).map fun j =>
    (List.range r).map fun i =>
      ((m.rows.get? i).getD [] |>.get? j |>.getD 0.0)
  { rows }

/-- Cofactor-expansion determinant. -/
partial def det (rows : List (List Float)) : Float :=
  let n := rows.length
  if n == 1 then (rows.get? 0).getD [] |>.get? 0 |>.getD 0.0
  else if n == 2 then
    let r0 := (rows.get? 0).getD []
    let r1 := (rows.get? 1).getD []
    (r0.get? 0).getD 0.0 * (r1.get? 1).getD 0.0
      - (r0.get? 1).getD 0.0 * (r1.get? 0).getD 0.0
  else
    let total : Float := Id.run do
      let mut acc : Float := 0.0
      for c in [:n] do
        let minor : List (List Float) := (List.range (n - 1)).map fun i =>
          let row := (rows.get? (i + 1)).getD []
          (List.range n).filterMap fun k =>
            if k == c then none else some ((row.get? k).getD 0.0)
        let sign : Float := if c % 2 == 0 then 1.0 else -1.0
        let head0 := (rows.get? 0).getD []
        acc := acc + sign * ((head0.get? c).getD 0.0) * det minor
      pure acc
    total

def determinant (m : Matrix) : Except String Float :=
  if ¬ m.isSquare then .error s!"not square: {m.shape}"
  else .ok (det m.rows)

def approxEq (a b : Matrix) (eps : Float) : Bool :=
  if a.shape ≠ b.shape then false
  else
    (List.range a.nrows).all fun i =>
      (List.range a.ncols).all fun j =>
        let diff := a.get i j - b.get i j
        (if diff < 0.0 then -diff else diff) ≤ eps

end Matrix

end Classes
