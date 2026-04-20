-- Log File Maker — structured append-only log with level filtering and rotation.

namespace Files.LFM

inductive Level where | debug | info | warn | error
  deriving Repr, BEq, Inhabited

def Level.rank : Level → Nat
  | .debug => 0 | .info => 1 | .warn => 2 | .error => 3

def Level.geq (a b : Level) : Bool := a.rank ≥ b.rank

def Level.asStr : Level → String
  | .debug => "DEBUG" | .info => "INFO"
  | .warn => "WARN"   | .error => "ERROR"

def Level.parse (s : String) : Option Level :=
  match s with
  | "DEBUG" => some .debug | "INFO" => some .info
  | "WARN"  => some .warn  | "ERROR" => some .error
  | _ => none

structure Entry where
  timestamp : Nat
  level : Level
  category : String
  message : String
  deriving Repr

def Entry.toString (e : Entry) : String :=
  s!"{e.timestamp}\t{e.level.asStr}\t{e.category}\t{e.message}"

structure Logger where
  levelThreshold : Level := .debug
  maxEntries : Nat := 100
  active : List Entry := []
  archive : List Entry := []

def Logger.log (l : Logger) (ts : Nat) (level : Level) (category message : String)
    : Logger :=
  if level.geq l.levelThreshold then
    let newEntry : Entry := { timestamp := ts, level, category, message }
    let active := l.active ++ [newEntry]
    let overflow := active.length - l.maxEntries
    if overflow > 0 then
      let archived := active.take overflow
      { l with active := active.drop overflow, archive := l.archive ++ archived }
    else
      { l with active }
  else l

def Logger.queryLevel (l : Logger) (min : Level) : List Entry :=
  l.active.filter (fun e => e.level.geq min)

def Logger.queryCategory (l : Logger) (cat : String) : List Entry :=
  l.active.filter (fun e => e.category == cat)

def Logger.queryMessageContains (l : Logger) (needle : String) : List Entry :=
  let lc := needle.toLower
  l.active.filter (fun e =>
    (e.message.toLower.splitOn lc).length > 1 ∨ e.message.toLower == lc)

def Logger.queryTimeRange (l : Logger) (from' to' : Nat) : List Entry :=
  l.active.filter (fun e => from' ≤ e.timestamp ∧ e.timestamp ≤ to')

def Logger.toText (l : Logger) : String :=
  match l.active with
  | [] => ""
  | _ => "\n".intercalate (l.active.map Entry.toString) ++ "\n"

def parseEntries (text : String) : Option (List Entry) := do
  let mut out : List Entry := []
  for line in text.splitOn "\n" do
    if line == "" then continue
    let parts := line.splitOn "\t"
    if parts.length < 4 then return none
    let ts ← (parts[0]!).toNat?
    let level ← Level.parse (parts[1]!)
    -- Reconstruct message (may itself contain tabs if we used SplitN semantics,
    -- but since Lean's splitOn is non-greedy, accept only 4-part case):
    out := out ++ [{ timestamp := ts, level, category := parts[2]!, message := parts[3]! }]
  return out

end Files.LFM
