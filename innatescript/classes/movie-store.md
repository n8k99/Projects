# G049 — Movie Store

> Two entities joined by a rental. Due dates introduce the first temporal obligation — a `where` that fires automatically when time passes.

```yaml
id: G049
title: Movie Store
category: classes
requires: [G048-product-inventory]
provides: [entity-relations, join-table, temporal-obligation, availability-aggregate]
```

## Insight: The Join Table Is the Conversation

G048 modeled a single entity with state. G049 introduces **two entities connected by a third**. Movies and Customers are independent — neither references the other. The `Rental` is the join record: when a customer borrows a movie, the interaction is captured as a new entity with its own identity and lifecycle.

This is how the noosphere works. `conversations` joins `user` and `project`. `tasks` joins `agent` and `goal`. `annotations` joins `ghost` and `article`. Every time two entities interact, a third record captures the interaction — and that third record is where the temporal dimension lives. Movies don't have due dates. Customers don't have due dates. The *rental* has a due date, because obligations live on interactions, not on the things being interacted with.

## Insight: Availability Is an Aggregate, Not a Counter

`copies_total` is stored. `copies_available` is computed: `total − count(active rentals)`. Same pattern as G048's `quantity_on_hand`, now with a filter on an auxiliary entity. The resolver query is: *fold over rentals where movie_id matches and returned_at is null; subtract from copies_total*.

This generalizes: every availability question in the noosphere is `capacity − active_obligations`. An agent's availability = allocated tokens − active choreographies. A project's bandwidth = weekly hours − active commitments. A timeline slot's freedom = calendar capacity − scheduled events. The movie store's formula scales to every capacity/demand situation in the vault.

## Insight: Due Dates Are the First Temporal Obligation

A rental with `due_at` creates a **pending obligation** until the movie is returned. Between `rented_at` and `due_at`, the obligation is healthy. After `due_at` with no return, the obligation is overdue. This is the first temporal `where` in the Rosetta Stone: a condition that flips from pass to fail as time passes, even though no one touched the system.

InnateScript's `until` now has a concrete model. `@rental until @returned or @due_at` — the obligation is bounded by *either* fulfillment or deadline. This is the shape of every scheduled check in the noosphere: the nightly summary has a `due_at` of midnight, the forex pace check has a `due_at` of month-end, the Rosetta Stone goal has a `due_at` of the milestone completion date. Pass or overdue, scored automatically, no one "calls" the check.

## Insight: The `overdue` List Is a Passive Observer

`store.overdue(now)` doesn't mutate anything. It doesn't send emails, charge fees, or lock accounts. It just *reports the set of obligations that have crossed their deadlines*. The reaction — late fees, notifications, re-assignment — is a separate choreography that takes the overdue list as input.

This separation is important. The data layer answers "what is true right now"; the choreography layer decides "what to do about it". In InnateScript terms, `@store/overdue` is a pure query. A cron-triggered choreography reads that query, scores a `where`, and spawns downstream actions. The store is oblivious to the consequences of overdue rentals. That's the correct shape of a state model.

## Choreographic Case: Nightly Overdue Sweep

```innate
(@nightly-overdue-sweep){
  @store     <- @movie-store/load
  @overdue   <- @store/overdue{now: @now}
  @by_customer <- @overdue.group_by(.customer_id)

  concurrent {
    @for each (cust, rentals) in @by_customer {
      @sarah/notify{customer: cust, rentals: @rentals, severity: "late"}
    }
    @kathryn/assess_fees{overdue: @overdue}
  } join as @summary

  where {
    all_customers_notified: @by_customer.every(.notified == true)
    fees_calculated:        @summary.total_fees >= 0
  }
}
```

The sweep runs every night. It groups overdue rentals by customer, notifies each, calculates fees. The `where` scores the sweep's success. If no rentals are overdue, all the aggregations trivially pass — the choreography exits clean.

## Structures

```innate
(defstruct movie
  id            : String
  title         : String
  director      : String
  year          : Nat
  genre         : String
  copies-total  : Nat)

(defstruct customer
  id   : String
  name : String)

(defstruct rental
  id          : Nat
  movie-id    : String
  customer-id : String
  rented-at   : Timestamp
  due-at      : Timestamp
  returned-at : Timestamp?)

(defstruct movie-store
  movies    : {String -> Movie}
  customers : {String -> Customer}
  rentals   : {Nat -> Rental})
```

## Resolver Natives

```innate
@movie-store{}                                               -> MovieStore
@store/add-movie{movie: Movie}                               -> MovieStore
@store/add-customer{customer: Customer}                      -> MovieStore
@store/rent{movie-id, customer-id, duration-days}            -> Rental
@store/return{rental-id}                                     -> Rental
@store/copies-available{movie-id}                            -> Nat
@store/active-rentals{customer-id}                           -> [Rental]
@store/overdue{now: Timestamp}                               -> [Rental]
@store/movie-history{movie-id}                               -> [Rental]
@store/customer-history{customer-id}                         -> [Rental]
```

## Demo

```innate
(@demo){
  @store <- @movie-store{}
    .add-movie{movie: {id: "M001", title: "Casablanca", copies-total: 2}}
    .add-customer{customer: {id: "C001", name: "Nathan"}}
  @rental <- @store/rent{movie-id: "M001", customer-id: "C001", duration-days: 7}
  @avail  <- @store/copies-available{movie-id: "M001"}
  ;; => 1
  @late   <- @store/overdue{now: @rental.due-at + 86400}
  ;; => [@rental]
}
```

## Where

`copies_available` MUST be computed from active rentals; never stored. Due dates MUST live on the rental, not on the movie or customer. The `overdue` query MUST be pure — it reports, it does not react. Those three invariants are the shape of every relational state model in the noosphere.
