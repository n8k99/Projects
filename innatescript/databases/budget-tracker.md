# G107 — Budget Tracker

> The Rosetta Stone's seventh **Databases project**. Introduces **envelope budgeting** — fixed monthly allocations per category, with variance tracking (actual vs budgeted) and alert thresholds that fire when spending crosses a configurable percentage of the envelope. The pattern every budgeting app (YNAB, Mint, Buckets, spreadsheet budgets) ships. Distinctive move: **uncategorised spending surfaces as a synthetic envelope with budget=0**, so "where did the money actually go?" questions are answerable even when the user hasn't created categories for everything.

```yaml
id: G107
title: Budget Tracker
category: databases
requires: [G008-average, G089-transaction-averages, G094-log-file-maker, G106-event-scheduler]
provides: [envelope-allocation, variance-analysis, alert-thresholds, uncategorised-synthetic-envelope]
```

## Insight: Envelope Budgeting Is Category → Allocation

A budget isn't one number — it's a **set of envelopes**, each with its own budgeted amount. Groceries: $500. Rent: $2000. Entertainment: $100. Each envelope tracks its own spending independently; overrunning groceries doesn't directly affect rent's remaining balance.

This is the YNAB model, the cash-envelope method, the 50/30/20 rule. G107 formalises it: `Envelope { category, budgeted_cents, rollover, alert_threshold_pct }`. The budget aggregates envelopes and a transaction stream.

First Rosetta Stone project where **a "budget" is structured per-category, not a single scalar**. G089's transaction averages were aggregations over categories; G107 is the first where categories are also a *specification* (budgets) not just a *grouping*.

## Insight: Variance = Actual − Budgeted

Variance is the signed difference. Positive = overspent. Negative = underspent. Variance percentage is `variance * 100 / budgeted`. The sign convention matches accounting: positive variance is bad news for expenses.

Zero-budget envelopes get a sentinel variance_pct of -101 (anything outside [-100, ∞) works). The UI renders this as "∞" or "no budget set" — any percentage over a zero base is undefined, and the sentinel surfaces that explicitly rather than silently returning 0 or crashing.

First Rosetta Stone project where **a sentinel value represents "undefined"** in a numeric field. G094's logs used Level::Error for undefined; G107 uses -101 because it's a percentage — inside the numeric type but outside the valid range.

## Insight: Alert Thresholds Scale With Envelope Size

Alerts aren't a fixed-dollar line — they're a **percentage** of the envelope. Rent at 90% ($1800/$2000) alerts. Entertainment at 90% ($90/$100) also alerts, even though the dollar amounts differ by 20x. The user's attention scales with their exposure; thresholds scale with budget.

Each envelope has its own threshold (default 80). A strict envelope (entertainment) alerts early; a predictable envelope (rent) alerts late. The policy is per-envelope, not per-budget.

First Rosetta Stone project where **a scale-invariant threshold replaces an absolute one**. G086's frecency was scale-invariant (logarithm); G107's alert threshold is also scale-invariant (percentage). Different math, same goal: surfaces what's proportionally surprising.

## Insight: Refunds Subtract From Spent, Not Add

A refund is a positive amount_cents. When computing `spent`, a refund **reduces** the category's running total: `spent = -expense + refund_amount`, which simplifies to `-amount_cents` for both cases when expense is negative and refund is positive. Clean, symmetric.

Why not model refunds as separate transactions? Because accounting-wise they offset the original purchase. A $50 refund against a $100 purchase yields $50 spent, not $50 + $(-50) = $0 spent with no record. The transaction history preserves both; the envelope's running total correctly reflects net.

First Rosetta Stone project where **a sign flip carries semantic meaning** (positive = money in, negative = money out). G089 had integer cents but all transactions were expenses. G107 handles both directions.

## Insight: Uncategorised Spending Surfaces As a Synthetic Envelope

Real data has categories the user never budgeted for. Vacation, emergency repairs, one-time gifts. G107's status computation walks the transaction stream, groups by category, and emits a status row for every category — including ones that match no envelope.

Those synthetic envelopes have `budgeted_cents = 0`, `over_budget = true` (any spending is over), `variance_pct = -101` (sentinel for "no budget"). The UI renders them as "?" categories the user should either assign a real envelope to, or accept as overflow.

First Rosetta Stone project where **the output includes entries not present in the input schema**. G094 synthesised group keys from transaction data. G107 synthesises *envelopes* from transaction data when the user's schema didn't cover the ground truth.

## Choreographic Case: Vault Monthly Budget

```innate
(@vault-monthly-budget){
  @budget <- @budget/from-yaml{path: "budgets/2026-04.yaml"}
  @month-start <- @clock/start-of-month{year: 2026, month: 4}
  @month-end   <- @clock/start-of-month{year: 2026, month: 5}

  @status <- @budget/status-between{budget: @budget,
                                      from-ms: @month-start, to-ms: @month-end}
  @ui/render-envelopes{status: @status}

  @alerts <- @status.filter(.alert)
  @when (@alerts.length > 0){
    @notify/send{channels: ["email"],
                 message: "Budget alerts: ${@alerts.map(.category).join(', ')}"}
  }

  @uncategorised <- @status.filter(.budgeted-cents == 0 && .spent-cents > 0)
  @when (@uncategorised.length > 0){
    @ui/suggest-envelopes{candidates: @uncategorised}
  }
}
```

The vault's budget pane loads monthly envelopes, computes status, fires alerts, and nudges the user to categorise stray spending. All logic is inside the budget engine; the UI is a renderer.

## Structures

```innate
(defenum rollover-policy RESET | CARRY)

(defstruct envelope
  category              : String
  budgeted-cents        : Int
  rollover              : RolloverPolicy
  alert-threshold-pct   : Int)

(defstruct transaction
  timestamp-ms  : Int
  category      : String
  amount-cents  : Int          ;; negative = expense, positive = refund
  note          : String)

(defstruct category-status
  category        : String
  budgeted-cents  : Int
  spent-cents     : Int
  remaining-cents : Int
  variance-cents  : Int
  variance-pct    : Int         ;; -101 sentinel = no budget
  over-budget     : Bool
  alert           : Bool)
```

## Resolver Natives

```innate
@budget/new{}                                          -> Budget
@budget/add-envelope{budget, envelope}                 -> Unit
@budget/record{budget, transaction}                    -> Unit
@budget/status-between{budget, from-ms, to-ms}         -> [CategoryStatus]
@budget/status-all{budget}                             -> [CategoryStatus]
@budget/alerts{budget}                                 -> [CategoryStatus]
@budget/total-budgeted{budget}                         -> Int
@budget/total-spent-between{budget, from-ms, to-ms}    -> Int
```

## Demo

```innate
(@demo){
  @b <- @budget/new{}
  @budget/add-envelope{budget: @b, envelope: {category: "groceries",
                                                budgeted-cents: 50000}}
  @budget/add-envelope{budget: @b, envelope: {category: "rent",
                                                budgeted-cents: 200000,
                                                alert-threshold-pct: 90}}
  @budget/record{budget: @b, transaction: {timestamp-ms: 100,
                                              category: "groceries",
                                              amount-cents: -30000}}
  @budget/record{budget: @b, transaction: {timestamp-ms: 200,
                                              category: "vacation",
                                              amount-cents: -5000}}
  @budget/alerts{budget: @b}
  ;; -> []  (groceries at 60%, < 80% default threshold)
  @budget/status-all{budget: @b}
  ;; Includes synthetic "vacation" envelope with budget=0, over_budget=true.
}
```

## Where

Envelopes MUST be per-category, NOT a single pool — the whole point of envelope budgeting is that one category can't quietly eat another's allocation. Variance MUST be computed `spent - budgeted` (positive = overspent) — match accounting convention; any other sign flip confuses users coming from every real budgeting app. Zero-budget envelopes MUST return a sentinel variance_pct (-101), NOT zero or crash — undefined ratios are a distinct user-visible state. Alert thresholds MUST be percentages of the envelope, NOT fixed dollar amounts — scale-invariance is the whole reason thresholds exist. Refunds MUST reduce the spent total, NOT appear as separate categories — accounting reconciliation depends on net. Uncategorised spending MUST appear as synthetic envelopes with budget=0, NOT be silently hidden — "where did my money go?" is the primary question budgeting answers, and hiding unmatched spending breaks that answer.
