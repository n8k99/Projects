# G080 — Scheduled Auto Login and Action

> The Rosetta Stone's first **cron-like scheduler**. Tasks are *data*: (name, schedule, action, credential-ref). The scheduler fires due tasks on every `tick(now)` through a pluggable dispatch function. **Catch-up policy is "skip missed, not backfill"** — cron's behaviour, because thundering-herd-after-downtime is worse than skipping a run.

```yaml
id: G080
title: Scheduled Auto Login and Action
category: web
requires: [G072-file-downloader, G077-password-safe, G078-media-player]
provides: [cron-schedule-types, skip-missed-policy, pluggable-dispatch, task-history, task-enable-disable]
```

## Insight: Tasks Are Data

The most important design choice in G080 is that **tasks are data, not code**. A `Task` is a struct with a name, a schedule enum, an action string, and an optional credential reference. It can be serialised to JSON and deserialised on the next boot — the scheduler's state is reconstructible from the task list + history.

This is the opposite of the naive approach where tasks are closures / lambdas / function pointers scheduled directly. Closure-based scheduling is easy to start with but painful to persist, audit, or migrate: you can't inspect what's scheduled, you can't pause or modify a task without re-writing the code that created it, you can't restart the scheduler and recover the schedule.

First Rosetta Stone project where **data-driven scheduling** is explicit. Systemd timers, cron's `/etc/crontab`, Kubernetes CronJobs, Airflow DAGs, the vault's future scheduled-automations — all use this model. The task is a declarative description; the executor is separate and pluggable. G080's `dispatch` function is the bridge.

## Insight: Catch-Up Policy Is a Design Choice With Real Consequences

If the scheduler goes offline for an hour and a task was due to fire 30 times during that hour, what happens when the scheduler comes back up?

Two choices:
- **Backfill**: fire 30 times in rapid succession.
- **Skip-missed**: fire once (acknowledging the deadline was missed), reschedule to the next natural boundary.

G080 picks **skip-missed**. Same as cron. Reason: a thundering herd of 30 simultaneous backups, 30 simultaneous credential refreshes, 30 simultaneous API calls — is almost never what the user wants. If they *do* want it, they should schedule the backfill explicitly.

First Rosetta Stone project where **a visible policy choice about missed work** is the design's core decision. The alternative "fire every missed instance" is correct for some semantics (event replay, idempotent work) and wrong for others (rate-limited APIs, resource-expensive backups). G080 documents the choice and sticks to it.

Analogues: systemd's `Persistent=false` (skip) vs `Persistent=true` (fire once on recovery, still not backfilling). Kubernetes CronJobs have `startingDeadlineSeconds` that bounds how far back a missed run can still fire. Most production schedulers default to skip-missed precisely because backfill blows up under downtime.

## Insight: Schedule Is a Sum Type, Not a Cron String

Rather than parse cron syntax (`* * * * *`), G080 uses a **Schedule sum type** with three variants: `EveryMs(interval)`, `AtMs(absolute_time)`, `Daily(hour, minute)`. Each variant has its own computation for "when is the next fire?"

The tradeoff: cron strings are expressive (every 5 minutes on weekdays from 9 to 5) but require a parser and runtime evaluation. Enum variants are less expressive (you have to add a new variant for each pattern) but trivially typecheckable and testable. G080 picks the simpler approach; production schedulers often add a full cron parser on top.

First Rosetta Stone project where **the schedule grammar is an enum, not a string DSL**. Related pattern in the noosphere: rather than parsing trigger conditions from a YAML front-matter field, represent them as a sum type the choreography runtime can exhaustively match.

## Insight: Disabled Tasks Are Retained, Not Deleted

`set_enabled(id, false)` doesn't remove the task — it flips a flag. Re-enabling restores firing. This lets users pause tasks, debug failures without losing the schedule, test what happens with fewer concurrent runs, etc.

The alternative (only remove/add) loses the task's metadata on every pause. Every production scheduler has this distinction (cron's commented-out lines, systemd's `mask` vs `delete`). G080 models it with a boolean flag.

First Rosetta Stone project with **soft-disable** as a first-class state. Previous projects had hard state changes (G077 lock/unlock, G078 stop/play). G080 adds a third option: present-but-dormant.

## Insight: History Is Per-Task, Queryable, and Immutable

Every run (successful or failed) is appended to `history`. Runs carry task-id, start/finish times, status, and a message. `history_for(task_id)` filters; total history is the append-only record.

The history is **diagnostic gold**: "why did yesterday's backup fail?" → look at the run for that task's last fire. "Is this task chronically slow?" → compute elapsed time across recent runs. "Which tasks fired in the last hour?" → filter by `started_ms`.

First Rosetta Stone project where **execution history is a first-class observability primitive**. G073 had per-session history for one connection; G080 has global execution history across all scheduled tasks. The noosphere's future observability will have this exact shape — every choreography execution logged as a Run, queryable by task/status/time.

## Insight: Dispatch Function Is the Extension Point

The scheduler doesn't know what any task *does*. It knows when they fire and that they return (succeeded, message). The actual work — downloading files, sending emails, running scripts, authenticating to APIs — happens inside the user-provided `dispatch` function, which receives the task and the current time.

This is the same **abstract-transport** pattern as G072 File Downloader. Tests use a fake dispatch that succeeds/fails deterministically; production plugs in a real one that looks up credentials, invokes external tools, and returns results.

First Rosetta Stone project with **dependency injection at the core execution boundary**. The scheduler is 200 lines of scheduling logic; the dispatch function is where the real work lives, and it's a parameter. The split is what makes the scheduler reusable across projects with wildly different task semantics.

## Choreographic Case: Vault Automation

```innate
(@vault-automations){
  @sched <- @scheduler/new

  @scheduler/add-task{
    sched: @sched, name: "daily backup",
    schedule: {kind: "daily", hour: 3, minute: 0},
    action: "backup-vault", credential: "backup-host",
    first_fire_base_ms: @now
  }

  @scheduler/add-task{
    sched: @sched, name: "agent heartbeat",
    schedule: {kind: "every_ms", interval_ms: 60000},
    action: "ping-agents", credential: null,
    first_fire_base_ms: @now
  }

  @every 1.s {
    @runs <- @scheduler/tick{sched: @sched, now_ms: @now,
                              dispatch: @vault-action-dispatcher}
    @for r in @runs {
      when (@r.status == "failed") {
        @alert/dispatch{task: @r.task_id, msg: @r.message}
      }
    }
  }
}
```

The vault's future automations will use G080's shape: declarative task list, pluggable dispatcher, tick-driven execution, per-run history for debugging failures. The primitive composes with G077 (credential lookup) and G072 (file operations with atomic promotion).

## Structures

```innate
(defenum schedule-kind EveryMs | AtMs | Daily)

(defstruct schedule
  kind          : ScheduleKind
  interval-ms   : Int
  at-ms         : Int
  hour, minute  : Int)

(defstruct task
  id              : Int
  name            : String
  schedule        : Schedule
  action          : String
  credential-ref  : String?
  next-fire-ms    : Int
  enabled         : Bool)

(defstruct run
  task-id, started-ms, finished-ms : Int
  status   : "succeeded" | "failed"
  message  : String)

(defstruct scheduler
  tasks    : [Task]
  history  : [Run]
  next-id  : Int)
```

## Resolver Natives

```innate
@scheduler/new                                                   -> Scheduler
@scheduler/add-task{sched, name, schedule, action,
                     credential?, first_fire_base_ms}            -> TaskId
@scheduler/remove-task{sched, id}                                -> Bool
@scheduler/set-enabled{sched, id, enabled}                       -> Bool
@scheduler/get-task{sched, id}                                   -> Task?
@scheduler/history{sched}                                        -> [Run]
@scheduler/history-for{sched, id}                                -> [Run]
@scheduler/next-fires{sched}                                     -> {TaskId -> Int}
@scheduler/tick{sched, now_ms, dispatch}                         -> [Run]
```

## Demo

```innate
(@demo){
  @s <- @scheduler/new
  @scheduler/add-task{sched: @s, name: "hourly backup",
                       schedule: {kind: "every_ms", interval_ms: 60000},
                       action: "backup-vault", credential: "backup-host",
                       first_fire_base_ms: 0}
  @scheduler/add-task{sched: @s, name: "morning report",
                       schedule: {kind: "daily", hour: 9, minute: 30},
                       action: "send-report", credential: "email",
                       first_fire_base_ms: 0}
  @scheduler/add-task{sched: @s, name: "one-off migration",
                       schedule: {kind: "at_ms", at_ms: 150000},
                       action: "run-migration", credential: null,
                       first_fire_base_ms: 0}

  @scheduler/tick{sched: @s, now_ms: 60000, dispatch: @always-ok}
    ;; -> [{task 1 succeeded}]
  @scheduler/tick{sched: @s, now_ms: 34200000, dispatch: @always-ok}
    ;; -> [{task 1 succeeded} {task 2 succeeded}]       ;; backup + 9:30am report
    ;; Note: task 1 fires ONCE not 570 times, despite 570 intervals passing
    ;; since last tick. Skip-missed policy.
}
```

## Where

The initial `next_fire_ms` MUST be computed from `first_fire_base_ms` — a task added at time T with an `EveryMs(60000)` schedule MUST fire at T+60000 not at T+0; adding a task MUST NOT fire it immediately. Missed runs MUST be skipped, not backfilled — a tick far past multiple boundaries MUST produce exactly one run per due task, with the next fire scheduled to the next natural boundary; backfilling missed runs is a cron-antipattern and G080 refuses it. One-shot (`AtMs`) tasks MUST disable themselves after firing — leaving them enabled causes them to fire repeatedly on every subsequent tick. Daily schedules MUST compute "next occurrence strictly after now" — if today's target has already passed, schedule tomorrow; if not yet passed, schedule today. The dispatch function MUST be the only place task-specific logic lives — the scheduler itself MUST remain task-agnostic, so it can be used for HTTP polling, vault backups, agent heartbeats, credential refreshes, etc. without modification. Failed runs MUST be recorded with status=Failed and the dispatch's error message — they do NOT disable the task; disabling-on-failure is a separate concern (circuit breaker) outside G080's scope.
