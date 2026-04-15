/-
  Mortgage calculator — monthly payments, amortization, pace tracking.
  Uses Float for financial calculations.
-/

namespace Numbers

structure MortgageResult where
  monthlyPayment : Float
  totalPayments : Nat
  totalPaid : Float
  totalInterest : Float

/-- Calculate fixed monthly payment for a mortgage. -/
def monthlyPayment (principal : Float) (annualRate : Float) (years : Nat) : Float :=
  if annualRate == 0.0 then
    principal / (years.toFloat * 12.0)
  else
    let r := annualRate / 100.0 / 12.0
    let n := years.toFloat * 12.0
    let factor := (1.0 + r) ^ n
    principal * r * factor / (factor - 1.0)

/-- Calculate total cost of a mortgage. -/
def mortgageTotalCost (principal : Float) (annualRate : Float) (years : Nat) : MortgageResult :=
  let pmt := monthlyPayment principal annualRate years
  let n := years * 12
  let total := pmt * n.toFloat
  { monthlyPayment := pmt
  , totalPayments := n
  , totalPaid := total
  , totalInterest := total - principal }

structure PaceResult where
  expectedByNow : Float
  actual : Float
  delta : Float
  pacePercent : Float
  onTrack : Bool

/-- Check if cumulative payments are on pace to meet target. -/
def paceCheck (target : Float) (periods : Nat) (currentPeriod : Nat) (cumulative : Float) : PaceResult :=
  let expected := target * currentPeriod.toFloat / periods.toFloat
  let delta := cumulative - expected
  let pace := if expected > 0.0 then cumulative / expected * 100.0 else 0.0
  { expectedByNow := expected
  , actual := cumulative
  , delta := delta
  , pacePercent := pace
  , onTrack := delta >= 0.0 }

end Numbers
