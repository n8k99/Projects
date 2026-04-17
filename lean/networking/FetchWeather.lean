-- Fetch Current Weather — structured weather data from external APIs.
-- Pure data model and parsing. Lean's stdlib doesn't include HTTP, so the
-- module defines structures, conversions, and a JSON-like parse layer that
-- can be driven by any IO transport.

namespace Networking

structure WindInfo where
  speedMps : Float
  directionDeg : Float
  gustMps : Option Float := none
  deriving Repr

namespace WindInfo

def speedMph (w : WindInfo) : Float := w.speedMps * 2.23694
def speedKmh (w : WindInfo) : Float := w.speedMps * 3.6

private def cardinalDirs : Array String :=
  #["N", "NNE", "NE", "ENE", "E", "ESE", "SE", "SSE",
    "S", "SSW", "SW", "WSW", "W", "WNW", "NW", "NNW"]

def cardinal (w : WindInfo) : String :=
  let idx := (Float.round (w.directionDeg / 22.5)).toUInt32.toNat % 16
  cardinalDirs[idx]!

end WindInfo

structure WeatherCondition where
  id : Nat
  main : String
  description : String
  icon : String
  deriving Repr

structure WeatherReport where
  city : String
  country : String
  latitude : Float
  longitude : Float
  temperatureK : Float
  feelsLikeK : Float
  humidity : Nat
  pressureHpa : Nat
  visibilityM : Nat
  conditions : List WeatherCondition := []
  wind : WindInfo
  cloudsPct : Nat := 0
  timestamp : Nat := 0
  sunrise : Option Nat := none
  sunset : Option Nat := none
  deriving Repr

namespace WeatherReport

def temperatureC (r : WeatherReport) : Float := r.temperatureK - 273.15
def temperatureF (r : WeatherReport) : Float := r.temperatureC * 9.0 / 5.0 + 32.0
def feelsLikeC (r : WeatherReport) : Float := r.feelsLikeK - 273.15
def feelsLikeF (r : WeatherReport) : Float := r.feelsLikeC * 9.0 / 5.0 + 32.0

def description (r : WeatherReport) : String :=
  match r.conditions with
  | c :: _ => c.description
  | [] => "unknown"

def summary (r : WeatherReport) : String :=
  s!"{r.city}, {r.country}: {r.temperatureC}°C, {r.description}, humidity {r.humidity}%, wind {r.wind.speedMps} m/s {r.wind.cardinal}"

end WeatherReport

-- Query types for weather lookup
inductive WeatherQuery where
  | byCity (city : String) (countryCode : Option String := none)
  | byCoords (lat : Float) (lon : Float)
  | byZip (zip : String) (countryCode : String := "US")
  deriving Repr

-- Build the query path (minus API key) for an OWM request.
def WeatherQuery.toPath (q : WeatherQuery) (apiKey : String) : String :=
  let base := "/data/2.5/weather"
  match q with
  | .byCity city none => s!"{base}?q={city}&appid={apiKey}"
  | .byCity city (some cc) => s!"{base}?q={city},{cc}&appid={apiKey}"
  | .byCoords lat lon => s!"{base}?lat={lat}&lon={lon}&appid={apiKey}"
  | .byZip zip cc => s!"{base}?zip={zip},{cc}&appid={apiKey}"

-- Minimal key-value extraction from a flat JSON-like string.
-- Not a full parser — handles the OWM response shape.
private def findJsonValue (json : String) (key : String) : Option String :=
  let needle := "\"" ++ key ++ "\""
  match json.findSubstr? needle with
  | none => none
  | some pos =>
    let after := json.drop (pos + needle.length)
    let after := after.dropWhile (· != ':') |>.drop 1 |>.trimLeft
    if after.isEmpty then none
    else if after.front == '\"' then
      let rest := after.drop 1
      let endQ := rest.findIdx (· == '\"')
      some (rest.take endQ)
    else
      let endV := after.findIdx fun c => c == ',' || c == '}' || c == ']' || c == ' '
      some (after.take endV |>.trim)

-- Helper to search for a substring position
private def String.findSubstr? (s : String) (needle : String) : Option Nat := do
  let sLen := s.length
  let nLen := needle.length
  if nLen > sLen then none
  else
    let mut i := 0
    for _ in [:sLen - nLen + 1] do
      if s.extract ⟨i⟩ ⟨i + nLen⟩ == needle then
        return i
      i := i + 1
    none

private def parseFloat (s : String) : Float :=
  s.trim.toFloat?.getD 0.0

private def parseNat (s : String) : Nat :=
  s.trim.toNat?.getD 0

-- Parse an OWM JSON response into a WeatherReport.
-- This is intentionally minimal — a real application would use a JSON library.
def parseWeatherReport (json : String) : Except String WeatherReport := do
  let city := (findJsonValue json "name").getD "Unknown"
  let country := (findJsonValue json "country").getD "??"
  let lat := parseFloat ((findJsonValue json "lat").getD "0")
  let lon := parseFloat ((findJsonValue json "lon").getD "0")
  let temp := parseFloat ((findJsonValue json "temp").getD "0")
  let feelsLike := parseFloat ((findJsonValue json "feels_like").getD "0")
  let humidity := parseNat ((findJsonValue json "humidity").getD "0")
  let pressure := parseNat ((findJsonValue json "pressure").getD "0")
  let vis := parseNat ((findJsonValue json "visibility").getD "0")
  let speed := parseFloat ((findJsonValue json "speed").getD "0")
  let deg := parseFloat ((findJsonValue json "deg").getD "0")
  let clouds := parseNat ((findJsonValue json "all").getD "0")
  let dt := parseNat ((findJsonValue json "dt").getD "0")

  let condMain := (findJsonValue json "main").getD ""
  let condDesc := (findJsonValue json "description").getD ""
  let condIcon := (findJsonValue json "icon").getD ""
  let condId := parseNat ((findJsonValue json "id").getD "0")

  .ok {
    city := city
    country := country
    latitude := lat
    longitude := lon
    temperatureK := temp
    feelsLikeK := feelsLike
    humidity := humidity
    pressureHpa := pressure
    visibilityM := vis
    conditions := [{ id := condId, main := condMain, description := condDesc, icon := condIcon }]
    wind := { speedMps := speed, directionDeg := deg }
    cloudsPct := clouds
    timestamp := dt
  }

-- Compare weather across locations.
structure WeatherComparison where
  warmest : String
  coldest : String
  mostHumid : String
  windiest : String
  deriving Repr

def compareWeather (reports : List WeatherReport) : Except String WeatherComparison := do
  match reports with
  | [] => .error "No reports to compare"
  | r :: rs =>
    let warmest := rs.foldl (init := r) fun best r =>
      if r.temperatureK > best.temperatureK then r else best
    let coldest := rs.foldl (init := r) fun best r =>
      if r.temperatureK < best.temperatureK then r else best
    let mostHumid := rs.foldl (init := r) fun best r =>
      if r.humidity > best.humidity then r else best
    let windiest := rs.foldl (init := r) fun best r =>
      if r.wind.speedMps > best.wind.speedMps then r else best
    .ok {
      warmest := warmest.city
      coldest := coldest.city
      mostHumid := mostHumid.city
      windiest := windiest.city
    }

-- Demo: construct reports from hardcoded data (no IO/HTTP needed)
def demo : IO Unit := do
  let jacksonville : WeatherReport := {
    city := "Jacksonville"
    country := "US"
    latitude := 30.33
    longitude := -81.66
    temperatureK := 300.0
    feelsLikeK := 302.0
    humidity := 65
    pressureHpa := 1013
    visibilityM := 10000
    conditions := [{ id := 800, main := "Clear", description := "clear sky", icon := "01d" }]
    wind := { speedMps := 3.5, directionDeg := 180.0 }
    cloudsPct := 0
    timestamp := 1700000000
  }
  IO.println jacksonville.summary

end Networking
