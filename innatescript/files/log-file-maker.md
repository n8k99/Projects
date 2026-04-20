# G094 — Log File Maker

> The Rosetta Stone's tenth **Files-category** project. Every production system needs logs. The essential pattern is four-fold: **append-only writes, level-based filtering at write time, size-bounded retention with archive rotation, structured readback with queries**. G094 captures all four. Tab-delimited text is the wire format — the one format `grep` and `awk` understand without ceremony.

```yaml
id: G094
title: Log File Maker
category: files
requires: [G008-average, G089-transaction-averages, G092-bulk-renamer, G093-mp3-tagger]
provides: [append-only-writes, level-filter-at-write, size-bounded-rotation, tab-delimited-round-trip]
```

## Insight: Append-Only Is the Simplest Write Discipline

The logger never updates an existing entry. Every `log()` call pushes a new row onto the end of `active`; rotation pops from the front. There is no `edit_entry`, no `delete_entry`, no `replace_entry`. That restriction is the single most important property of a log: **once written, never changed**.

Append-only is why logs are auditable. A system that rewrites its logs can lie about its past. A log that only appends has a linear history: every entry is either present (and therefore happened) or rotated (and therefore happened then too). This is the same discipline behind Git commits, Kafka streams, database WALs, blockchain ledgers.

First Rosetta Stone project where the **primary mutation is append-only by design**. G082's CMS had revisions (mutable content with immutable history); G086's launcher updated per-entry counters. G094 makes the mutation discipline explicit: writes go to the end; reads scan the whole thing; rotation moves entries to archive, never back.

## Insight: Level Filter at Write Time, Not Read Time

`Logger.log()` drops entries below `level_threshold` **at write time**. They never enter the active buffer. A naïve design would keep everything and filter at read time — but that wastes memory on entries the operator said they didn't want, and exposes debug-level output to production consumers who should never see it.

Write-time filtering is what `rustc`'s log crate, Python's `logging`, Go's `slog`, Java's `log4j` all do. The **threshold is policy**: operator configures `INFO` in production, `DEBUG` in development, and the application forgets about it. No per-call `if log_enabled(DEBUG) { ... }` boilerplate.

First Rosetta Stone project where **policy decisions happen at the point of capture, not consumption**. G085's quiz parser was permissive at capture (accept everything, let the caller filter); G094 takes the opposite stance for logs. Both are correct for their domain — logs at production volume cannot afford to retain everything.

## Insight: Rotation Is a Ring Buffer With an Archive

Unbounded logs fill disks. Naive "delete the file when it gets big" loses data. Log rotation is the middle ground: the **active buffer** holds the most recent N entries; when it overflows, the oldest entries move to the **archive**.

In the Rosetta Stone G094 keeps archive in memory for testability. Real systems (log4j, systemd-journald, logrotate) write the archive to numbered files (`app.log.1`, `app.log.2`...) or compress it. The data structure is the same: a ring of fresh entries plus a growing append-only archive. Queries run against `active` for speed; compliance audits scan `archive` for historical reconstruction.

First Rosetta Stone project with **two-tier storage**: hot (active, small, fast) and cold (archive, growing, scanned rarely). G090 had manifest vs payload but both hot; G094 is the first where recency dictates which tier data lives in.

## Insight: Tab-Delimited Is the Bash-Native Format

G094 serialises as `timestamp<TAB>LEVEL<TAB>category<TAB>message`. Not JSON, not binary, not a custom format — tabs separate fields, newlines separate rows. Why:
1. `grep` filters.
2. `awk -F'\t'` projects columns.
3. `sort` works out of the box.
4. `cut -f2` extracts one column.
5. Editors and pagers display it sanely.

This format dates back to Unix pipes and hasn't been improved on for operators. Modern systems add JSON for machine consumption (structured logging) but the tab-delimited variant is still every ops person's first tool. G094 picks the format that works with the universal toolbox.

First Rosetta Stone project where **the on-disk format is chosen for tool interop**, not parser convenience. G085's quiz format was for human edits; G094's tab format is for `grep | awk | sort | uniq -c`.

## Insight: Query Results Preserve Insertion Order

Every query returns entries **in the order they were logged**. Chronology is a contract; users reading query results expect time to march forward. A hash-based query (by category) could return random order without effort; G094 explicitly iterates the active buffer and filters, preserving order.

This is cheaper than G093's tag store (which sorts results) because the underlying data is already ordered. Append-only write discipline + no reordering = O(n) queries but zero sort cost.

First Rosetta Stone project where **the ordering of the data structure is the ordering of the output**. G093 paid for a sort because the map had no natural order; G094 gets ordering for free from append-only.

## Choreographic Case: Vault Observability

```innate
(@vault-observability){
  @logger <- @log/new{threshold: "INFO", max-entries: 10000}

  @on-any-event (@ts @level @category @message){
    @log/log{logger: @logger, ts: @ts, level: @level,
              category: @category, message: @message}
  }

  @on-user-asks-last-hour-errors {
    @now <- @clock/now
    @range <- @log/time-range{logger: @logger,
                               from: @now - 3_600_000, to: @now}
    @errors <- @log/level{logger: @logger, min-level: "ERROR"}
    @intersection <- @list/intersect{a: @range, b: @errors}
    @ui/render-log-view{entries: @intersection}
  }

  @on-periodic-rotation {
    @text <- @log/to-text{logger: @logger}
    @vault/append{path: "observability/app.log.${@yyyy-mm-dd}", content: @text}
  }
}
```

The vault layers an observability pane over the logger: events from any source funnel into one logger; queries drive the UI; periodic rotation flushes to dated files. Tab-delimited means operators can `grep` the flushed files with no extra tools.

## Structures

```innate
(defenum level DEBUG | INFO | WARN | ERROR)

(defstruct entry
  timestamp : Int
  level     : Level
  category  : String
  message   : String)

(defstruct logger
  level-threshold : Level
  max-entries     : Int
  active          : [Entry]
  archive         : [Entry])
```

## Resolver Natives

```innate
@log/new{threshold, max-entries}                 -> Logger
@log/log{logger, ts, level, category, message}   -> Unit
@log/level{logger, min-level}                    -> [Entry]
@log/category{logger, category}                  -> [Entry]
@log/message-contains{logger, needle}            -> [Entry]
@log/time-range{logger, from, to}                -> [Entry]
@log/to-text{logger}                             -> String
@log/parse-entries{text}                         -> [Entry] | null
```

## Demo

```innate
(@demo){
  @l <- @log/new{threshold: "DEBUG", max-entries: 100}
  @log/log{logger: @l, ts: 100, level: "INFO", category: "boot", message: "starting up"}
  @log/log{logger: @l, ts: 300, level: "WARN", category: "net", message: "slow response"}
  @log/log{logger: @l, ts: 400, level: "ERROR", category: "db", message: "connection lost"}

  @log/level{logger: @l, min-level: "WARN"}
  ;; -> [300/WARN/net/..., 400/ERROR/db/...]

  @log/category{logger: @l, category: "db"}
  ;; -> [400/ERROR/db/connection lost]
}
```

## Where

Writes MUST be append-only — no API for editing past entries, because audit integrity depends on it. Level filter MUST apply at write time — filtered-out entries never enter the buffer, so memory and downstream processing are both saved. Rotation MUST move oldest entries to archive, NOT drop them — logs under pressure must not silently lose data; archive is bounded by disk, not memory. Query results MUST preserve insertion (chronological) order — logs are read by humans who expect time to flow forward. Tab-delimited format MUST be used, NOT JSON by default — operators use `grep|awk|sort` and the format must compose with the Unix toolbox. Parse MUST reject on any malformed line — partial-parse of logs masks corruption that should be visible immediately.
