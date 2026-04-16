/- Fortune Teller — randomized wisdom from categorized pools. -/

namespace Text

/-- A fortune/prediction generator with categorized pools of wisdom. -/
structure FortuneTeller where
  pools : List (String × List String) := []
  deriving Repr

namespace FortuneTeller

/-- The default fortune pools. -/
def defaultPools : List (String × List String) :=
  [ ("wisdom", [
      "The obstacle is the path.",
      "What you resist persists.",
      "Still water runs deep.",
      "The map is not the territory.",
      "Every expert was once a beginner."]),
    ("humor", [
      "You will step on a Lego in the dark tonight.",
      "A closed mouth gathers no foot.",
      "Today is a good day to avoid making eye contact.",
      "Your socks will never match again."]),
    ("warning", [
      "Beware the shortcut that saves no time.",
      "Not every open door is an invitation.",
      "The comfortable path leads to the boring destination.",
      "Trust your instincts — they remember what you forgot."]),
    ("motivation", [
      "You are closer than you think.",
      "The best time to start was yesterday. The second best is now.",
      "Doubt kills more dreams than failure ever will.",
      "Small steps still move you forward.",
      "Discipline is choosing what you want most over what you want now."]),
    ("mystery", [
      "Something lost will find you when you stop looking.",
      "The answer you seek is in a room you haven't entered yet.",
      "A stranger already knows your name.",
      "The next full moon brings clarity."])
  ]

/-- Create a fortune teller pre-loaded with default fortunes. -/
def create : FortuneTeller :=
  { pools := defaultPools }

/-- Gather all fortunes across all categories. -/
def allFortunes (ft : FortuneTeller) : List String :=
  ft.pools.bind (·.2)

/-- Return the total number of fortunes. -/
def fortuneCount (ft : FortuneTeller) : Nat :=
  ft.allFortunes.length

/-- Get the pool for a given category. -/
def getPool (ft : FortuneTeller) (category : String) : Option (List String) :=
  match ft.pools.find? (fun p => p.1 == category) with
  | some (_, pool) => if pool.isEmpty then none else some pool
  | none => none

/-- Select an element at a given index (mod length), for use with IO.rand. -/
def selectAt (xs : List String) (idx : Nat) : String :=
  match xs.get? (idx % xs.length) with
  | some s => s
  | none => "The spirits are silent."

/-- Return a random fortune from all categories (in IO for randomness). -/
def tellFortune (ft : FortuneTeller) : IO String := do
  let all := ft.allFortunes
  if all.isEmpty then
    return "The spirits are silent."
  let idx ← IO.rand 0 (all.length - 1)
  return selectAt all idx

/-- Return a random fortune from the specified category. -/
def tellFortuneByCategory (ft : FortuneTeller) (category : String) : IO (Option String) := do
  match ft.getPool category with
  | none => return none
  | some pool =>
    let idx ← IO.rand 0 (pool.length - 1)
    return some (selectAt pool idx)

/-- Add a fortune to a category pool. -/
def addFortune (ft : FortuneTeller) (text : String) (category : String) : FortuneTeller :=
  let updated := ft.pools.map fun (cat, pool) =>
    if cat == category then (cat, pool ++ [text]) else (cat, pool)
  let exists := ft.pools.any (fun p => p.1 == category)
  if exists then { pools := updated }
  else { pools := ft.pools ++ [(category, [text])] }

/-- Acknowledge the question, return a random fortune regardless. -/
def askQuestion (ft : FortuneTeller) (_question : String) : IO String :=
  ft.tellFortune

end FortuneTeller
end Text
