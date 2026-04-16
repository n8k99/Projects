/- Distance Between Two Cities — great-circle distance via Haversine. -/

namespace Numbers

/-- Earth's mean radius in kilometres. -/
def earthRadiusKm : Float := 6371.0

/-- City coordinates: (name, latitude, longitude) in degrees. -/
def cityTable : List (String × Float × Float) :=
  [ ("New York",   40.7128,  -74.0060)
  , ("London",     51.5074,   -0.1278)
  , ("Tokyo",      35.6762,  139.6503)
  , ("Sydney",    -33.8688,  151.2093)
  , ("São Paulo", -23.5505,  -46.6333)
  , ("Cairo",      30.0444,   31.2357)
  ]

private def deg2rad (d : Float) : Float :=
  d * Float.pi / 180.0

/-- Great-circle distance in km between two points given in degrees. -/
def haversine (lat1 lon1 lat2 lon2 : Float) : Float :=
  let lat1 := deg2rad lat1
  let lon1 := deg2rad lon1
  let lat2 := deg2rad lat2
  let lon2 := deg2rad lon2
  let dlat := lat2 - lat1
  let dlon := lon2 - lon1
  let a := (dlat / 2.0).sin ^ 2 + lat1.cos * lat2.cos * (dlon / 2.0).sin ^ 2
  let c := 2.0 * Float.atan2 a.sqrt (1.0 - a).sqrt
  earthRadiusKm * c

private def findCity (name : String) : Option (Float × Float) :=
  match cityTable.find? (fun c => c.1 == name) with
  | some (_, lat, lon) => some (lat, lon)
  | none => none

/-- Great-circle distance in km between two named cities. -/
def cityDistance (city1 city2 : String) : Option Float :=
  match findCity city1, findCity city2 with
  | some (lat1, lon1), some (lat2, lon2) => some (haversine lat1 lon1 lat2 lon2)
  | _, _ => none

end Numbers
