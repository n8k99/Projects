/- Credit Card Validator — Luhn algorithm and network identification. -/

namespace Numbers

/-- Extract digit characters from a string as a list of natural numbers. -/
def digitsOnly (s : String) : List Nat :=
  s.toList.filterMap fun c =>
    if c.isDigit then some (c.toNat - '0'.toNat) else none

/-- Return only digit characters from a string. -/
def digitsString (s : String) : String :=
  ⟨s.toList.filter Char.isDigit⟩

/-- Validate a credit card number using the Luhn algorithm. -/
def luhnCheck (number : String) : Bool :=
  let digits := (digitsOnly number).reverse
  if digits.length < 2 then false
  else
    let total := digits.enum.foldl (fun acc (i, d) =>
      let v := if i % 2 == 1 then
        let doubled := d * 2
        if doubled > 9 then doubled - 9 else doubled
      else d
      acc + v) 0
    total % 10 == 0

/-- Parse the first n characters of a string as a natural number. -/
def prefixNat (s : String) (len : Nat) : Nat :=
  let sub := (s.toList.take len).asString
  sub.toNat!

/-- Identify the card network by prefix and length.
    Returns some network name or none. -/
def identifyNetwork (number : String) : Option String :=
  let digits := digitsString number
  let n := digits.length
  if n < 2 then none
  else
    let p2 := (digits.toList.take 2).asString
    -- Amex: 34xx or 37xx, length 15
    if n == 15 && (p2 == "34" || p2 == "37") then some "Amex"
    -- Visa: starts with 4, length 13 or 16
    else if digits.get ⟨0⟩ == '4' && (n == 13 || n == 16) then some "Visa"
    else if n == 16 then
      let prefix2 := prefixNat digits 2
      let prefix4 := prefixNat digits 4
      -- Mastercard: 51-55 or 2221-2720
      if (51 ≤ prefix2 && prefix2 ≤ 55) || (2221 ≤ prefix4 && prefix4 ≤ 2720) then
        some "Mastercard"
      -- Discover: 6011, 644-649, 65xx
      else
        let p4 := (digits.toList.take 4).asString
        let prefix3 := prefixNat digits 3
        if p4 == "6011" || (644 ≤ prefix3 && prefix3 ≤ 649) || p2 == "65" then
          some "Discover"
        else none
    else none

/-- Result of credit card validation. -/
structure CardValidation where
  valid : Bool
  network : Option String
  display : String
  deriving Repr

/-- Validate a credit card number: Luhn check + network identification. -/
def validateCard (number : String) : CardValidation :=
  let digits := digitsString number
  let n := digits.length
  let last4 := if n >= 4 then (digits.toList.drop (n - 4)).asString else digits
  let display := "****" ++ last4
  let network := identifyNetwork digits
  let valid := luhnCheck digits && network.isSome
  { valid, network, display }

end Numbers
