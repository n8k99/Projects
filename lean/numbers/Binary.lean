/-
  Binary to Decimal and Back — convert between number systems.
-/

namespace Numbers

/-- Convert a natural number to its binary string. -/
partial def toBinary (n : Nat) : String :=
  if n == 0 then "0"
  else
    let rec go (val : Nat) (acc : List Char) : List Char :=
      if val == 0 then acc
      else go (val / 2) ((if val % 2 == 1 then '1' else '0') :: acc)
    String.mk (go n [])

/-- Convert a binary string to its decimal value. -/
def toDecimal (s : String) : Option Nat :=
  if s.isEmpty then none
  else
    let chars := s.toList
    if chars.all (fun c => c == '0' || c == '1') then
      some (chars.foldl (fun acc c => acc * 2 + (if c == '1' then 1 else 0)) 0)
    else none

/-- Convert a natural number to a string in the given base (2-36). -/
partial def toBase (n : Nat) (base : Nat) : Option String :=
  if base < 2 || base > 36 then none
  else if n == 0 then some "0"
  else
    let digits := "0123456789abcdefghijklmnopqrstuvwxyz".toList
    let rec go (val : Nat) (acc : List Char) : List Char :=
      if val == 0 then acc
      else go (val / base) (digits.get! (val % base) :: acc)
    some (String.mk (go n []))

/-- Convert a string in the given base (2-36) to decimal. -/
def fromBase (s : String) (base : Nat) : Option Nat :=
  if base < 2 || base > 36 || s.isEmpty then none
  else
    let digits := "0123456789abcdefghijklmnopqrstuvwxyz".toList
    let chars := s.toLower.toList
    let vals := chars.map (fun c => digits.indexOf? c)
    if vals.any Option.isNone then none
    else
      let nums := vals.map (fun v => v.get!)
      if nums.any (· >= base) then none
      else some (nums.foldl (fun acc v => acc * base + v) 0)

end Numbers
