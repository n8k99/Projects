-- Transaction Averages — grouped aggregation over financial records.

namespace Files.TA

structure Transaction where
  date : String
  category : String
  amountCents : Int
  deriving Repr

def Transaction.month (t : Transaction) : String :=
  if t.date.length ≥ 7 then t.date.extract 0 ⟨7⟩ else t.date

structure Stats where
  count : Nat := 0
  sumCents : Int := 0
  meanCents : Float := 0.0
  minCents : Int := 0
  maxCents : Int := 0
  deriving Repr

def Stats.new : Stats :=
  { minCents := 1000000000000, maxCents := -1000000000000 }

def Stats.observe (s : Stats) (amount : Int) : Stats :=
  { s with
    count := s.count + 1,
    sumCents := s.sumCents + amount,
    minCents := if amount < s.minCents then amount else s.minCents,
    maxCents := if amount > s.maxCents then amount else s.maxCents }

def Stats.finalise (s : Stats) : Stats :=
  if s.count == 0 then { s with meanCents := 0.0 }
  else { s with meanCents := s.sumCents.toFloat / s.count.toNat.toFloat }

def parseTransaction (line : String) : Option Transaction :=
  match line.splitOn "," with
  | [d, c, a] =>
    let date := d.trim
    let category := c.trim
    match a.trim.toInt? with
    | some amt =>
      if date.isEmpty || category.isEmpty then none
      else some { date, category, amountCents := amt }
    | none => none
  | _ => none

def parseTransactions (text : String) : List Transaction :=
  (text.splitOn "\n").filterMap fun l =>
    if l == "" then none else parseTransaction l

def groupBy (txns : List Transaction) (keyFn : Transaction → String)
    : List (String × Stats) :=
  -- Naive per-transaction insert into a list of pairs, then finalise.
  let folded := txns.foldl (init := (#[] : Array (String × Stats))) fun acc t =>
    let k := keyFn t
    match acc.findIdx? (fun p => p.1 == k) with
    | some i =>
      let (kk, s) := acc[i]!
      acc.set! i (kk, s.observe t.amountCents)
    | none => acc.push (k, Stats.new.observe t.amountCents)
  let asList := folded.toList.map fun (k, s) => (k, s.finalise)
  asList.mergeSort fun a b => a.1 < b.1

def byCategory (txns : List Transaction) : List (String × Stats) :=
  groupBy txns Transaction.category

def byMonth (txns : List Transaction) : List (String × Stats) :=
  groupBy txns Transaction.month

def totalStats (txns : List Transaction) : Stats :=
  (txns.foldl (init := Stats.new) fun s t => s.observe t.amountCents).finalise

def topNByMean (groups : List (String × Stats)) (n : Nat)
    : List (String × Stats) :=
  (groups.mergeSort fun a b => a.2.meanCents > b.2.meanCents).take n

end Files.TA
