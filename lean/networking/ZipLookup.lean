-- Zip / Postal Code Lookup — resolve postal codes to location data.

namespace Networking

structure PostalResult' where
  postalCode : String
  countryCode : String := ""
  countryName : String := ""
  state : String := ""
  stateAbbr : String := ""
  city : String := ""
  latitude : Float := 0.0
  longitude : Float := 0.0
  deriving Repr

def PostalResult'.summary (r : PostalResult') : String :=
  let st := if r.stateAbbr.isEmpty then r.state else r.stateAbbr
  let parts := [r.city, st, r.countryCode].filter (!·.isEmpty)
  s!"{r.postalCode} -> {String.intercalate \", \" parts}"

private def postalDB : List PostalResult' := [
  { postalCode := "32202", countryCode := "US", state := "Florida", stateAbbr := "FL",
    city := "Jacksonville", latitude := 30.328, longitude := -81.655 },
  { postalCode := "32084", countryCode := "US", state := "Florida", stateAbbr := "FL",
    city := "St. Augustine", latitude := 29.8947, longitude := -81.3145 },
  { postalCode := "33101", countryCode := "US", state := "Florida", stateAbbr := "FL",
    city := "Miami", latitude := 25.7617, longitude := -80.1918 },
  { postalCode := "10001", countryCode := "US", state := "New York", stateAbbr := "NY",
    city := "New York", latitude := 40.7484, longitude := -73.9967 },
  { postalCode := "90210", countryCode := "US", state := "California", stateAbbr := "CA",
    city := "Beverly Hills", latitude := 34.0901, longitude := -118.4065 },
  { postalCode := "SW1A1AA", countryCode := "GB", state := "England",
    city := "London", latitude := 51.5014, longitude := -0.1419 },
  { postalCode := "75001", countryCode := "FR", state := "Île-de-France",
    city := "Paris", latitude := 48.8606, longitude := 2.3376 }
]

def postalLookup' (code : String) : Option PostalResult' :=
  let norm := code.toUpper.replace " " ""
  postalDB.find? (·.postalCode.toUpper.replace " " "" == norm)

def searchByCity' (city : String) : List PostalResult' :=
  postalDB.filter (·.city.toLower == city.toLower)

def searchByState' (state : String) : List PostalResult' :=
  let s := state.toLower
  postalDB.filter fun e => e.state.toLower == s || e.stateAbbr.toLower == s

-- Demo
def zipLookupDemo : IO Unit := do
  let codes := ["32202", "10001", "90210", "SW1A 1AA", "75001"]
  for code in codes do
    match postalLookup' code with
    | some r => IO.println r.summary
    | none => IO.println s!"{code} -> not found"

end Networking
