-- Handle Large Numbers — an arbitrary-precision signed integer as a class.
--
-- Note: Lean has native Nat (unbounded) and Int (unbounded) types built in.
-- We implement our own BigInt here for the Rosetta Stone exercise, showing the
-- schoolbook algorithms that Lean's core library provides for free.
-- Digits are base-10 little-endian; canonical form has no trailing zeros;
-- zero is { negative := false, digits := [] }.

namespace Classes

structure BigInt where
  negative : Bool := false
  digits : List Nat := []        -- little-endian base-10; [] = zero
  deriving Repr

def BigInt.zero : BigInt := { }

def BigInt.isZero (x : BigInt) : Bool := x.digits.isEmpty

/-- Build from a Nat (non-negative path). -/
def BigInt.ofNat (n : Nat) : BigInt :=
  if n == 0 then BigInt.zero
  else
    let rec loop (n : Nat) (acc : List Nat) : List Nat :=
      if n == 0 then acc
      else loop (n / 10) (acc ++ [n % 10])
    { negative := false, digits := loop n [] }

/-- Build from an Int. -/
def BigInt.ofInt (n : Int) : BigInt :=
  if n == 0 then BigInt.zero
  else if n < 0 then
    { negative := true, digits := (BigInt.ofNat n.natAbs).digits }
  else
    BigInt.ofNat n.natAbs

/-- Parse a decimal string. -/
def BigInt.parse (s : String) : Except String BigInt :=
  let t := s.trim
  if t.isEmpty then .error "empty string"
  else
    let (neg, body) :=
      if t.front == '-' then (true, t.drop 1)
      else if t.front == '+' then (false, t.drop 1)
      else (false, t)
    if body.isEmpty then .error s!"not a decimal integer: {s}"
    else if ¬ body.all Char.isDigit then .error s!"not a decimal integer: {s}"
    else
      let digs := body.toList.reverse.map fun c => (c.toNat - '0'.toNat)
      -- Strip trailing zeros (high end).
      let stripped := digs.foldr (fun d acc =>
        match acc with
        | [] => if d == 0 then [] else [d]
        | _  => d :: acc) []
      if stripped.isEmpty then .ok BigInt.zero
      else .ok { negative := neg, digits := stripped }

/-- Format as decimal with optional leading minus. -/
def BigInt.toString (x : BigInt) : String :=
  if x.isZero then "0"
  else
    let body := (x.digits.reverse.map fun d => Char.ofNat (d + '0'.toNat)).asString
    if x.negative then "-" ++ body else body

instance : ToString BigInt := ⟨BigInt.toString⟩

def BigInt.neg (x : BigInt) : BigInt :=
  if x.isZero then x else { x with negative := ¬ x.negative }

def BigInt.abs (x : BigInt) : BigInt := { x with negative := false }

/-- Compare absolute values: returns -1, 0, or +1 as an Int. -/
def cmpMagnitude (a b : List Nat) : Int :=
  if a.length < b.length then -1
  else if a.length > b.length then 1
  else
    let rec go (xs ys : List Nat) : Int :=
      match xs, ys with
      | [], [] => 0
      | x :: xs', y :: ys' =>
        match go xs' ys' with
        | 0 => if x < y then -1 else if x > y then 1 else 0
        | r => r
      | _, _ => 0
    go a b

/-- Total order: -big < -small < 0 < +small < +big. -/
def BigInt.cmp (x y : BigInt) : Int :=
  if x.isZero && y.isZero then 0
  else if x.negative && ¬ y.negative then -1
  else if ¬ x.negative && y.negative then 1
  else if ¬ x.negative then cmpMagnitude x.digits y.digits
  else cmpMagnitude y.digits x.digits

def BigInt.equal (x y : BigInt) : Bool := x.cmp y == 0

/-- Schoolbook add with carry over little-endian base-10 digits. -/
partial def addDigits (a b : List Nat) : List Nat :=
  let rec go (a b : List Nat) (carry : Nat) (acc : List Nat) : List Nat :=
    match a, b with
    | [], [] => if carry == 0 then acc.reverse else (carry :: acc).reverse
    | x :: xs, [] =>
      let s := x + carry
      go xs [] (s / 10) (s % 10 :: acc)
    | [], y :: ys =>
      let s := y + carry
      go [] ys (s / 10) (s % 10 :: acc)
    | x :: xs, y :: ys =>
      let s := x + y + carry
      go xs ys (s / 10) (s % 10 :: acc)
  go a b 0 []

/-- Strip trailing zeros (high end of little-endian list). -/
def stripTrailingZeros (xs : List Nat) : List Nat :=
  let rec recurse (xs : List Nat) : List Nat :=
    xs.foldr (fun d acc =>
      match acc with
      | [] => if d == 0 then [] else [d]
      | _  => d :: acc) []
  recurse xs

/-- Schoolbook subtract, assuming |a| ≥ |b|. -/
partial def subDigits (a b : List Nat) : List Nat :=
  let rec go (a b : List Nat) (borrow : Int) (acc : List Int) : List Int :=
    match a, b with
    | [], _ => acc.reverse
    | x :: xs, [] =>
      let d := (x : Int) - borrow
      if d < 0 then go xs [] 1 ((d + 10) :: acc)
      else go xs [] 0 (d :: acc)
    | x :: xs, y :: ys =>
      let d := (x : Int) - (y : Int) - borrow
      if d < 0 then go xs ys 1 ((d + 10) :: acc)
      else go xs ys 0 (d :: acc)
  let raw := (go a b 0 []).map Int.toNat
  stripTrailingZeros raw

/-- Schoolbook multiply via shifted partial products. -/
def mulDigits (a b : List Nat) : List Nat :=
  if a.isEmpty || b.isEmpty then []
  else
    let n := a.length + b.length
    let init : Array Nat := Array.mkArray n 0
    let acc := a.foldlIdx (fun i acc da =>
      let carryAndAcc := b.foldlIdx (fun j (state : Nat × Array Nat) db =>
        let (carry, a0) := state
        let s := a0.getD (i + j) 0 + da * db + carry
        (s / 10, a0.set! (i + j) (s % 10))) (0, acc)
      let (carry, a1) := carryAndAcc
      a1.modify (i + b.length) (· + carry)) init
    stripTrailingZeros acc.toList

def BigInt.add (x y : BigInt) : BigInt :=
  if x.negative == y.negative then
    if x.isZero && y.isZero then BigInt.zero
    else { negative := x.negative, digits := addDigits x.digits y.digits }
  else
    let c := cmpMagnitude x.digits y.digits
    if c == 0 then BigInt.zero
    else if c > 0 then
      { negative := x.negative, digits := subDigits x.digits y.digits }
    else
      { negative := y.negative, digits := subDigits y.digits x.digits }

def BigInt.sub (x y : BigInt) : BigInt := x.add y.neg

def BigInt.mul (x y : BigInt) : BigInt :=
  if x.isZero || y.isZero then BigInt.zero
  else
    { negative := x.negative ≠ y.negative, digits := mulDigits x.digits y.digits }

end Classes
