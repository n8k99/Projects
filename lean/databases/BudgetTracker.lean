-- Budget Tracker — envelope allocation, variance analysis, alert thresholds.

namespace Databases.BT

inductive Rollover where | reset | carry
  deriving Repr, BEq

structure Envelope where
  category : String
  budgetedCents : Int
  rollover : Rollover := .reset
  alertThresholdPct : Int := 80
  deriving Repr

structure Transaction where
  timestampMs : Int
  category : String
  amountCents : Int  -- negative = expense, positive = refund
  note : String := ""
  deriving Repr

structure CategoryStatus where
  category : String
  budgetedCents : Int
  spentCents : Int
  remainingCents : Int
  varianceCents : Int
  variancePct : Int
  overBudget : Bool
  alert : Bool
  deriving Repr

structure Budget where
  envelopes : List (String × Envelope) := []
  transactions : List Transaction := []
  deriving Repr

def Budget.addEnvelope (b : Budget) (e : Envelope) : Budget :=
  { b with envelopes := (b.envelopes.filter (fun p => p.1 ≠ e.category)) ++ [(e.category, e)] }

def Budget.record (b : Budget) (t : Transaction) : Budget :=
  { b with transactions := b.transactions ++ [t] }

def Budget.totalBudgeted (b : Budget) : Int :=
  b.envelopes.foldl (init := 0) fun acc p => acc + p.2.budgetedCents

def Budget.totalSpentBetween (b : Budget) (fromMs toMs : Int) : Int :=
  b.transactions.foldl (init := 0) fun acc t =>
    if t.timestampMs < fromMs ∨ t.timestampMs ≥ toMs then acc
    else acc + (-t.amountCents)

def spentBetween (txs : List Transaction) (fromMs toMs : Int) : List (String × Int) :=
  let filtered := txs.filter fun t => t.timestampMs ≥ fromMs ∧ t.timestampMs < toMs
  filtered.foldl (init := ([] : List (String × Int))) fun acc t =>
    let delta := -t.amountCents
    match acc.findIdx? (fun p => p.1 == t.category) with
    | some i => acc.mapIdx fun idx p => if idx == i then (t.category, p.2 + delta) else p
    | none => acc ++ [(t.category, delta)]

def Budget.statusBetween (b : Budget) (fromMs toMs : Int) : List CategoryStatus :=
  let spent := spentBetween b.transactions fromMs toMs
  let fromEnvs := b.envelopes.map fun (_, env) =>
    let s := (spent.find? (fun p => p.1 == env.category)).map Prod.snd |>.getD 0
    let remaining := env.budgetedCents - s
    let variance := s - env.budgetedCents
    let variancePct := if env.budgetedCents == 0 then -101
                        else variance * 100 / env.budgetedCents
    let over := s > env.budgetedCents
    let alert := env.budgetedCents > 0 ∧
                  (max 0 (s * 100 / env.budgetedCents)) ≥ env.alertThresholdPct
    ({ category := env.category, budgetedCents := env.budgetedCents,
       spentCents := s, remainingCents := remaining,
       varianceCents := variance, variancePct := variancePct,
       overBudget := over, alert } : CategoryStatus)
  let syntheticCats := spent.filter fun p =>
    !(b.envelopes.any (fun e => e.1 == p.1))
  let synthetics := syntheticCats.map fun (cat, s) =>
    ({ category := cat, budgetedCents := 0, spentCents := s,
       remainingCents := -s, varianceCents := s, variancePct := -101,
       overBudget := s > 0, alert := false } : CategoryStatus)
  (fromEnvs ++ synthetics).mergeSort (fun a b => a.category < b.category)

def Budget.statusAll (b : Budget) : List CategoryStatus :=
  b.statusBetween (-(1 <<< 62)) (1 <<< 62)

def Budget.alerts (b : Budget) : List CategoryStatus :=
  b.statusAll.filter (·.alert)

end Databases.BT
