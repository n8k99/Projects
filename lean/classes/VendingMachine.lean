-- Vending Machine — a finite state machine with coin-constrained change-making.

namespace Classes

inductive VendingState where
  | idle
  | accepting
  deriving Repr, DecidableEq

def vendingDenominations : List Int := [100, 25, 10, 5, 1]

structure VendingSlot where
  id : String
  product : String
  priceCents : Int
  quantity : Int
  deriving Repr

structure VendingMachine where
  slots : List VendingSlot := []
  coinInventory : List (Int × Int) := []
  insertedCoins : List (Int × Int) := []
  insertedCents : Int := 0
  state : VendingState := .idle
  deriving Repr

def VendingMachine.empty : VendingMachine :=
  let init := vendingDenominations.map fun d => (d, (0 : Int))
  { coinInventory := init, insertedCoins := init }

def VendingMachine.findSlot (m : VendingMachine) (id : String) : Option VendingSlot :=
  m.slots.find? (·.id == id)

def VendingMachine.addSlot (m : VendingMachine) (s : VendingSlot)
    : Except String VendingMachine :=
  if m.slots.any (·.id == s.id) then .error s!"duplicate slot: {s.id}"
  else if s.priceCents ≤ 0 then .error "price must be positive"
  else .ok { m with slots := m.slots ++ [s] }

private def isValidDenom (d : Int) : Bool := vendingDenominations.contains d

private def updateKV (pairs : List (Int × Int)) (k : Int) (f : Int → Int)
    : List (Int × Int) :=
  if pairs.any (fun p => p.1 == k) then
    pairs.map fun p => if p.1 == k then (p.1, f p.2) else p
  else
    pairs ++ [(k, f 0)]

private def lookupKV (pairs : List (Int × Int)) (k : Int) : Int :=
  (pairs.find? fun p => p.1 == k).map (·.2) |>.getD 0

def VendingMachine.loadChange (m : VendingMachine) (denom count : Int)
    : Except String VendingMachine :=
  if ¬ isValidDenom denom then .error s!"invalid denomination: {denom}"
  else if count < 0 then .error "non-negative count required"
  else .ok { m with coinInventory := updateKV m.coinInventory denom (· + count) }

def VendingMachine.insertCoin (m : VendingMachine) (denom : Int)
    : Except String VendingMachine :=
  if ¬ isValidDenom denom then .error s!"invalid denomination: {denom}"
  else .ok { m with
    insertedCents := m.insertedCents + denom,
    insertedCoins := updateKV m.insertedCoins denom (· + 1),
    state := .accepting
  }

def VendingMachine.refund (m : VendingMachine) : VendingMachine × List (Int × Int) :=
  let refunded := m.insertedCoins.filter fun p => p.2 > 0
  let zeroed := m.insertedCoins.map fun p => (p.1, (0 : Int))
  ({ m with insertedCents := 0, insertedCoins := zeroed, state := .idle }, refunded)

/-- Greedy change: pool is denom→count pairs; try to make `amount` or fail. -/
partial def greedyChange (amount : Int) (pool : List (Int × Int))
    : Option (List (Int × Int)) :=
  let sortedDenoms := (pool.map (·.1)).mergeSort (· > ·)
  let rec loop (denoms : List Int) (remaining : Int) (acc : List (Int × Int))
      : Option (List (Int × Int)) :=
    if remaining == 0 then some acc
    else match denoms with
      | [] => none
      | d :: rest =>
        if d > remaining then loop rest remaining acc
        else
          let available := lookupKV pool d
          let use := min available (remaining / d)
          let newAcc := if use > 0 then acc ++ [(d, use)] else acc
          loop rest (remaining - d * use) newAcc
  loop sortedDenoms amount []

def VendingMachine.select (m : VendingMachine) (slotId : String)
    : Except String (VendingMachine × String × List (Int × Int)) := do
  if m.state ≠ .accepting then throw s!"cannot select while {repr m.state}"
  let slot ← (m.findSlot slotId).elim (throw s!"unknown slot: {slotId}") pure
  if slot.quantity ≤ 0 then throw s!"sold out: {slotId}"
  if m.insertedCents < slot.priceCents then
    throw s!"insufficient funds: {slot.priceCents} cents owed, {m.insertedCents} inserted"
  let changeOwed := m.insertedCents - slot.priceCents
  -- Pool = machine coins + inserted coins
  let pool := m.coinInventory.foldl (fun acc p => updateKV acc p.1 (· + p.2))
                (m.insertedCoins.foldl (fun acc p => updateKV acc p.1 (· + p.2)) [])
  let change ← (greedyChange changeOwed pool).elim
                  (throw s!"cannot make change for {changeOwed} cents") pure
  let poolAfter := change.foldl (fun p c => updateKV p c.1 (· - c.2)) pool
  let slotsAfter := m.slots.map fun s =>
    if s.id == slotId then { s with quantity := s.quantity - 1 } else s
  let zeroed := m.insertedCoins.map fun p => (p.1, (0 : Int))
  let mAfter : VendingMachine := {
    slots := slotsAfter,
    coinInventory := poolAfter,
    insertedCoins := zeroed,
    insertedCents := 0,
    state := .idle
  }
  pure (mAfter, slot.product, change.filter fun p => p.2 > 0)

end Classes
