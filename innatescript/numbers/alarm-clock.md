# Alarm Clock

A choreography that schedules and triggers time-based alerts.

## Design feedback

This is the project where InnateScript crosses from computation into temporality. Everything before G011 — pi digits, Fibonacci sequences, calculators, unit converters — evaluates on demand. You call a function, it returns a value, done. An alarm clock is different. An alarm clock says: "do nothing now, but when 07:00 arrives, do THIS." It introduces a new primitive that pure computation never needed: the scheduled obligation.

The insight is that the vault already has this. The temporal chain — daily notes, weekly reviews, monthly summaries, quarterly assessments, yearly retrospectives — is a nested stack of alarm clocks. The daily note template that fires `(@LenaMorris){nightly_summary}` every night at 23:00 is an alarm. The weekly review that triggers every Friday is an alarm. The quarterly assessment that fires on the first of every third month is an alarm. InnateScript didn't need to invent a scheduler. It already has one. The temporal chain IS the scheduler.

This reframes the relationship between the resolver and the temporal chain. The resolver evaluates expressions — it's the "what." The temporal chain determines "when." An alarm is the bridge between them: it tells the temporal chain WHEN to invoke the resolver, and tells the resolver WHAT to evaluate when invoked. This is a new architectural layer that pure computation (G001-G010) never surfaced.

The `until` keyword from G004 (infinite iterators) reappears here, but with different semantics. In G004, `until` bounded an infinite sequence — "keep generating Fibonacci numbers until you hit one above 1000." Here, `until` bounds a temporal wait — "hold this alarm until the target time arrives." A generator produces values. An alarm produces a single EVENT at a specific TIME. This is the first project that introduces event-driven semantics into InnateScript.

The difference between polling and events matters. A calculator evaluates on demand — you push a button, it computes. An alarm evaluates on schedule — time pushes the button for you. InnateScript needs both. The resolver handles on-demand evaluation. But alarms require something to watch the clock and fire when the moment arrives. Where does that something live? In the temporal chain. The vault's `Temporal/` directory structure — Daily Notes, Weekly, Monthly, Quarterly, Yearly — is the scheduler's cron table. Each level of the temporal hierarchy is an alarm resolution: daily alarms fire once per day, weekly alarms fire once per week, and so on.

```dpn
# Basic alarm: set and wait
@alarm{at: "07:00", label: "wake-up"} -> @wake
@alarm{at: "08:30", label: "standup"} -> @standup
@alarm{at: "23:00", label: "nightly-summary"} -> @nightly

# Check which alarms have fired
@alarm.check{now: @time.now} -> @triggered
@triggered.each -> @fired {
    @notify{message: "@fired.label triggered at @fired.at"}
}

# Cancel an alarm by label
@alarm.cancel{label: "standup"} -> @cancelled  # true

# List what's still pending
@alarm.pending -> @remaining
```

The `@alarm` native introduces a temporal primitive that the resolver doesn't evaluate immediately. Instead, it registers an obligation with the temporal chain: "at this time, produce this event." The resolver's job changes from "evaluate now" to "schedule for later." This is the same shift that happens when you move from synchronous function calls to asynchronous event handlers — the control flow inverts.

## Choreographic case

Morning operations alarm that triggers a concurrent check-in from all executive ghosts. This is the real power of alarms in InnateScript — they don't just beep, they initiate choreographies. An alarm firing is the starting gun for a coordinated multi-agent workflow.

```dpn
# Morning operations alarm
@alarm{at: "07:00", label: "morning-ops"} ->
concurrent [
    @kathryn{report: "overnight P&L"}
    @eliana{report: "system health"}
    @sarah{report: "today's schedule"}
]
join
@eliana{compile: "morning briefing"} -> @briefing
@nathan{deliver: @briefing}
```

The alarm at 07:00 doesn't just notify — it kicks off a concurrent fan-out to three agents. Kathryn pulls financial data, Eliana checks infrastructure, Sarah reviews the calendar. They work in parallel. When all three return, Eliana compiles their reports into a unified morning briefing, which is delivered to Nathan. The alarm is the trigger. The choreography is the payload.

This pattern — alarm triggers choreography — is how the vault's temporal chain actually works. The daily note template isn't just a reminder. It's a script that fires at a specific time and orchestrates a sequence of agent actions. The nightly summary calls Lena to review the day's work. The weekly review calls the full executive team for retrospective. Each temporal level has its own alarm and its own choreography.

```dpn
# Temporal chain as nested alarm stack
@alarm{at: "23:00", repeat: "daily", label: "nightly"} ->
    @lena{nightly_summary: @today}

@alarm{at: "friday 17:00", repeat: "weekly", label: "weekly-review"} ->
    concurrent [
        @kathryn{weekly: "financial review"}
        @eliana{weekly: "infrastructure report"}
        @sarah{weekly: "schedule retrospective"}
        @vincent{weekly: "design review"}
    ]
    join
    @sarah{compile: "weekly summary"} -> @weekly

@alarm{at: "first-of-month 09:00", repeat: "monthly", label: "monthly"} ->
    @kathryn{monthly: "budget reconciliation"}

@alarm{at: "first-of-quarter 09:00", repeat: "quarterly", label: "quarterly"} ->
    @nathan{quarterly: "strategic assessment"}
```

The nesting is the key insight. Daily alarms fire inside weekly alarms, which fire inside monthly alarms, which fire inside quarterly alarms. Each layer has its own resolution and its own choreography. The vault's `Temporal/` directory mirrors this nesting — it IS the alarm clock, implemented as a filesystem hierarchy.

## What this means

The alarm clock reveals the architectural boundary between computation and coordination. G001-G010 were all computation: take input, produce output, done. G011 introduces coordination over time. The resolver can evaluate any expression on demand, but it cannot, by itself, decide WHEN to evaluate. That requires a scheduler — something that watches time and fires events.

InnateScript's answer is that the scheduler already exists: it's the temporal chain. The vault's directory structure (`Temporal/Daily Notes/`, `Temporal/Weekly/`, etc.) is a cron table. Each entry is an alarm. Each alarm's payload is a choreography. The resolver evaluates the choreography when the scheduler fires the alarm.

This means InnateScript has two execution modes:
1. **On-demand**: the resolver evaluates an expression when asked (calculator, unit converter, Fibonacci)
2. **Scheduled**: the temporal chain fires an alarm, which invokes the resolver on a choreography (morning briefing, nightly summary, weekly review)

The `@alarm` native bridges these modes. It lets choreographies register temporal obligations that the scheduler fulfills. The resolver doesn't need to know about time. The scheduler doesn't need to know about evaluation. The alarm is the interface between them.

This is also where the distinction between `@alarm` and `until` crystallizes. In G004, `until` was a filter on a stream — "keep producing values until a condition is met." Here, `@alarm{at: "07:00"} until triggered` would mean "hold this obligation until the time arrives." But alarms don't produce a stream of values. They produce exactly one event. The `until` isn't filtering — it's waiting. This is a different kind of `until`: temporal suspension rather than stream termination. InnateScript may need both, and distinguishing them is the language design challenge that G011 surfaces.

## Native implementation

The resolver provides `@alarm{at: TIME, label: LABEL}` to register a temporal obligation. `@alarm.check{now: TIME}` compares pending alarms against the given time and returns those that have triggered. `@alarm.pending` lists all un-triggered alarms. `@alarm.cancel{label: LABEL}` removes a pending alarm. The host language provides time comparison. The temporal chain provides scheduling. The alarm native connects them.
