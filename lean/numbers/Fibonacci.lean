/-
  Fibonacci sequence — generate to a count or up to a value.
  Lean's Nat is arbitrary precision — no overflow.
-/

namespace Numbers

/-- Return the nth Fibonacci number (0-indexed). -/
def fib : Nat → Nat
  | 0 => 0
  | 1 => 1
  | n + 2 => fib (n + 1) + fib n

/-- Return the nth Fibonacci number using tail-recursive accumulator. -/
def fibFast (n : Nat) : Nat :=
  let rec go (i : Nat) (a b : Nat) : Nat :=
    if i == 0 then a
    else go (i - 1) b (a + b)
  go n 0 1

/-- Return the first n Fibonacci numbers as a list. -/
def fibonacci (n : Nat) : List Nat :=
  let rec go (i : Nat) (a b : Nat) (acc : List Nat) : List Nat :=
    if i == 0 then acc.reverse
    else go (i - 1) b (a + b) (a :: acc)
  go n 0 1 []

/-- Return all Fibonacci numbers up to and including limit. -/
def fibonacciUpTo (limit : Nat) : List Nat :=
  let rec go (a b : Nat) (acc : List Nat) : List Nat :=
    if a > limit then acc.reverse
    else go b (a + b) (a :: acc)
  go 0 1 []

end Numbers
