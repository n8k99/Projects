# G052 — Bank Account Manager

> Double-entry ledger. Transactions are atomic multi-entry operations whose entries sum to zero (conservation). Balance is the projection of the ledger onto one account.

```yaml
id: G052
title: Bank Account Manager
category: classes
requires: [G048-product-inventory, G007-change-return]
provides: [double-entry-ledger, atomic-transaction, conservation-invariant, account-lifecycle]
```

## Insight: The First Multi-Sided Event

G048's inventory movement touched one product. G049's rental involved one movie and one customer but *tracked* one entity (the rental itself). G051's grade touched one student and one assignment with one value. Every prior project had **single-sided events** — one record per operation.

G052 introduces the double-entry transaction: one operation, *multiple* entries, all landing atomically. A transfer of $100 from A to B is two entries — `(A, -10000)` and `(B, +10000)` — that MUST be written together or not at all. Deposits and withdrawals are the single-entry degenerate case where money crosses the bank's boundary.

This is the shape of every coordinated state change. A git commit is a multi-entry transaction: several files modified, all or none. A choreography step that updates three vault records must land all three atomically — otherwise a later reader sees a partial state. G052 is the Rosetta Stone naming the pattern the noosphere has been using implicitly.

## Insight: Conservation as an Invariant

Transfer entries sum to zero. Money is conserved *within* the bank. Deposits and withdrawals break conservation locally because they cross the system boundary — but internally, every redistribution is zero-sum.

```
∀ tx where tx.kind = transfer: Σ(entries.amount) = 0
```

This is a global invariant on the ledger, checkable at any time (`ledgerIsBalanced` in every implementation). If it ever fails, the ledger is corrupted — a reconstruction bug, a race condition, a missing entry. The invariant is a detection mechanism, not just a rule.

Generalizes throughout the noosphere. When a choreography reassigns work from one agent to another, the total assigned work is conserved. When a project moves a task from backlog to active, the total task count is conserved. Any time state redistributes *within* a closed system, the redistribution must sum to zero. The Bank is the canonical example because money makes the conservation explicit, but the pattern is everywhere.

## Insight: Balance Is Still an Aggregate

G048's `quantity_on_hand` was the signed sum of movements. G052's `balance` is the signed sum of entries touching an account. Same pattern, different primary key. The event log is canonical; the "current value" is a projection.

What makes G052 stronger than G048: a single transaction contributes to *multiple* accounts simultaneously. A transfer adds a `-100` entry to A's projection and a `+100` entry to B's projection. The same log powers both queries. This is how a database view works — one source of truth, many derived tables.

In InnateScript: every `@account/balance{id}` is a fold over the global transaction stream, filtered to that account. Cache the fold if you want; recompute from canonical if you need certainty.

## Insight: The Account Lifecycle Has a Precondition for Closure

An account can't just be closed — it must have zero balance first. This is the first project where a **lifecycle transition has a numeric precondition**. Previous projects had categorical preconditions (movie in stock, slot not conflicting). This one requires the aggregate to hit a specific value before the transition is legal.

```innate
@account/close{id} <- {balance == 0}
```

The `where` gate on closure is an equality, not an inequality. That's rarer and more interesting. It shows up elsewhere in the noosphere: you can't archive a project with open tasks, you can't close a conversation with unresolved references, you can't retire a schema with active records. "Zero balance" is the general name for "this entity has no unresolved obligations."

## Insight: Frozen Is a Meta-Policy, Not a Data Check

G048's `sell` inline-checked sufficient stock. G052's `deposit` and `withdraw` check a *separate* predicate first: `status == open`. A frozen account has funds, but transactions against it are refused *independent of the amounts involved*. This is the first project with a policy gate at the entity level rather than the operation level.

In the noosphere, this is the pattern for read-only modes, maintenance windows, compliance holds, and permission revocations. The account still exists, the balance is still correct, the history is intact — but operations against it are disabled. `frozen` is not a data state; it is a *gate on state transitions*. InnateScript needs this distinction: `where balance > 0` is a data condition; `where status == open` is a policy condition. Both can fail a `where`; they fail for different reasons and warrant different reactions.

## Choreographic Case: Monthly Sweep with Balanced Reconciliation

```innate
(@monthly-sweep){
  @month-start <- @first-of-month
  @all-tx      <- @bank/transactions-since{at: @month-start}

  concurrent {
    @kathryn/verify_conservation{transactions: @all-tx}
    @sarah/flag_overdrafts{transactions: @all-tx}
    @ellie/sum_fees{transactions: @all-tx}
  } join as @audit

  where {
    ledger_balanced:      @bank/ledger-balanced
    no_negative_balances: @bank/accounts.every(.balance >= 0)
    kathryn_approves:     @audit.kathryn == "confirmed"
  }
}
```

`@bank/ledger-balanced` is the global conservation check. `no_negative_balances` is a per-account check that individual account rules (checking can't go negative, say) held through the month. Kathryn approves. Three different invariants, one `where` clause. The sweep fails if any fires.

## Structures

```innate
(defstruct account
  id        : String
  holder-id : String
  kind      : "checking" | "savings" | "credit"
  status    : "open" | "frozen" | "closed")

(defstruct entry
  account-id    : String
  amount-cents  : Int)                    ; + credit, - debit

(defstruct transaction
  id      : Nat
  kind    : "deposit" | "withdrawal" | "transfer" | "fee" | "interest"
  entries : [Entry]
  memo    : String)

(defstruct bank
  customers    : {String -> Customer}
  accounts     : {String -> Account}
  transactions : {Nat -> Transaction})
```

## Resolver Natives

```innate
@bank{}                                                -> Bank
@bank/open-account{holder-id, kind}                    -> Account
@bank/close-account{id}                                -> Account     ;; balance must be 0
@bank/freeze{id}                                       -> Account
@bank/deposit{account-id, amount-cents}                -> Transaction
@bank/withdraw{account-id, amount-cents}               -> Transaction  ;; balance ≥ amount
@bank/transfer{from-id, to-id, amount-cents}           -> Transaction  ;; both active, sum to zero
@bank/balance{account-id}                              -> Int
@bank/statement{account-id, from?, to?}                -> [Transaction]
@bank/total-assets{customer-id}                        -> Int
@bank/ledger-balanced                                  -> Bool
```

## Demo

```innate
(@demo){
  @bank <- @bank{}
  @bank <- @bank/add-customer{id: "C001", name: "Nathan"}
  @chk  <- @bank/open-account{holder-id: "C001", kind: "checking"}
  @sav  <- @bank/open-account{holder-id: "C001", kind: "savings"}
  @bank/deposit{account-id: @chk.id, amount-cents: 50000}
  @bank/transfer{from-id: @chk.id, to-id: @sav.id, amount-cents: 20000}
  ;; @bank/balance{@chk.id} => 30000
  ;; @bank/balance{@sav.id} => 20000
  ;; @bank/ledger-balanced  => true
}
```

## Where

Transfer entries MUST sum to zero. Withdrawals MUST NOT exceed balance. Frozen and closed accounts MUST refuse new transactions. Closure MUST require zero balance. Those four invariants are what distinguishes a ledger from a spreadsheet.
