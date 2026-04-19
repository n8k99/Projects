-- Josephus Problem — circular elimination and the survivor.
--
-- N people stand in a circle (0..n). Starting at 0, count k and remove
-- that person; continue from the next remaining position. The last one
-- standing is the survivor.
--
-- Two different queries on the same process:
--   josephus            — O(n) recurrence, survivor only.
--   josephusSimulate    — O(n²) list simulation, survivor + elimination order.
--
-- The recurrence is the first closed-form shortcut in Rosetta Stone Classes:
-- a problem whose answer can be computed without materialising the process.

namespace Classes

/-- Survivor via the recurrence J(1,k)=0 ; J(n,k) = (J(n-1,k) + k) mod n. -/
def josephus (n k : Nat) : Nat :=
  let rec loop (m survivor : Nat) : Nat :=
    if m > n then survivor
    else loop (m + 1) ((survivor + k) % m)
  loop 2 0

/-- Remove the element at index `i`, returning (removed, remaining). -/
private def removeAt {α} (xs : List α) (i : Nat) : Option (α × List α) :=
  match xs, i with
  | [],      _     => none
  | x :: rest, 0   => some (x, rest)
  | x :: rest, n+1 =>
    match removeAt rest n with
    | none => none
    | some (v, rest') => some (v, x :: rest')

/-- Simulate the full elimination. Returns (survivor, elimination order). -/
partial def josephusSimulate (n k : Nat) : Nat × List Nat :=
  let initial : List Nat := (List.range n)
  let rec loop (ring : List Nat) (pos : Nat) (order : List Nat)
      : Nat × List Nat :=
    match ring with
    | [x] => (x, order.reverse)
    | _ =>
      let len := ring.length
      let p := (pos + k - 1) % len
      match removeAt ring p with
      | none => (0, order.reverse)  -- unreachable when ring non-empty
      | some (v, rest) => loop rest p (v :: order)
  loop initial 0 []

end Classes
