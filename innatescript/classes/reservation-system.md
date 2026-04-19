# G050 — Reservation System

> Bookable resources with time intervals. Overlap is the core primitive; availability is a negative search.

```yaml
id: G050
title: Reservation System
category: classes
requires: [G049-movie-store]
provides: [interval-overlap, specific-resources, scheduled-obligation, conflict-detection]
```

## Insight: Specific Resources, Not Fungible Copies

G049 modeled movies as fungible — any of 2 copies of Casablanca satisfies a rental. G050 modifies the pattern: seat 12A is specifically 12A. Room 304 has a specific view. If you reserve 12A, you can't be handed 12B. This is the first model in the Rosetta Stone with **identity at the resource level**, not just at the title level. The resource table is the primary key space; fungibility becomes a special case (multiple resources of the same kind with equivalent attributes).

The noosphere has both. The `conversations` table: thread 1a2b3c is specifically that thread, not interchangeable with any other. The `tasks` table: task #247 is a specific task. The temporal calendar: `[[2026-04-19]]` is specifically that day — not a fungible "some day in April." G050 describes how the vault models things that can't be substituted.

## Insight: Intervals Conflict, Points Don't

A movie rental (G049) is either active or not *at a point in time*. A reservation is a **span**. Two spans either overlap or they don't. Overlap detection is the core primitive, and it has a clean definition:

```
[a, b) overlaps [c, d)  ≡  a < d ∧ c < b
```

Half-open intervals matter: a reservation ending at 3:00 and another starting at 3:00 do not conflict. This is the same convention the temporal calendar uses — `[[2026-04-19]]` is `[April 19 00:00, April 20 00:00)`. Adjacent days don't overlap. The half-open convention is how discrete time composes without ambiguity.

InnateScript choreographies that coordinate over intervals — "Sarah handles the morning, Kathryn the afternoon" — should use the same half-open semantics by default. Two agents with touching intervals don't conflict; the handoff is clean.

## Insight: Availability Is a Negative Search

For G049, "what's available?" = products where `on_hand > 0`. For G050, "what's available for [start, end)?" = resources with *no reservation* overlapping that range. The query is structured as **negation over conflicts**: for each resource, check `conflicts(resource, start, end)` is empty.

This generalizes. Finding a free agent for a task = agents with no active choreography during the needed window. Finding a free meeting slot = times with no existing event in any participant's calendar. Finding a quiet period for deployment = time ranges with no open incident, no change freeze, no release. Every scheduling question in the noosphere is "empty conflict set" over intervals.

## Insight: Cancel Is the First Non-Atomic Retraction

Rent/return (G049) happened at discrete moments: rent now, return later. A reservation is different — it's a **future claim**. The interval `[start, end)` may not have arrived yet when you cancel. Canceling retracts an intention before it takes full effect; returning releases an effect that already started.

This is the shape of scheduled choreographies. A `@nightly-summary` scheduled for midnight has a future claim on Lena's attention. If it's cancelled at 22:00, it retracts. If it runs and Lena partially executes, "cancel" wouldn't make sense — you'd need a "return partial" semantics. InnateScript needs both: a `cancel` primitive for future choreographies and a `retract` or `rollback` primitive for in-flight ones. G050 surfaces the distinction that G049 didn't need.

## Choreographic Case: Conflict-Free Calendar Assignment

```innate
(@assign-agent-window){
  @window <- @proposed-window{start: @t0, end: @t1}
  @candidates <- @agents/available{start: @window.start, end: @window.end, capability: "forex"}

  @selected <- @candidates.choose_by{strategy: "least-loaded"}
  @reservation <- @calendar/book{
    resource: @selected,
    customer: @choreography-id,
    start: @window.start,
    end: @window.end
  } <- {no_conflict}

  where {
    no_conflict: @reservation.status == booked
    agent_available: @candidates.length > 0
  }
}
```

The choreography proposes a window, finds capable agents who are free in that window, books one. The `<- {no_conflict}` gate uses the reservation system's conflict check as an assertion. If all candidates conflict, the booking fails and the `where` reports *why*. This is how the calendar and the choreography engine unify.

## Structures

```innate
(defstruct resource
  id    : String
  kind  : String          ; "seat", "room", "slot", "agent-window"
  label : String)

(defstruct reservation
  id          : Nat
  resource-id : String
  customer-id : String
  start       : Timestamp
  end         : Timestamp
  status      : "booked" | "cancelled" | "completed")

(defstruct reservation-system
  resources    : {String -> Resource}
  customers    : {String -> Customer}
  reservations : {Nat -> Reservation})
```

## Resolver Natives

```innate
@reservation-system{}                                             -> ReservationSystem
@rs/add-resource{resource: Resource}                              -> ReservationSystem
@rs/book{resource-id, customer-id, start, end}                    -> Reservation
@rs/cancel{reservation-id}                                        -> Reservation
@rs/conflicts{resource-id, start, end}                            -> [Reservation]
@rs/available{start, end, kind?}                                  -> [Resource]
@rs/occupancy{resource-id}                                        -> [Reservation]
@rs/customer-bookings{customer-id}                                -> [Reservation]
```

## Demo

```innate
(@demo){
  @sys <- @reservation-system{}
    .add-resource{id: "12A", kind: "seat",  label: "window"}
    .add-resource{id: "12B", kind: "seat",  label: "aisle"}
    .add-resource{id: "304", kind: "room",  label: "king"}
    .add-customer{id: "C001", name: "Nathan"}

  @r1 <- @sys/book{resource-id: "12A", customer-id: "C001", start: @t, end: @t + 3h}
  ;; overlap attempt:
  @err <- @sys/book{resource-id: "12A", customer-id: "C001", start: @t + 1h, end: @t + 4h}
  ;; => error: conflict with @r1

  @free <- @sys/available{start: @t, end: @t + 3h, kind: "seat"}
  ;; => [{id: "12B", ...}]
}
```

## Where

The `book` operation MUST refuse overlapping intervals on the same resource. The half-open `[start, end)` convention MUST be used — touching boundaries do not conflict. `available` MUST return the negation of the conflict set, not a derived counter. Those invariants are what makes a scheduler a scheduler and not a best-effort hint.
