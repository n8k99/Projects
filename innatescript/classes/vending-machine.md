# G062 — Vending Machine

> The Rosetta Stone's first explicit **finite state machine**. Operations are gated by the machine's *state* rather than its data. Change-making is greedy but constrained by the machine's current coin stock.

```yaml
id: G062
title: Vending Machine
category: classes
requires: [G007-change-return, G048-product-inventory, G061-flower-shop]
provides: [finite-state-machine, state-gated-operations, coin-constrained-change, atomic-multi-entity-mutation]
```

## Insight: Operations Gated by State, Not (Only) by Data

Every prior Classes project had operations gated by **data invariants**: sufficient stock, valid triangle, correct shape. G062 introduces a qualitatively new gate: the machine's **state** (idle vs accepting). Some operations are only legal in certain states:

| Operation | Idle | Accepting |
|---|---|---|
| `insert_coin` | ✅ (transitions to Accepting) | ✅ |
| `select` | ❌ (no coins) | ✅ |
| `refund` | — (nothing to refund) | ✅ (returns to Idle) |

`select` in Idle state is not just "insufficient funds" — it's the WRONG OPERATION for the current state. The machine has no coins; asking it to dispense anything is a category error. This is the distinction: data gates say "the numbers don't add up"; state gates say "the system isn't in a posture to do that at all."

This is the noosphere's model for any multi-turn interaction. A choreography in `planning` state accepts `commit_plan`; in `executing` state it accepts `abort` but not `commit_plan`; in `completed` state it accepts nothing mutating. A conversation in `awaiting_reply` state cannot accept another prompt. Every stateful protocol in the vault is an FSM, and G062 introduces the primitive.

## Insight: Transitions Are First-Class Events

The state itself changes over time: `Idle → Accepting → Idle`. Each transition is tied to a specific operation:

- `insert_coin` triggers `Idle → Accepting` on the first coin.
- `select` (on success) triggers `Accepting → Idle`.
- `refund` triggers `Accepting → Idle`.
- `select` (on failure) triggers **no transition** — state stays in Accepting so the caller can insert more coins or refund.

That last point matters. Some failures preserve state; others would reset it. G062's design: data-shortage failures (insufficient funds, sold out, cannot-make-change) leave the user in Accepting with their coins intact. They failed to pick the right thing, but the machine doesn't punish them by dropping their coins. The only way out of Accepting is a valid purchase or an explicit refund.

This is a choice with noosphere-wide implications. A choreography that encounters "insufficient data" at a `where` clause might stall (preserve state) or abort (transition to cancelled); the decision depends on whether the user can provide more data or must restart. FSM design requires being explicit about each failure's transition.

## Insight: Change-Making Reuses G007 But With a New Constraint

G007 Change Return computed greedy change assuming an **unlimited** coin supply. G062 constrains the supply to the machine's current inventory (plus the coins just inserted). If the machine needs to return 15¢ but holds only quarters, no greedy solution exists — and no non-greedy one either, for standard US denominations.

This makes change-making a **constraint satisfaction** problem rather than a pure arithmetic one. The algorithm tries high-denom first, but capped at the available count. If a denomination runs out mid-computation, the algorithm continues with smaller denoms. If the remaining amount can't be reached at all, the purchase fails — and the coins are refunded, not pocketed.

The demo's second machine illustrates: no small change loaded, user inserts $1.00 to buy 30¢ gum, machine needs to return 70¢ but has only a single dollar coin in inventory (the one just inserted). No valid change exists. Purchase fails. Refund returns the $1.00.

This is the Rosetta Stone's first **capacity-constrained algorithm**. Greedy-with-caps appears throughout the noosphere — token budget for an LLM call (use the fewest tokens, up to the limit), agent assignment under load (prefer least-loaded agents, capped at their capacity), disk space allocation (largest free block first, up to available space). G062 presents the pattern minimally.

## Insight: Purchase Is a Multi-Entity Atomic Commit with Bidirectional Flow

A successful `select` touches multiple pieces of state simultaneously:

1. **Slot inventory** — product quantity decrements.
2. **Coin inventory** — change coins leave the machine (decrement); inserted coins become part of inventory (add to the pool at commit time).
3. **Inserted coins / cents** — reset to zero.
4. **State** — transitions from Accepting back to Idle.

Four coordinated updates. Either all land or none do. Same pattern as G052's transfer and G061's checkout, but with a new twist: **bidirectional flow** within the same entity (coin inventory). Some coins leave (change); some coins arrive (payment). The net effect depends on the specific coins and the specific change. This is more complex than inventory-only decrement: the inventory's *composition* changes, not just its totals.

This generalizes to any exchange system in the noosphere. A trade (forex) exchanges one currency for another at a specific rate — both balances change directions. A swap in a choreography (pass work from one agent to another) is simultaneously a decrement on one agent's queue and an increment on another's. G062 shows the primitive with the clearest possible domain.

## Insight: The Inserted-Coin Pool Joins the Machine's Pool at Commit, Not Before

A subtle but important design choice: inserted coins are **not** part of the machine's change-making pool until the purchase is committed. During Accepting, they sit in a separate `inserted_coins` table.

This matters for the failure case: if the purchase fails and coins are refunded, the refund returns the *exact* coins that were inserted — quarters for quarters, dollars for dollars. If inserted coins were merged into inventory immediately, the refund could only return coins from the general pool, which might not match what was inserted.

Keeping them separate until commit is the first instance of **two-phase settlement** in the Rosetta Stone. Phase 1: insert coins into a holding area. Phase 2: commit (merge into main pool) or refund (return from holding). This is how real payment systems work — holds on credit cards, escrow accounts, pending transactions.

## Choreographic Case: Automated Snack Restocking

```innate
(@restock-cycle){
  @machines <- @fleet/all
  @for each m in @machines {
    @inventory <- @m/inventory-snapshot
    @low-slots <- @inventory.slots.filter(.quantity < 2)
    @low-change <- @inventory.coins.filter((denom, count) => count < 10)

    @order <- @warehouse/prepare-delivery{
      products: @low-slots.map(.slot-id),
      coins: @low-change.keys
    }
    where {
      m_in_idle:         @m.state == idle      ;; must be safely restockable
      no_pending_coins:  @m.inserted-cents == 0
    }
  }
}
```

Restocking is only legal when the machine is in Idle state with no pending user coins — otherwise you'd interfere with a customer mid-transaction. The `where` enforces the FSM precondition.

## Structures

```innate
(defstruct vending-slot
  slot-id      : String
  product      : String
  price-cents  : Int
  quantity     : Int)

(defstruct vending-machine
  slots            : {String -> VendingSlot}
  coin-inventory   : {Int -> Int}        ;; denom → count
  inserted-coins   : {Int -> Int}
  inserted-cents   : Int
  state            : "idle" | "accepting")
```

## Resolver Natives

```innate
@vending-machine/new                                     -> VendingMachine
@vending-machine/add-slot{slot}                          -> VendingMachine
@vending-machine/load-change{denom, count}               -> VendingMachine
@vending-machine/insert-coin{denom}                      -> VendingMachine
@vending-machine/select{slot-id}                         -> (Product, Change)
@vending-machine/refund                                  -> Change        ;; always legal
```

## Demo

```innate
(@demo){
  @m <- @vending-machine/new
    .add-slot{slot-id: "A1", product: "Cola", price-cents: 125, quantity: 5}
    .load-change{denom: 25, count: 20}
    .load-change{denom: 10, count: 20}

  ;; Idle state: select refused.
  @m/select{slot-id: "A1"}    ;; -> error: cannot select while idle

  @m <- @m/insert-coin{denom: 100}
  @m <- @m/insert-coin{denom: 25}
  @m <- @m/insert-coin{denom: 25}       ;; total 150c, state: accepting
  @result <- @m/select{slot-id: "A1"}   ;; -> (Cola, {25: 1})
  ;; state now back to idle, coin inventory updated
}
```

## Where

State gates MUST be checked before data gates — `select` in Idle state MUST fail with a state error, not an insufficient-funds error. Inserted coins MUST be held separate from the machine's change pool until commit. Change-making MUST respect current coin inventory (no phantom coins). Failure on cannot-make-change MUST not consume the inserted coins. Every transition MUST be triggered by a specific operation and logged in code, not left implicit.
