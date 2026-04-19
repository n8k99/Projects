# G061 — Flower Shop Ordering

> The Rosetta Stone's first **build-up/commit** transaction. A cart accumulates over time, reviews its own state, and commits atomically at checkout. Prices are frozen at add-time. Orders are reversible.

```yaml
id: G061
title: Flower Shop Ordering
category: classes
requires: [G048-product-inventory, G052-bank-account, G055-recipe-manager]
provides: [build-up-commit, frozen-snapshot, discount-rule-composition, reversal-as-inverse-operation]
```

## Insight: The Cart Is the First Intentionally Mutable Primary Entity

Every prior entity was either immutable (value classes like BigInt, Recipe) or mutated only through atomic commits (inventory movements, appointment reschedules). G061's Cart is **designed** to evolve: add flowers, remove, change quantities, apply discounts. A single cart may be modified dozens of times before anyone commits to an order.

This is the first project where the class's design point is *ongoing mutation as the primary operation*. Previous classes treated mutation as the rare operation (commit, complete, cancel); G061 treats it as the default. Carts are conversations with yourself: "I want roses — actually, make it six — plus two tulips — hmm, scratch the tulips, add lilies."

In the noosphere, this maps directly onto *drafts*: a daily note being written, a project being scoped, a choreography being designed before being executed. Every "draft" state in the vault is a cart pattern — mutable while you're working, committed only when you're done.

## Insight: Build-Up/Commit Separates Editing From Committing

The lifecycle has three distinct stages:

1. **Draft** (Cart) — mutable; any add/remove/update edits the cart in place.
2. **Placed** (Order) — committed; inventory decrement is applied, cart status flips.
3. **Cancelled/Fulfilled** — terminal; cancellation reverses the commit, fulfillment confirms it.

Only the *transition* from draft to placed is atomic. Everything within draft is freely mutable; nothing within placed or terminal states can be modified. This three-phase lifecycle (edit → commit → finalize) is the Rosetta Stone's first truly staged transaction.

The vault's Daily Note works this way: you edit freely throughout the day (draft), the nightly summary at midnight is the commit (placed), and the following day's rollup archives it (fulfilled). A conversation has the same shape: messages accumulate (draft), the reply is committed (placed), and it's either answered (fulfilled) or retracted (cancelled). G061 makes the lifecycle explicit; the pattern recurs everywhere.

## Insight: Prices Are Frozen at Add-Time

```
add_to_cart(cart, "ROSE", 6)   # captures current price: $4.50
# ... time passes ...
flower["ROSE"].unit_price = 6.00   # price changes
checkout(cart)                  # still charges $4.50
```

The cart line carries the price it had when added, not whatever the flower costs at checkout. This invariant is load-bearing. If the price tracked the catalog live, carts would become unpredictable: a customer might add an item at one price and be charged another, with no warning. The freeze is a promise: "you agreed to this price; we'll honor it."

The demo verifies this: the catalog price of ROSE changes to $6.00 after the cart is built, but the cart's line remains at $4.50. The checkout receipt reflects the frozen price.

This generalizes throughout the noosphere. When a project references another project, does it reference the *current* state or the *state at the time of the reference*? When a choreography quotes an upstream definition, does updating that definition retroactively rewrite history? G061 answers for its domain: the cart line *snapshots* the price. The vault will make analogous choices everywhere — some references are live (`@projects.get_current`), some are frozen (`@conversation.original_message`). G061 introduces the explicit snapshot pattern.

## Insight: Discount Rules Compose

A cart may have multiple discount rules, each with its own applicability check. 10% off orders over $30. Free shipping over $40. $5 off for loyalty members. All three can apply to the same cart. The total discount is the sum of each rule's contribution:

```
total_discount = Σ rule.discount_cents(subtotal, shipping) over all rules
```

Some rules are **conditional on subtotal** (the 10%-off threshold). Some are **additive** (10% + $5 off). Some are **scoped to a specific cost** (free shipping affects shipping_cents, not subtotal). Despite the variety, they all conform to one interface: `(subtotal, shipping) → discount_cents`. This is the closed-polymorphism version of G059's open interface — a fixed small set of rule types, each with its own dispatch logic.

The noosphere will compose rules this way constantly. Pace-check strategies compose: daily + weekly + monthly pace scores contribute to an overall score. Agent routing rules compose: priority-based, capability-based, timezone-based filters all apply. Permission checks compose: user, role, project, and resource policies all factor in. G061's discount composition is the canonical teaching example.

## Insight: Checkout Is a Multi-Entity Atomic Commit

Checkout touches multiple pieces of state in one logical operation:

- Every flower in the cart has its stock decremented.
- An Order record is created with snapshot copies of lines, rules, and totals.
- The Cart's status flips from DRAFT to PLACED.

Either all of this happens or none does. If any flower lacks stock at checkout time (inventory changed since add-to-cart), the *entire* checkout fails before any decrement. This is G052's atomic-transaction pattern applied outside finance: the inventory and order tables must stay consistent, so the commit is all-or-nothing.

A pre-flight pass verifies all shortages before any mutation. This matches the noosphere's general pattern for multi-entity updates: check everything first (read phase), then write everything (commit phase), with a failure in read aborting before any write. The vault's batch operations (moving all tasks from a phase, archiving a project) should follow the same two-phase discipline.

## Insight: Cancellation Is Reversal — an Inverse Operation

Cancelling a placed order is not just "mark it cancelled." It must **restore** the inventory that was decremented at checkout. For each line, `stock += line.quantity`. This is the Rosetta Stone's first explicit *inverse operation*: the cancellation applies the reverse of what commit did.

This differs from earlier "cancel" operations:

- G050's `cancel` on a reservation just marked status — the interval was future, nothing needed unwinding.
- G049's `return` on a rental marked the loan returned — a separate lifecycle event.
- G061's `cancel_order` undoes a completed commit — inventory state is explicitly reversed.

The distinction: G050 cancelled a *promise*, G049 ended a *duration*, G061 reverses a *fact*. When state has been committed, cancellation is inverse-operation, not status-flip.

This generalizes. Cancelling a money transfer (G052) would require a reverse-transfer with the entries swapped. Rolling back a choreography would require applying each step's inverse. Every commit needs a defined inverse if the system wants to support reversal. G061 is the first place the Rosetta Stone makes this explicit.

**Not every commit has a clean inverse.** Sending an email has no true inverse. Publishing a post has no true inverse. Notifying a user has no true inverse. When a system advertises "cancel/undo," it's making a claim about which operations have inverses — and the design must enforce that claim. G061's inventory model is cleanly reversible because stock decrements are commutative with increments. Many real-world operations aren't, and the cart/order lifecycle quietly restricts cancellation to `PLACED` states to avoid pretending it can reverse a shipped order.

## Choreographic Case: Weekly Flower Subscription

```innate
(@weekly-subscription){
  @subscriptions <- @db/active-subscriptions{day: @today}

  concurrent {
    @for each sub in @subscriptions {
      @cart <- @shop/new-cart{owner: @sub.customer-id, shipping: @sub.address.ship_cost}
      @sub.items.each(item => @shop/add-to-cart{cart: @cart.id, sku: item.sku, qty: item.qty})
      @shop/apply-discount{cart: @cart.id, rule: @loyalty-discount{customer: @sub.customer-id}}
      @order <- @shop/checkout{cart: @cart.id}
      @notifications/email{to: @sub.customer, receipt: @shop/receipt{order: @order}}
    }
  } join as @results

  where {
    all_orders_placed:    @results.every(.order.status == placed)
    inventory_consistent: @shop/ledger-consistent
    emails_sent:          @results.every(.email.status == sent)
  }
}
```

One subscription → one cart → one checkout → one notification. The cart is the staging area; the commit is atomic; any failed cart rolls back cleanly without affecting others.

## Structures

```innate
(defstruct flower
  sku              : String
  name             : String
  unit-price-cents : Int
  stock            : Int)

(defstruct cart-line
  sku              : String
  quantity         : Int
  unit-price-cents : Int)               ;; FROZEN snapshot

(defstruct discount-rule
  kind                : "percentage-off" | "fixed-off" | "free-shipping"
  value               : Int
  min-subtotal-cents  : Int)

(defstruct cart
  id                : String
  owner-id          : String
  lines             : [CartLine]
  discount-rules    : [DiscountRule]
  shipping-cents    : Int
  tax-rate-bps      : Int
  status            : "draft" | "placed" | "fulfilled" | "cancelled")

(defstruct order
  id, owner-id, lines, discount-rules, shipping-cents, tax-rate-bps, status)
```

## Resolver Natives

```innate
@shop/new-cart{owner, shipping?, tax-rate-bps?}           -> Cart
@shop/add-to-cart{cart-id, sku, qty}                      -> Cart
@shop/update-quantity{cart-id, sku, new-qty}              -> Cart
@shop/apply-discount{cart-id, rule}                       -> Cart
@shop/subtotal{cart-or-order}                             -> Int
@shop/total-discount{cart-or-order}                       -> Int
@shop/tax{cart-or-order}                                  -> Int
@shop/total{cart-or-order}                                -> Int
@shop/checkout{cart-id}                                   -> Order     ;; atomic commit
@shop/cancel-order{order-id}                              -> Order     ;; reversal
@shop/fulfill-order{order-id}                             -> Order
```

## Demo

```innate
(@demo){
  @shop <- @shop{}
    .add-flower{sku: "ROSE", unit-price-cents: 450}
    .stock{sku: "ROSE", quantity: 50}

  @cart <- @shop/new-cart{owner: "P-001", shipping: 500, tax-rate-bps: 825}
  @shop/add-to-cart{cart-id: @cart.id, sku: "ROSE", qty: 6}
  ;; change price AFTER add — cart is unaffected
  @shop/flowers{sku: "ROSE"}.unit-price-cents = 600
  @order <- @shop/checkout{cart-id: @cart.id}   ;; charges 6 × $4.50 = $27.00
  @shop/cancel-order{order-id: @order.id}       ;; stock restores to 50
}
```

## Where

The cart MUST be mutable while in DRAFT status; any mutation after transition MUST fail. Prices MUST be frozen at add-time into the cart line. Checkout MUST do pre-flight inventory check across all lines before any decrement (all-or-nothing). Discount rules MUST be applied as a sum of independent applications, not in sequence (no rule affects another's subtotal). Cancellation MUST reverse inventory decrements — the order's inverse operation. Those five rules define a shop; without them, you have a disguised spreadsheet.
