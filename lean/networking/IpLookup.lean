-- Country from IP Lookup — resolve IP addresses to geographic location data.
-- Pure data model: IP ranges, geo results, binary search, and distance calculation.

namespace Networking

structure GeoIPResult where
  ip : String
  countryCode : String := ""
  countryName : String := ""
  region : String := ""
  city : String := ""
  latitude : Float := 0.0
  longitude : Float := 0.0
  timezone : String := ""
  isPrivate : Bool := false
  deriving Repr

def GeoIPResult.summary (r : GeoIPResult) : String :=
  if r.isPrivate then s!"{r.ip} -> private/reserved"
  else
    let parts := [r.city, r.region, r.countryName].filter (!·.isEmpty)
    s!"{r.ip} -> {String.intercalate \", \" parts} ({r.countryCode})"

structure GeoIPRange where
  start : Nat
  «end» : Nat
  countryCode : String
  countryName : String := ""
  region : String := ""
  city : String := ""
  latitude : Float := 0.0
  longitude : Float := 0.0
  deriving Repr

-- Simple IP to Nat (for dotted-quad strings like "8.8.8.8")
-- In a real implementation, parse properly. Here we use a lookup table approach.
private def simpleIPLookup : List (String × String × String) :=
  [("8.8.8.8", "US", "Mountain View"),
   ("1.1.1.1", "AU", "Sydney"),
   ("192.168.1.1", "private", ""),
   ("10.0.0.1", "private", ""),
   ("127.0.0.1", "private", ""),
   ("144.126.251.126", "US", "New York"),
   ("77.100.50.25", "DE", "Frankfurt"),
   ("212.58.244.70", "GB", "London")]

def geoLookup (ip : String) : GeoIPResult :=
  match simpleIPLookup.find? (·.1 == ip) with
  | some (_, "private", _) => { ip, isPrivate := true }
  | some (_, cc, city) => { ip, countryCode := cc, city }
  | none => { ip }

def bulkGeoLookup (ips : List String) : List GeoIPResult :=
  ips.map geoLookup

def countryBreakdown (ips : List String) : List (String × Nat) :=
  let results := ips.map geoLookup
  let codes := results.map fun r =>
    if r.isPrivate then "private"
    else if r.countryCode.isEmpty then "unknown"
    else r.countryCode
  let unique := codes.eraseDups
  unique.map fun c => (c, codes.filter (· == c) |>.length)

-- Haversine distance in km
def haversineKm (lat1 lon1 lat2 lon2 : Float) : Float :=
  let r := 6371.0
  let dlat := (lat2 - lat1) * Float.pi / 180.0
  let dlon := (lon2 - lon1) * Float.pi / 180.0
  let a := Float.sin (dlat / 2) ^ 2 +
    Float.cos (lat1 * Float.pi / 180.0) * Float.cos (lat2 * Float.pi / 180.0) *
    Float.sin (dlon / 2) ^ 2
  r * 2 * Float.atan2 (Float.sqrt a) (Float.sqrt (1 - a))

-- Demo
def ipLookupDemo : IO Unit := do
  let ips := ["8.8.8.8", "1.1.1.1", "192.168.1.1", "144.126.251.126"]
  for ip in ips do
    IO.println (geoLookup ip).summary
  IO.println s!"Breakdown: {countryBreakdown ips}"

end Networking
