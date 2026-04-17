# G041 — Country from IP Lookup

> Resolve IP addresses to geographic location data.

```yaml
id: G041
title: Country from IP Lookup
category: networking
requires: [G012-distance-cities, G014-tax-calculator, G029-gift-suggestions, G040-packet-sniffer]
provides: [identity-to-context, geo-resolution, proximity-routing, jurisdiction-detection]
```

## Insight: Identity-to-Context Resolution

G012 (Distance Between Two Cities) looked up coordinates by city name — a human-readable identifier mapped to geographic data. G041 does the inverse: a machine identifier (IP address) mapped to geographic context. The IP tells you nothing about location on its own. The database supplies the context.

This is a new resolver pattern: **identity-to-context resolution**. Given an opaque identifier, enrich it with contextual information from a lookup table. The resolver already does this with `@references` — `@kathryn` is an opaque name that resolves to a full agent profile with role, capabilities, and department. IP geolocation is the same pattern applied to network addresses. The Ghost Registry is the GeoIP database of the noosphere.

## Insight: Private Addresses Are Unresolvable

`192.168.1.1` has no country. `10.0.0.1` has no city. Private addresses exist only within their local network — they have no global context. The lookup must recognize this and return "private/reserved" instead of guessing.

In InnateScript, some `@references` are similarly unresolvable outside their scope. A local variable inside a choreography has no meaning to agents outside that choreography. The private IP check is the resolver's scope boundary: "this identifier is valid, but only within a context I can't see."

## Insight: Binary Search Over Sorted Ranges Is the Resolver's Dispatch Table

The GeoIP database is a sorted list of ranges. Lookup is binary search: O(log n) to find which range contains the IP. This is the same dispatch mechanism the resolver uses for `@references` — given an identifier, binary search the namespace for the handler. G013's credit card prefix dispatch was linear scan. G014's tax brackets were sequential ranges. G041 formalizes the pattern: sorted ranges + binary search = fast dispatch.

## Insight: Proximity Routing Connects Geography to Choreography

`nearest_server(client_ip, server_ips)` finds the geographically closest server. This is G015's Dijkstra applied to physical distance: the "graph" is the set of servers, the "edges" are great-circle distances, and the "shortest path" is the nearest server. But unlike Dijkstra's abstract graph, the weights here are real-world distances derived from IP geolocation.

In the noosphere, agent selection could use proximity routing. When multiple agents can fulfill a request, choose the one closest to the data source. "Closest" might mean geographic proximity (for latency), organizational proximity (same department), or capability proximity (best skill match). The `nearest_server` pattern generalizes to `nearest_agent`.

## Insight: Country Breakdown Is `@breakdown` Applied to Identity

`country_breakdown(ips)` partitions IPs by country and counts per partition. This is G018's `@breakdown` (vowel frequency) and G020's `@breakdown` (word frequency) applied to network identities. The measurement primitive continues to generalize: characters → words → protocols (G040) → countries (G041). Same operation, ever-broader domains.

## Choreographic Case: Jurisdiction-Aware Operations

```innate
(@compliance-check){
  @visitor_geo <- @geo{ip: @visitor_ip}
  match @visitor_geo.country_code {
    "EU" | "GB" -> (@JMaxwellCharbourne){gdpr_compliance_check}
    "US" -> (@JMaxwellCharbourne){ccpa_compliance_check}
    "private" -> @skip   ;; internal traffic, no jurisdiction
    _ -> (@JMaxwellCharbourne){default_compliance_check}
  }
  where {
    jurisdiction_determined: @visitor_geo.country_code != ""
    compliance_verified: @compliance_result.passed
  }
}
```

The IP determines the jurisdiction. The jurisdiction determines the compliance rules. The rules determine which `<-` gates apply. Geography becomes policy through the lookup chain: IP → country → jurisdiction → rules → gates. G014's versioned rule engine meets G041's geographic resolution.

## Structures

```innate
(defstruct geo-result
  ip           : String
  country-code : String
  country-name : String
  region       : String
  city         : String
  latitude     : Float
  longitude    : Float
  timezone     : String
  is-private   : Bool)

(defstruct ip-range
  start        : Nat
  end          : Nat
  country-code : String
  country-name : String
  region       : String
  city         : String
  latitude     : Float
  longitude    : Float)
```

## Resolver Natives

```innate
@geo{ip: String}                                        -> GeoResult
@geo/bulk{ips: [String]}                                -> [GeoResult]
@geo/breakdown{ips: [String]}                           -> {String -> Nat}
@geo/nearest{client: String, servers: [String]}         -> String
@geo/distance{ip1: String, ip2: String}                 -> Float
@geo/is-private{ip: String}                             -> Bool
```

## Demo

```innate
(@demo){
  @db <- @geo-database{}
  @droplet <- @db/lookup{ip: "144.126.251.126"}
  @print{@droplet.summary}
  ;; => 144.126.251.126 -> New York, New York, United States (US) [40.7128, -74.0060]
  
  @nearest <- @db/nearest{
    client: "212.58.244.70",
    servers: ["8.8.8.8", "1.1.1.1"]
  }
  @print{@nearest}  ;; => 8.8.8.8 (US closer to London than AU)
}
```
