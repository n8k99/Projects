/- Unit Converter — convert between measurement units. -/

namespace Numbers

/-- Category tag for ratio-based units. -/
inductive UnitCategory where
  | distance
  | weight
  | volume
deriving BEq, Repr

/-- Look up the category and base-unit factor for a ratio unit.
    Base units: meters (distance), grams (weight), liters (volume). -/
def unitInfo (unit : String) : Option (UnitCategory × Float) :=
  match unit with
  -- distance → meters
  | "km" => some (.distance, 1000.0)
  | "m"  => some (.distance, 1.0)
  | "mi" => some (.distance, 1609.344)
  | "ft" => some (.distance, 0.3048)
  -- weight → grams
  | "kg" => some (.weight, 1000.0)
  | "g"  => some (.weight, 1.0)
  | "lb" => some (.weight, 453.592)
  | "oz" => some (.weight, 28.3495)
  -- volume → liters
  | "L"   => some (.volume, 1.0)
  | "gal" => some (.volume, 3.78541)
  | "mL"  => some (.volume, 0.001)
  | "cup" => some (.volume, 0.236588)
  | _ => none

/-- Convert a temperature value to celsius. -/
def toCelsius (value : Float) (unit : String) : Option Float :=
  match unit with
  | "celsius"    => some value
  | "fahrenheit" => some ((value - 32.0) * 5.0 / 9.0)
  | "kelvin"     => some (value - 273.15)
  | _ => none

/-- Convert a celsius value to the target temperature unit. -/
def fromCelsius (value : Float) (unit : String) : Option Float :=
  match unit with
  | "celsius"    => some value
  | "fahrenheit" => some (value * 9.0 / 5.0 + 32.0)
  | "kelvin"     => some (value + 273.15)
  | _ => none

def isTemperature (unit : String) : Bool :=
  unit == "celsius" || unit == "fahrenheit" || unit == "kelvin"

/-- Convert a value from one measurement unit to another.
    Supports temperature (celsius, fahrenheit, kelvin),
    distance (km, mi, m, ft), weight (kg, lb, oz, g),
    and volume (L, gal, mL, cup). -/
def convertUnit (value : Float) (from to : String) : Option Float :=
  if from == to then some value
  else if isTemperature from || isTemperature to then
    if isTemperature from && isTemperature to then
      match toCelsius value from with
      | some c => fromCelsius c to
      | none => none
    else none
  else
    match unitInfo from, unitInfo to with
    | some (fromCat, fromFactor), some (toCat, toFactor) =>
      if fromCat == toCat then some (value * fromFactor / toFactor)
      else none
    | _, _ => none

end Numbers
