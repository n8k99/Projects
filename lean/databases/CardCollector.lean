-- Card Collector — multiset inventory with valuation and trade balancing.

namespace Databases.CC

inductive Rarity where | common | uncommon | rare | mythic
  deriving Repr, BEq, Inhabited

def Rarity.multiplier : Rarity → Float
  | .common => 1.0 | .uncommon => 2.0 | .rare => 8.0 | .mythic => 25.0

inductive Condition where | mint | nearMint | played | damaged
  deriving Repr, BEq, Inhabited

def Condition.multiplier : Condition → Float
  | .mint => 1.0 | .nearMint => 0.8 | .played => 0.4 | .damaged => 0.15

structure Card where
  setCode : String
  number : Nat
  name : String
  rarity : Rarity
  basePriceCents : Nat
  deriving Repr

def Card.id (c : Card) : String :=
  let num := toString c.number
  let padded :=
    if num.length ≥ 3 then num
    else (String.mk (List.replicate (3 - num.length) '0')) ++ num
  c.setCode ++ "-" ++ padded

def Card.valueCents (c : Card) (cond : Condition) : Nat :=
  (c.basePriceCents.toFloat * c.rarity.multiplier * cond.multiplier).toUInt32.toNat

structure Catalogue where
  cards : List (String × Card) := []
  deriving Repr

def Catalogue.add (cat : Catalogue) (c : Card) : Catalogue :=
  { cards := cat.cards ++ [(c.id, c)] }

def Catalogue.get (cat : Catalogue) (id : String) : Option Card :=
  (cat.cards.find? (fun p => p.1 == id)).map Prod.snd

def Catalogue.allIds (cat : Catalogue) : List String :=
  (cat.cards.map Prod.fst).mergeSort (· < ·)

structure InventoryKey where
  cardId : String
  condition : Condition
  deriving Repr, BEq

structure Collection where
  counts : List (InventoryKey × Nat) := []
  deriving Repr

def Collection.add (c : Collection) (cardId : String) (cond : Condition) (qty : Nat)
    : Collection :=
  let k : InventoryKey := { cardId, condition := cond }
  match c.counts.findIdx? (fun p => p.1 == k) with
  | some i =>
    { counts := c.counts.mapIdx fun idx p =>
        if idx == i then (k, p.2 + qty) else p }
  | none => { counts := c.counts ++ [(k, qty)] }

def Collection.totalCards (c : Collection) : Nat :=
  c.counts.foldl (init := 0) (fun acc p => acc + p.2)

def Collection.uniqueCards (c : Collection) : Nat :=
  ((c.counts.map fun p => p.1.cardId).eraseDups).length

def Collection.totalValueCents (c : Collection) (cat : Catalogue) : Nat :=
  c.counts.foldl (init := 0) fun acc (k, n) =>
    match cat.get k.cardId with
    | some card => acc + card.valueCents k.condition * n
    | none => acc

def Collection.missing (c : Collection) (cat : Catalogue) : List String :=
  let owned := (c.counts.map fun p => p.1.cardId).eraseDups
  cat.allIds.filter (fun id => !owned.contains id)

structure TradeBundleItem where
  cardId : String
  condition : Condition
  qty : Nat
  deriving Repr

structure TradeBundle where
  cards : List TradeBundleItem := []
  deriving Repr

def TradeBundle.add (b : TradeBundle) (cardId : String) (cond : Condition) (qty : Nat)
    : TradeBundle :=
  { cards := b.cards ++ [{ cardId, condition := cond, qty }] }

def TradeBundle.valueCents (b : TradeBundle) (cat : Catalogue) : Nat :=
  b.cards.foldl (init := 0) fun acc it =>
    match cat.get it.cardId with
    | some c => acc + c.valueCents it.condition * it.qty
    | none => acc

inductive TradeVerdict where | fair | offeredMore | askedMore
  deriving Repr, BEq

structure TradeEvaluation where
  verdict : TradeVerdict
  diffCents : Nat
  deriving Repr

def evaluateTrade (offered asked : TradeBundle) (cat : Catalogue) (tolerance : Nat)
    : TradeEvaluation :=
  let o := offered.valueCents cat
  let a := asked.valueCents cat
  let diff := if o ≥ a then o - a else a - o
  if diff ≤ tolerance then { verdict := .fair, diffCents := 0 }
  else if o > a then { verdict := .offeredMore, diffCents := o - a }
  else { verdict := .askedMore, diffCents := a - o }

end Databases.CC
