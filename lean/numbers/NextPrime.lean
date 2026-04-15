/-
  Next prime number — continuously find primes on demand.
-/

namespace Numbers

/-- Test whether n is prime. -/
def isPrime (n : Nat) : Bool :=
  if n < 2 then false
  else if n < 4 then true
  else if n % 2 == 0 || n % 3 == 0 then false
  else
    let rec go (d : Nat) : Bool :=
      if d * d > n then true
      else if n % d == 0 || n % (d + 2) == 0 then false
      else go (d + 6)
    go 5

/-- Return the smallest prime greater than n. -/
partial def nextPrime (n : Nat) : Nat :=
  let start := if n >= 2 then n + 1 else 2
  if start == 2 then 2
  else
    let candidate := if start % 2 == 0 then start + 1 else start
    let rec go (c : Nat) : Nat :=
      if isPrime c then c
      else go (c + 2)
    go candidate

/-- Return the nth prime (1-indexed). -/
partial def nthPrime (n : Nat) : Nat :=
  let rec go (candidate : Nat) (count : Nat) : Nat :=
    if isPrime candidate then
      if count + 1 == n then candidate
      else go (candidate + if candidate == 2 then 1 else 2) (count + 1)
    else go (candidate + if candidate == 2 then 1 else 2) count
  go 2 0

end Numbers
