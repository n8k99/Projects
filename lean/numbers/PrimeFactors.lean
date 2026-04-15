/-
  Prime factorization — find all prime factors of a number.
-/

namespace Numbers

/-- Return the prime factors of n in ascending order, with repetition. -/
partial def primeFactors (n : Nat) : List Nat :=
  if n < 2 then []
  else
    let rec go (n : Nat) (d : Nat) (acc : List Nat) : List Nat :=
      if d * d > n then
        if n > 1 then (n :: acc).reverse
        else acc.reverse
      else if n % d == 0 then
        go (n / d) d (d :: acc)
      else
        go n (d + 1) acc
    go n 2 []

/-- Return prime factorization as a list of (prime, exponent) pairs. -/
def primeFactorization (n : Nat) : List (Nat × Nat) :=
  let factors := primeFactors n
  let rec go (fs : List Nat) (acc : List (Nat × Nat)) : List (Nat × Nat) :=
    match fs with
    | [] => acc.reverse
    | f :: rest =>
      match acc with
      | (p, e) :: tail =>
        if p == f then go rest ((p, e + 1) :: tail)
        else go rest ((f, 1) :: (p, e) :: tail)
      | [] => go rest [(f, 1)]
  go factors []

end Numbers
