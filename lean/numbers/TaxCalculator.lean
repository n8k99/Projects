/- Tax Calculator — progressive income tax computation.
   Uses Float for tax calculations with 2024 US federal single-filer brackets.
-/

namespace Numbers

structure BracketTax where
  bracketFloor : Float
  rate : Float
  taxable : Float
  tax : Float

structure TaxResult where
  totalTax : Float
  effectiveRate : Float
  marginalRate : Float
  breakdown : List BracketTax

/-- 2024 US federal single-filer brackets: (upper limit, rate). -/
private def brackets : List (Float × Float) :=
  [ (11600.0, 0.10)
  , (47150.0, 0.12)
  , (100525.0, 0.22)
  , (191950.0, 0.24)
  , (243725.0, 0.32)
  , (609350.0, 0.35)
  , (1.0e18, 0.37) ]

/-- Calculate progressive income tax for a given income. -/
def calculateTax (income : Float) : TaxResult :=
  if income <= 0.0 then
    { totalTax := 0.0
    , effectiveRate := 0.0
    , marginalRate := 0.0
    , breakdown := [] }
  else
    let rec loop (bs : List (Float × Float)) (prevLimit : Float)
                 (totalTax : Float) (marginalRate : Float)
                 (breakdown : List BracketTax) : TaxResult :=
      match bs with
      | [] =>
        { totalTax := totalTax
        , effectiveRate := totalTax / income
        , marginalRate := marginalRate
        , breakdown := breakdown.reverse }
      | (limit, rate) :: rest =>
        if income <= prevLimit then
          { totalTax := totalTax
          , effectiveRate := totalTax / income
          , marginalRate := marginalRate
          , breakdown := breakdown.reverse }
        else
          let cap := if income < limit then income else limit
          let taxable := cap - prevLimit
          let tax := taxable * rate
          let entry : BracketTax :=
            { bracketFloor := prevLimit
            , rate := rate
            , taxable := taxable
            , tax := tax }
          loop rest limit (totalTax + tax) rate (entry :: breakdown)
    loop brackets 0.0 0.0 0.0 []

end Numbers
