# G110 — Travel Planner System

> The Rosetta Stone's tenth **Databases project**. Introduces **itinerary integrity validation** — a trip is an ordered sequence of legs that must chain correctly in space and time. The validator walks consecutive legs and surfaces structural issues: location discontinuity (`leg[N].destination ≠ leg[N+1].origin`), time conflict (leg N+1 departs before leg N arrives), tight layover (gap below minimum), zero/negative duration. This is the pattern every travel-booking, logistics, and supply-chain system ships — chain-of-edges validation.

```yaml
id: G110
title: Travel Planner System
category: databases
requires: [G088-sort-file-records, G106-event-scheduler, G109-tv-show-tracker]
provides: [itinerary-chain-validation, layover-enumeration, per-leg-issue-reporting, severity-laddered-issues]
```

## Insight: An Itinerary Is a Chain of Graph Edges

A leg is an edge in a transportation graph: `origin --(mode, depart_ms → arrive_ms)--> destination`. A trip is a walk along that graph — a sequence of edges where each edge's target equals the next edge's source.

G110's validator checks the chain composition property: **leg[N].destination == leg[N+1].origin**. Any mismatch is a `LocationMismatch` issue — someone's supposed to teleport. This is the same invariant `git log --graph` checks when parents don't link, the same invariant rail routing uses when a transfer makes no sense.

First Rosetta Stone project where **the data structure is a typed chain** and the invariant is that consecutive elements compose. G106's events were independent points; G109's episodes were structurally ordered but independent. G110 is the first where the Nth element *depends* on the N-1st for correctness.

## Insight: Time Chain Is a Second Invariant

Beyond location, the *temporal* chain must be monotone: leg[N+1].depart_ms must be >= leg[N].arrive_ms. Otherwise the traveller is in two places at once. G110 flags `TimeConflict` as an `Error`.

The time chain and location chain are **independent** invariants — a trip can violate either or both. The validator reports both classes separately so the user can fix them independently. A single "this trip is broken" verdict would lose this information.

First Rosetta Stone project with **two independent consecutive-element invariants**, each reported separately.

## Insight: Tight Layovers Are Warnings, Not Errors

A 15-minute layover is legal (the traveller has time), but probably unwise — missed connections happen. G110 flags gaps below `min_layover_minutes` as `Warning`, not `Error`. The user may accept the risk, but the warning is there.

Zero-minute layover (connecting at instant) is neither a TightLayover nor a TimeConflict — it's an edge case where the connection is theoretically legal but physically tight. G110 treats it as acceptable (no issue), which matches how airlines model "legal connection time".

First Rosetta Stone project where **a continuum of connection time produces different severity responses** — Error for negative, no-issue for zero, Warning for >0 but below threshold, no-issue for above threshold.

## Insight: Leg Duration Has Its Own Sanity Checks

Independent of the chain, each leg has its own integrity: `arrive_ms >= depart_ms`. Zero duration (instant teleportation) is a Warning; negative duration (arriving before departing) is an Error. These checks run per-leg, not per-pair.

First Rosetta Stone project where **the validator has two layers**: per-element checks (is this leg internally consistent?) and per-pair checks (do these two legs compose?). The layers are independent and their results merge into one issue list.

## Insight: Layovers Are Derived From Consecutive Legs

`layovers(trip)` walks pairs of consecutive legs and emits `Layover { location, arrive_ms, depart_ms, duration_ms }` for each gap. Pure function of the trip. The UI renders them as "3h 45m in SFO" rows between leg cards.

Because layovers are derived from legs (not stored), they always reflect current state. If the user edits a leg's depart time, the layover updates on next read. Same pattern as G095's spreadsheet formulae and G109's progress roll-up.

First Rosetta Stone project where **per-pair derivations are a distinct view** of the same underlying sequence.

## Insight: Trip Metrics Separate Span From Transit

Three time quantities for a trip:
* **Total span** — first depart to last arrive (what the traveller is "on the trip" for).
* **In-transit** — sum of leg durations (actually moving).
* **Total layover** — span minus transit (waiting).

A trip that's 10 hours total span with 4 hours of flights has 6 hours of layovers. The metrics decompose cleanly. Users care about all three — span for "when am I back?", in-transit for "how much flying?", layovers for "how much airport time?".

First Rosetta Stone project where **three derived metrics relate via a linear identity** (span = transit + layover). Same decomposition every trip-planning app ships; G110 formalises it.

## Choreographic Case: Vault Trip Planner

```innate
(@vault-trip-planner){
  @trip <- @trip/load{path: "trips/sfo-lax-2026-05.yaml"}
  @issues <- @trip/validate{trip: @trip}
  @layovers <- @trip/layovers{trip: @trip}

  @ui/render-trip{
    title: @trip.title,
    legs: @trip.legs,
    layovers: @layovers,
    issues: @issues,
    metrics: {
      span: @trip.total-span-ms,
      in-transit: @trip.in-transit-ms,
      layover: @trip.total-layover-ms,
      cost: @trip.total-cost-cents
    }
  }

  @on-user-edits-leg (@idx @new-leg){
    @trip.legs[@idx] <- @new-leg
    ;; Re-validate on every edit — issues update immediately.
    @trip/validate{trip: @trip}
  }
}
```

The vault's trip pane shows legs, computed layovers, and a live issues list. Edits re-trigger validation; the UI never has to recompute the chain invariants itself.

## Structures

```innate
(defenum transport-mode FLIGHT | TRAIN | BUS | CAR | WALK)

(defstruct leg
  id          : Int
  mode        : TransportMode
  origin      : String
  destination : String
  depart-ms   : Int
  arrive-ms   : Int
  cost-cents  : Int
  carrier     : String)

(defstruct trip
  id                    : Int
  title                 : String
  legs                  : [Leg]
  min-layover-minutes   : Int)

(defstruct layover
  location     : String
  arrive-ms    : Int
  depart-ms    : Int
  duration-ms  : Int)

(defenum issue-severity INFO | WARNING | ERROR)

(defenum issue-kind
  LOCATION_MISMATCH | TIME_CONFLICT | TIGHT_LAYOVER
  | ZERO_DURATION_LEG | NEGATIVE_DURATION_LEG)

(defstruct issue
  severity  : IssueSeverity
  kind      : IssueKind
  leg-idx   : Int
  detail    : String)
```

## Resolver Natives

```innate
@trip/new{id, title}                  -> Trip
@trip/add-leg{trip, leg}              -> Unit
@trip/validate{trip}                  -> [Issue]
@trip/layovers{trip}                  -> [Layover]
@trip/total-span-ms{trip}             -> Int
@trip/in-transit-ms{trip}             -> Int
@trip/total-cost-cents{trip}          -> Int
@trip/total-layover-ms{trip}          -> Int
```

## Demo

```innate
(@demo){
  @trip <- @trip/new{id: 1, title: "NYC -> SFO -> LAX"}
  @trip/add-leg{trip: @trip, leg: {origin: "JFK", destination: "SFO",
                                     depart-ms: 100_000_000, arrive-ms: 120_000_000,
                                     cost-cents: 30000}}
  @trip/add-leg{trip: @trip, leg: {origin: "SFO", destination: "LAX",
                                     depart-ms: 130_000_000, arrive-ms: 135_000_000,
                                     cost-cents: 15000}}

  @trip/validate{trip: @trip}        ;; -> []  (valid chain)
  @trip/layovers{trip: @trip}        ;; -> [{location: "SFO", duration-ms: 10_000_000}]
  @trip/total-cost-cents{trip: @trip} ;; -> 45000
}
```

## Where

Location chain MUST match exactly — case-sensitive string equality. If users want case-insensitive or airport-code aliasing, that's a normaliser applied before validation. Time chain MUST be non-strict (>=, not >) — same-instant connections are legal if physically feasible. Layover threshold MUST be a per-trip parameter — international trips and domestic ones need different minimums. Per-leg duration checks MUST be independent of chain checks — a broken leg should be flagged even if the chain is also broken. Layovers MUST be derived, NOT stored — editing a leg invalidates stored layovers silently. Issues MUST have per-leg indexes — UIs anchor error rendering to the affected leg. Severity ladder MUST be three-tier (Info/Warning/Error) — same convention as G101 so tooling can thresh uniformly across Rosetta Stone modules.
