-- RPG Character Stat Creator — class templates + derived stats + level progression.

namespace Files.RPG

inductive Stat where | strength | dexterity | intellect | stamina | luck
  deriving Repr, BEq, Inhabited

def allStats : List Stat := [.strength, .dexterity, .intellect, .stamina, .luck]

def Stat.name : Stat → String
  | .strength => "strength" | .dexterity => "dexterity"
  | .intellect => "intellect" | .stamina => "stamina" | .luck => "luck"

inductive CharClass where | warrior | mage | rogue
  deriving Repr, BEq

def CharClass.name : CharClass → String
  | .warrior => "warrior" | .mage => "mage" | .rogue => "rogue"

def multiplier (c : CharClass) (s : Stat) : Float :=
  match c, s with
  | .warrior, .strength => 1.5
  | .warrior, .stamina => 1.3
  | .mage, .intellect => 1.5
  | .mage, .stamina => 0.8
  | .rogue, .dexterity => 1.4
  | .rogue, .luck => 1.3
  | _, _ => 1.0

def maxPerStat (_ : CharClass) : Nat := 99

structure Derived where
  maxHP : Nat
  maxMP : Nat
  armour : Nat
  carryCapacity : Nat
  critChance : Float
  deriving Repr

structure Character where
  name : String
  cls : CharClass
  level : Nat := 1
  base : List (Stat × Nat) := []
  unspentPoints : Nat := 0
  deriving Repr

structure Lcg where
  state : UInt64
  deriving Repr

def Lcg.seed (s : UInt64) : Lcg := { state := s + 1 }

def Lcg.nextU32 (l : Lcg) : (UInt32 × Lcg) :=
  let next := l.state * 6364136223846793005 + 1442695040888963407
  ((next >>> 33).toUInt32, { state := next })

def Lcg.nextIn (l : Lcg) (lo range : UInt32) : (UInt32 × Lcg) :=
  let (v, l') := l.nextU32
  (lo + v % range, l')

def rollCharacter (name : String) (cls : CharClass) (seed : UInt64) : Character :=
  let rng := Lcg.seed seed
  let (base, _) := allStats.foldl (init := (([] : List (Stat × Nat)), rng))
    fun (acc, rng) s =>
      let (v, rng') := rng.nextIn 0 16
      (acc ++ [(s, 3 + v.toNat)], rng')
  { name, cls, level := 1, base, unspentPoints := 0 }

def Character.lookup (c : Character) (s : Stat) : Nat :=
  (c.base.find? (fun p => p.1 == s)).map (·.2) |>.getD 0

def Character.adjusted (c : Character) (s : Stat) : Nat :=
  let b := c.lookup s
  let m := multiplier c.cls s
  (b.toFloat * m).toUInt32.toNat

def Character.derived (c : Character) : Derived :=
  let str := c.adjusted .strength
  let int_ := c.adjusted .intellect
  let sta := c.adjusted .stamina
  let dex := c.adjusted .dexterity
  let luk := c.adjusted .luck
  { maxHP := 10 + sta * 5 + c.level * 10,
    maxMP := int_ * 3 + c.level * 5,
    armour := str / 3 + sta / 5,
    carryCapacity := 10 + str * 2,
    critChance := (luk.toFloat * 0.5 + dex.toFloat * 0.2) / 100.0 }

def Character.levelUp (c : Character) : Character :=
  { c with level := c.level + 1, unspentPoints := c.unspentPoints + 5 }

def Character.spendPoint (c : Character) (s : Stat) (points : Nat)
    : Except String Character := do
  if points > c.unspentPoints then
    throw s!"not enough points: have {c.unspentPoints}, need {points}"
  let current := c.lookup s
  let newVal := current + points
  if newVal > maxPerStat c.cls then
    throw s!"stat cap exceeded: {newVal} > {maxPerStat c.cls}"
  let base' := c.base.map fun p => if p.1 == s then (s, newVal) else p
  return { c with base := base', unspentPoints := c.unspentPoints - points }

end Files.RPG
