# G043 — Zip / Postal Code Lookup

> Resolve postal codes to location data.

```yaml
id: G043
title: Zip / Postal Code Lookup
category: networking
requires: [G012-distance-cities, G041-ip-lookup, G029-gift-suggestions]
provides: [address-resolution, radius-search, locale-detection, shipping-zones]
```

## Insight: The Third Resolution Layer

G012 resolved city names to coordinates. G041 resolved IP addresses to countries. G043 resolves postal codes to locations. Three layers of identity-to-context resolution, each mapping a different kind of identifier to geographic data:

| Project | Input | Resolution |
|---------|-------|-----------|
| G012 | City name | → coordinates |
| G041 | IP address | → country/city |
| G043 | Postal code | → city/state/coordinates |

The resolver's pattern is consistent: take an opaque identifier, look it up in a table, return enriched context. The identifier type varies. The operation doesn't. The Ghost Registry follows the same pattern: take a ghost name, return team/role/capabilities.

## Insight: Postal Codes Are Hierarchical Identifiers

US zip codes encode hierarchy: `3` = southeast, `32` = Florida, `322` = Jacksonville. The first digit is the region. The first three are the sectional center. The full five are the delivery area. This is namespace hierarchy — the same structure as `@forge/engineering/testing` or `domain.subdomain.host`.

The resolver's namespace could follow postal code logic: the prefix routes to the department, the middle to the team, the suffix to the specific agent. `@success.strategic.kathryn` would resolve hierarchically: success department → strategic office → Kathryn.

## Insight: Radius Search Is Proximity Discovery

`codes_within_radius(center, km)` finds all postal codes near a given one. This is G038's port scan generalized to geographic space: instead of probing ports on a host, probe the database for entries near a point. The pattern — enumerate candidates, measure distance, filter by threshold — is the same.

In the noosphere, capability search could work this way. "Find all agents within 2 hops of Kathryn's skill set" is a radius search in capability space. The distance metric changes (geographic → skill similarity), but the algorithm is identical.

## Insight: Multiple Indexes Serve Different Query Patterns

The database has three indexes: by-code (exact lookup), by-city (reverse lookup), by-state (regional aggregation). Each index serves a different access pattern. The same data, different entry points. This is the resolver's namespace viewed from multiple angles: lookup by name, search by capability, filter by department.

## Choreographic Case: Location-Aware Delivery

```innate
(@shipping-estimate){
  @origin <- @postal{code: "32202"}      ;; Jacksonville
  @destination <- @postal{code: "10001"} ;; New York
  @distance <- @haversine{from: @origin, to: @destination}
  @nearby_warehouses <- @postal/radius{center: @destination.code, km: 200}
  where {
    both_resolved: @origin != nil AND @destination != nil
    warehouse_available: @nearby_warehouses.length > 0
  }
}
```

## Structures

```innate
(defstruct postal-result
  postal-code  : String
  country-code : String
  country-name : String
  state        : String
  state-abbr   : String
  city         : String
  latitude     : Float
  longitude    : Float
  timezone     : String)
```

## Resolver Natives

```innate
@postal{code: String}                              -> PostalResult?
@postal/city{name: String}                         -> [PostalResult]
@postal/state{name: String}                        -> [PostalResult]
@postal/distance{a: String, b: String}             -> Float?
@postal/radius{center: String, km: Float}          -> [(PostalResult, Float)]
@postal/breakdown{codes: [String]}                 -> {String -> Nat}
```

## Demo

```innate
(@demo){
  @r <- @postal{code: "32202"}
  @print{@r.summary}
  ;; => 32202 -> Jacksonville, FL, US [30.3280, -81.6550]
  
  @d <- @postal/distance{a: "32202", b: "32084"}
  @print{@d}  ;; => ~50 km (Jacksonville to St. Augustine)
  
  @nearby <- @postal/radius{center: "32202", km: 100}
  @print{"Codes within 100km: " ++ @nearby.length}
}
```
