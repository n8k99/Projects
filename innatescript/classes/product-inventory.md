# G048 — Product Inventory

> Entities with identity that persist and mutate. Movements form a ledger; current quantity is an aggregate projection.

```yaml
id: G048
title: Product Inventory
category: classes
requires: [G007-change-return, G032-regex-query]
provides: [entity-state, event-ledger, reorder-threshold, inventory-valuation]
```

## Insight: The First Persistent Entity

Everything in the Numbers, Text, and Networking categories either computed a value and returned it, or processed a stream and moved on. Nothing *stayed*. A product is the first thing in this repo with **identity that persists across calls**. The SKU is not a parameter — it's a name. `SKU-001` on Monday is the same `SKU-001` on Friday, with different quantities, same identity.

This is the shape of every vault entity. A project has a slug, a task has an id, a conversation has a thread. They persist. Their fields mutate. The category is called `classes`, but what it's really modeling is `entities`.

## Insight: Quantity Is an Aggregate, Not a Field

`quantity_on_hand(sku)` is not stored. It is **summed from the movement ledger**. Current state is a projection over event history. This is event sourcing, and it is how the noosphere already works: `current_context` on a project is the latest aggregate over a stream of conversations; a daily note's `accomplishments` section is a projection over `T.A.S.K.S.` completions. The inventory makes the pattern explicit — the ledger is canonical, the field is a view.

The implication for a choreographic language: `@inventory/quantity{sku}` is not a field access. It is a resolver fold over a stream. Swap the stream for `@conversations{project: X}` and the same fold computes a project's message count.

## Insight: Reorder Points Are the `where` Threshold

A reorder point is a condition: *when on-hand drops to or below this number, fire*. The `restock_needed` list is the set of products whose `where` currently fails. This is Kathryn's `pace_check` applied to inventory — not "are we on track for month-end revenue," but "are we on track for next week's stockout." Same shape, different units.

Every `where` clause in InnateScript can be re-read as a reorder point on an aggregate. The inventory is the canonical example: the clause `quantity_on_hand > reorder_point` is what a thousand `where` blocks in the noosphere will eventually look like.

## Insight: Receive and Sell Are Preconditioned Transitions

`sell` fails if `on_hand < requested`. This is the first transition in the Rosetta Stone with a precondition that depends on aggregated state. The choreographic reading: `@inventory/sell{sku, qty}` is a step with a `<-` gate. The gate tests the aggregate. If it fails, the choreography halts. This is not error handling — it is structural. The transition is only legal when the precondition holds.

## Choreographic Case: Monthly Stock Audit

```innate
(@monthly-stock-audit){
  @inventory <- @inventory/load{as_of: @month-end}
  @needed   <- @inventory/restock-needed
  @valuation <- @inventory/total-value

  concurrent {
    @sarah/verify{skus: @needed.map(.sku)}
    @kathryn/allocate{cost: @needed.map(@cost-of-restock), against: @monthly-budget}
  } join as @plan

  where {
    all_low_stock_known: @needed.every(.flagged_to_sarah)
    budget_available:    @kathryn.approved == true
    valuation_reported:  @valuation > 0
  }
}
```

Sarah verifies the low-stock list is complete. Kathryn confirms the restock cost fits the monthly budget. The `where` scores whether the audit actually closes. The inventory provides the facts; the choreography coordinates the judgment around them.

## Structures

```innate
(defstruct product
  sku           : String
  name          : String
  price         : Float
  category      : String
  reorder-point : Nat)

(defstruct movement
  sku      : String
  kind     : "receive" | "sell"
  quantity : Nat
  note     : String)

(defstruct inventory
  products  : {String -> Product}
  movements : [Movement])
```

## Resolver Natives

```innate
@inventory{}                                         -> Inventory
@inventory/add-product{product: Product}             -> Inventory
@inventory/receive{sku: String, qty: Nat}            -> Inventory
@inventory/sell{sku: String, qty: Nat}               -> Inventory
@inventory/quantity{sku: String}                     -> Int
@inventory/history{sku: String}                      -> [Movement]
@inventory/total-value                               -> Float
@inventory/restock-needed                            -> [Product]
@inventory/by-category{category: String}             -> [Product]
```

## Demo

```innate
(@demo){
  @inv <- @inventory{}
    .add-product{product: {sku: "SKU-001", name: "Widget", price: 9.99, reorder-point: 5}}
    .add-product{product: {sku: "SKU-002", name: "Gadget", price: 24.50, reorder-point: 3}}
  @inv <- @inv/receive{sku: "SKU-001", qty: 50}
  @inv <- @inv/sell{sku: "SKU-001", qty: 48}
  @low <- @inv/restock-needed
  ;; => [{sku: "SKU-001", ...}]  (2 on hand ≤ reorder-point 5)
}
```

## Where

Each implementation is a callable library module. `quantity_on_hand` MUST be computed from the movement ledger — never stored as a mutable field. That invariant is the whole point of the project.
