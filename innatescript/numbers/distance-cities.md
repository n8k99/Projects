# Distance Between Two Cities

A choreography that computes the great-circle distance between two points on Earth using the Haversine formula.

## Design feedback

Distance between cities is the first Rosetta Stone problem where the resolver needs external data. The Haversine formula itself is pure math — `sin`, `cos`, `atan2`, all derivable from first principles. But the coordinates of Tokyo are not. They are facts about the world. `@city{name: "Tokyo"}` doesn't compute Tokyo's latitude — it looks it up.

This introduces a new resolver primitive: the fact lookup. Previous problems had computation (`@calc`), representation (`@base`), and scheduling (`@alarm`). Now there is a fourth: world-state. The resolver needs both a computational engine and a fact store. In the Rosetta Stone implementations, the fact store is a hardcoded lookup table. In InnateScript, the fact store is the database — Postgres on the droplet. `@city{name: "Tokyo"} -> @coords` bottoms out in a database query, not a calculation.

The distinction between computed values and looked-up values matters architecturally. Computed values are deterministic and timeless — `sin(0.5)` is always `sin(0.5)`. Looked-up values are contingent and mutable — a city's coordinates could be refined, a new city could be added. The resolver must treat these differently: computed values can be cached forever, fact lookups must respect the freshness of their source.

Who maintains the city database? In the other five languages, it's a compile-time constant. In InnateScript, it's a shared mutable resource that agents can read and potentially update. This is the first time the Rosetta Stone has encountered shared mutable state. The fact store is a coordination surface — agents reading from it need to agree on the same snapshot, or accept eventual consistency.

```dpn
@haversine{lat1: 40.7128, lon1: -74.0060, lat2: 51.5074, lon2: -0.1278} -> 5570.25
@city_distance{from: "New York", to: "London"} -> 5570.25
@city_distance{from: "Tokyo", to: "Sydney"} -> 7795.28
@city_distance{from: "São Paulo", to: "Cairo"} -> 9185.98
```

The `@city_distance` native composes two operations: fact lookup (`@city -> @coords`) and computation (`@haversine`). The resolver chains them. The lookup resolves first — it must, because the computation depends on its output. This is a data dependency, the same kind the resolver handles everywhere, but the source of the data is different. It comes from the world, not from the computation graph.

## Choreographic case

Logistics and travel planning where agents in different cities need to coordinate based on physical proximity. The core question: which agent is closest?

```dpn
# Warehouse proximity selection
@delivery{address: "customer location"} -> @dest_coords

concurrent [
    @warehouse{name: "warehouse_tokyo"} -> @w1_coords
    @warehouse{name: "warehouse_sydney"} -> @w2_coords
    @warehouse{name: "warehouse_cairo"} -> @w3_coords
]
join

concurrent [
    @haversine{lat1: @dest_coords.lat, lon1: @dest_coords.lon,
               lat2: @w1_coords.lat, lon2: @w1_coords.lon} -> @d1
    @haversine{lat1: @dest_coords.lat, lon1: @dest_coords.lon,
               lat2: @w2_coords.lat, lon2: @w2_coords.lon} -> @d2
    @haversine{lat1: @dest_coords.lat, lon1: @dest_coords.lon,
               lat2: @w3_coords.lat, lon2: @w3_coords.lon} -> @d3
]
join

@select{min: [@d1, @d2, @d3]} -> @nearest
@dispatch{warehouse: @nearest.source, destination: @dest_coords}
```

Multiple agents report their locations concurrently — the `concurrent` block fetches all coordinates in parallel. Then distances are computed in parallel (each `@haversine` is independent). Finally, `@select{min: ...}` picks the nearest. The `where` scores by proximity.

This extends to dynamic fleet coordination:

```dpn
# Ride-hailing: find nearest available driver
@rider{request: "pickup"} -> @pickup_coords

@query{resource: "drivers", filter: {status: "available"}} -> @drivers

@map{over: @drivers, apply: @haversine{
    lat1: @pickup_coords.lat, lon1: @pickup_coords.lon,
    lat2: @_.lat, lon2: @_.lon
}} -> @distances

@select{min: @distances} -> @nearest_driver
@dispatch{driver: @nearest_driver.id, pickup: @pickup_coords}
```

Here the fact store is live — the list of available drivers changes moment to moment. The resolver queries the DB for current state, computes distances against each, and selects the minimum. The computation is pure, but the data feeding it is real-time.

## What this means

Distance between cities reveals the boundary between computation and world-state. The Haversine formula is mathematics. City coordinates are facts. InnateScript needs both: a resolver for computation and a fact store for world-state. The fact store is the database. The resolver chains lookups and computations seamlessly — the data dependency graph doesn't care whether a value was computed or looked up.

The choreographic case shows why this matters for multi-agent systems. Proximity-based coordination is everywhere: logistics, ride-hailing, resource allocation, emergency response. Every instance follows the same pattern: gather locations (fact lookups, concurrent), compute distances (pure math, concurrent), select by proximity (aggregation). The resolver's ability to mix fact lookups with computation in the same dependency graph makes this natural.

The shared mutable fact store introduces a new concern: consistency. If two agents query the city database at different times and get different coordinates (because someone updated Tokyo's entry), their distance calculations won't agree. The resolver needs a consistency model for fact lookups — at minimum, snapshot reads within a single choreography. This is the beginning of the transaction story.

## Native implementation

The resolver provides `@haversine{lat1, lon1, lat2, lon2}` as a built-in computation native — pure math, no external state. City lookup is a DB-backed resource: `@city{name: N}` queries the fact store and returns `{lat, lon}`. `@city_distance{from, to}` composes both: look up two cities, compute Haversine. The host language provides trigonometric functions. Unknown cities produce resolver errors. The Earth radius constant (6371 km) is baked into the native.
