# G106 — Event Scheduler and Calendar

> The Rosetta Stone's sixth **Databases project**. Introduces the **half-open interval `[start, end)`** — the time convention every modern calendar API uses (ISO-8601, iCal RFC 5545, Outlook, Google Calendar, SQL `BETWEEN`). Adds **recurrence expansion**: a recurring template produces concrete occurrences on demand for any date range. **Overlap detection** is a two-inequality check: `a_start < b_end && b_start < a_end` — total, symmetric, no off-by-one.

```yaml
id: G106
title: Event Scheduler and Calendar
category: databases
requires: [G057-hash-table, G080-scheduled-auto-login, G102-remote-sql-tool]
provides: [half-open-intervals, interval-overlap-check, recurrence-expansion, conflict-detection]
```

## Insight: Half-Open Intervals Are the Universal Convention

An event from 10:00 to 11:00 occupies `[10:00, 11:00)` — ten-am inclusive, eleven-am exclusive. This is the convention everywhere modern: adjacent events `[10:00, 11:00)` and `[11:00, 12:00)` don't overlap; their endpoint touches but neither interval contains 11:00 on both sides.

Half-open intervals eliminate off-by-one bugs. With inclusive-inclusive `[10, 11]` vs `[11, 12]`, both contain 11 → they "overlap" — nonsensical. With half-open, the math is clean: `a.start < b.end && b.start < a.end` correctly reports them as non-overlapping.

First Rosetta Stone project where **the interval convention is a correctness premise**. G079's text game didn't deal with continuous time; G080's scheduler used point-in-time triggers, not ranges. G106 is the first with range semantics, and the convention is load-bearing.

## Insight: Overlap Check Is Two Inequalities

`a` and `b` overlap iff `a.start < b.end AND b.start < a.end`. Total (handles every case), symmetric (swap a and b — same result), no branching on which is earlier.

Most naïve overlap checks have bugs: checking only `a.start <= b.end` misses "a ends before b starts", checking containment misses partial overlaps. The two-inequality check is the shortest correct form.

First Rosetta Stone project with **a canonical interval algorithm** that every language implements identically. G097's point-in-polygon was similar (a classical algorithm worth implementing six times for cross-language confidence).

## Insight: Recurrence Is a Template + Expansion

A recurring event stores the **first occurrence** and a **recurrence rule** (Daily / Weekly / Monthly). Querying a date range expands the rule into concrete occurrences. The template doesn't materialise all future occurrences — that would require an end date (and many events recur forever).

Expansion is driven by a query window `[from, to)`. Start at the first occurrence, advance by the step (1 day, 7 days, 30 days), emit each that falls in the window, stop at `to` or the optional series-end.

First Rosetta Stone project where **a single template represents an unbounded set**. G082's CMS had revisions — concrete snapshots. G094's logs were eager. G106 is the first where **data is generated on demand** from a rule.

## Insight: Fast-Forward Skips Irrelevant Occurrences

Naïve recurrence expansion iterates from the first occurrence forward to the query window. For a daily event starting 10 years ago with a query of "next week", that's 3650+ wasted iterations.

G106 fast-forwards: computes how many `step`s to skip to reach the query window, adds them in one multiply-and-add. Then iterates normally. O(1) setup + O(range/step) emission, regardless of how long ago the first occurrence was.

First Rosetta Stone project where **an optimisation is load-bearing for correctness at scale**. Slower implementations are correct but intractable; the fast-forward is what makes a daily event with a 1-year lookback tractable instead of a 10-year iteration.

## Insight: Conflict Detection Is "Any Overlap Exists"

A candidate event at `[new_start, new_end)` conflicts with an existing event iff any occurrence of the existing event overlaps that window. For one-off events, check the single occurrence. For recurring events, expand into the window (with fast-forward) and check each.

`find_conflicts(start, end)` returns the IDs of conflicting events. The UI renders these ("4 conflicts"), and the user decides whether to schedule anyway, move, or bail.

First Rosetta Stone project with **a batch query that returns matching identifiers, not details**. G102's SELECT returned rows; G106's find_conflicts returns IDs — the caller can look up details separately. Different granularity for different purposes.

## Insight: Integer Milliseconds Avoid Timezone Complexity

Time is an `i64` count of milliseconds from an arbitrary epoch. Not a `DateTime` with timezone, not an ISO string, not a Unix timestamp per se. Arithmetic is integer; comparisons are integer; no leap second, no DST, no timezone offsets.

The caller converts wall-clock time to ms outside the engine. This is the right split — timezones are a display concern, and engine internals become simple once time is a number.

First Rosetta Stone project where **time is explicitly a monotonic integer**. Every cross-language confidence test depends on integer ms being identical across all six — none of them has to agree on timezone database versions.

## Choreographic Case: Vault Daily Calendar

```innate
(@vault-daily-calendar){
  @cal <- @calendar/load{path: "calendar/events.json"}
  @now-ms <- @clock/now
  @today-start <- @clock/start-of-day{ms: @now-ms}
  @today-end <- @today-start + 86400000

  @today-events <- @cal/events-between{calendar: @cal,
                                         from-ms: @today-start, to-ms: @today-end}
  @ui/render-agenda{events: @today-events}

  @on-user-proposes-event (@title @start-ms @end-ms){
    @conflicts <- @cal/find-conflicts{calendar: @cal, start-ms: @start-ms, end-ms: @end-ms}
    @when (@conflicts.length > 0){
      @ui/show-conflicts{ids: @conflicts}
    }
    @else {
      @cal/add{calendar: @cal, event: {title: @title, start-ms: @start-ms, end-ms: @end-ms}}
    }
  }
}
```

The vault's daily agenda queries the calendar by today's window; proposing a new event surfaces conflicts before committing. Recurrence expansion happens inside the query; the UI never sees it.

## Structures

```innate
(defenum recurrence NONE | DAILY | WEEKLY | MONTHLY)

(defstruct event
  id            : Int
  title         : String
  start-ms      : Int
  end-ms        : Int
  recurrence    : Recurrence
  series-end-ms : Int?)

(defstruct occurrence
  event-id : Int
  title    : String
  start-ms : Int
  end-ms   : Int)

(defstruct calendar
  events : {Int -> Event})
```

## Resolver Natives

```innate
@cal/new{}                                    -> Calendar
@cal/add{calendar, event}                     -> Unit
@cal/get{calendar, id}                        -> Event | null
@cal/events-between{calendar, from-ms, to-ms} -> [Occurrence]
@cal/find-conflicts{calendar, start-ms, end-ms} -> [Int]
```

## Demo

```innate
(@demo){
  @cal <- @cal/new{}
  @cal/add{calendar: @cal, event: {id: 1, title: "Standup",
                                     start-ms: @day{10}, end-ms: @day{10} + 1800000,
                                     recurrence: "daily", series-end-ms: @day{30}}}
  @cal/events-between{calendar: @cal, from-ms: @day{10}, to-ms: @day{15}}
  ;; -> 5 daily standups (days 10, 11, 12, 13, 14)

  @cal/find-conflicts{calendar: @cal, start-ms: @day{12}, end-ms: @day{12} + 43200000}
  ;; -> [1]  (the standup at day 12 falls inside the proposed window)
}
```

## Where

Intervals MUST be half-open `[start, end)` — adjacent events at the same boundary (A ends at 11:00, B starts at 11:00) MUST NOT be considered overlapping. Overlap check MUST use `a.start < b.end && b.start < a.end` — the two-inequality form is the shortest correct formulation and every other is either wrong or longer. Recurrence expansion MUST fast-forward to the query window — naive iteration from the first occurrence costs O(age/step) even for short query ranges. Series end MUST be inclusive if set — if `series_end_ms = 1000`, an occurrence at 1000 IS emitted (callers who want exclusive should subtract one step). Time MUST be integer milliseconds — any other representation introduces timezone/leap/DST ambiguity that the engine cannot resolve. Zero-duration events (`start == end`) ARE allowed but never overlap anything (two strict inequalities both fail) — callers needing point-in-time events should use 1-ms duration or a separate marker type.
