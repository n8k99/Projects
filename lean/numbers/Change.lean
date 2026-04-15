/-
  Change return program — break an amount into denominations.
-/

namespace Numbers

/-- Break amount (in cents) into fewest coins. Returns list of (denomination, count). -/
partial def makeChange (amount : Nat) (denominations : List Nat) : List (Nat × Nat) :=
  let denoms := denominations.mergeSort (· > ·)
  let rec go (remaining : Nat) (ds : List Nat) (acc : List (Nat × Nat)) : List (Nat × Nat) :=
    match ds with
    | [] => acc.reverse
    | d :: rest =>
      let count := remaining / d
      if count > 0 then
        go (remaining - count * d) rest ((d, count) :: acc)
      else
        go remaining rest acc
  go amount denoms []

end Numbers
