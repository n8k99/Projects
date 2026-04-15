/-
  Find PI to the Nth digit.

  Lean 4's Nat is arbitrary precision. We use the Machin formula
  with scaled integer arithmetic: compute pi * 10^(n+margin),
  then format the result.

  pi/4 = 4 * arccot(5) - arccot(239)
-/

namespace Numbers

/-- Compute arccot(x) * 10^precision using Taylor series. -/
def arccot (x : Nat) (nTerms : Nat) (precision : Nat) : Int :=
  let scale := (10 ^ precision : Int) / (x : Int)
  let xSquared := (x * x : Int)
  let rec go (i : Nat) (power : Int) (sum : Int) (sign : Int) (k : Nat) : Int :=
    if i >= nTerms then sum
    else
      let term := sign * (power / (k : Int))
      let newPower := power / xSquared
      go (i + 1) newPower (sum + term) (-sign) (k + 2)
  go 0 scale 0 1 1

/-- Return PI to n decimal places as a String. -/
def pi (n : Nat) : String :=
  if n == 0 then "3"
  else
    let precision := n + 10
    let nTerms := precision + 10
    let result := 4 * (4 * arccot 5 nTerms precision - arccot 239 nTerms precision)
    let s := toString result
    let intPart := s.take 1
    let decPart := (s.drop 1).take n
    s!"{intPart}.{decPart}"

end Numbers
