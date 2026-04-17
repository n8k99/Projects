# G045 — Site Checker with Time Scheduling

> Periodic health monitoring of web endpoints with alerting and history.

```yaml
id: G045
title: Site Checker with Time Scheduling
category: networking
requires: [G011-alarm-clock, G026-news-ticker, G036-fetch-weather, G038-port-scanner, G039-mail-checker]
provides: [health-monitoring, scheduled-checks, uptime-tracking, status-change-alerts]
```

## Insight: The Convergence Project

G045 is where five prior patterns converge:
- **G011 Alarm Clock**: the schedule — check every N seconds
- **G026 News Ticker**: the status dashboard with priority and TTL
- **G036 Weather**: the HTTP request to an external endpoint
- **G038 Port Scanner**: the three-state health model (up/down/degraded ≈ open/closed/filtered)
- **G039 Mail Checker**: the poll-filter-report pattern with change detection

The site checker isn't a new concept. It's the *composition* of five existing concepts into a single operational tool. This is the Rosetta Stone demonstrating what it claimed in G019 (Palindrome): operations compose. The building blocks are all here. The site checker assembles them.

## Insight: Four-State Health Is the Agent Liveness Model

Up, down, degraded, unknown. These are the states of any entity in the noosphere:
- **Up**: the agent responds correctly within time bounds
- **Down**: the agent fails or doesn't respond
- **Degraded**: the agent responds but slowly or incorrectly (wrong content, slow response)
- **Unknown**: no check has been performed yet

G038's port scanner had three states (open/closed/filtered). The site checker adds a fourth: **degraded** — the entity is responding, but not well enough. This matters for choreographies: a degraded agent might complete the task but miss the deadline. The `where` needs to score not just success but quality.

## Insight: Consecutive Failures Are Confidence Decay

One failure is noise. Two is concerning. Five is a pattern. The `consecutive_failures` counter is a confidence metric: how sure are we that the site is actually down? This prevents false alerts from transient failures while still catching real outages.

In the noosphere, agent health monitoring would use the same metric. An agent that fails once might have hit a transient error. An agent that fails five consecutive times is offline. The confidence in the diagnosis increases with consecutive observations — the same way NTP (G034) used multiple samples to reduce noise.

## Insight: Status Change Alerts Are Event-Sourced State Machines

The alert fires on state *transitions*, not states. "Down" isn't an alert — "up → down" is. The system only notifies when something *changes*. This is the event-sourcing pattern from G021 (Text Editor) applied to monitoring: the check history is the event log, the current status is the derived state, and alerts are triggered by state transitions.

## Choreographic Case: Infrastructure Health Dashboard

```innate
(@infra-health){
  @alarm{every: 5min} -> {
    concurrent {
      @em_site <- @check{url: "https://eckenrodemuziekopname.com", expect: 200}
      @wiki <- @check{url: "https://wiki.eckenrodemuziekopname.com", expect: 200}
      @api <- @check{url: "http://144.126.251.126:8080/health", expect: 200}
      @db <- @check{host: "144.126.251.126", port: 5432, type: :tcp}
    }
    join {
      @dashboard <- @compose{sites: [@em_site, @wiki, @api, @db]}
    }
    where {
      all_up: @dashboard.sites.all{|s| s.status == :up}
      no_degraded: @dashboard.sites.none{|s| s.status == :degraded}
      // Alert only on state changes, not steady-state failures.
      state_change -> @alert{to: @nathan, channel: :urgent}
    }
  }
}
```

The choreography runs every 5 minutes. Four endpoints checked concurrently. Results joined into a dashboard. The `where` evaluates aggregate health. Alerts fire only on transitions.

## Structures

```innate
(defstruct check-config
  url              : String
  name             : String
  interval-secs    : Nat
  expected-status  : Nat
  max-response-ms  : Float)

(defstruct check-result
  url              : String
  status           : :up | :down | :degraded | :unknown
  http-status      : Nat
  response-time-ms : Float
  error            : String
  checked-at       : Instant)

(defstruct site-history
  config               : CheckConfig
  checks               : [CheckResult]
  consecutive-failures : Nat)
```

## Resolver Natives

```innate
@monitor/add{config: CheckConfig}                -> Bool
@monitor/check{url: String}                       -> CheckResult
@monitor/check-all                                 -> [CheckResult]
@monitor/dashboard                                 -> String
@monitor/status{url: String}                       -> SiteHealth
@monitor/uptime{url: String}                       -> Float
@monitor/sites-by-status{status: SiteHealth}       -> [String]
```

## Demo

```innate
(@demo){
  @monitor <- @site-monitor{}
  @monitor/add{url: "https://eckenrodemuziekopname.com", name: "EM Website", interval: 300s}
  @monitor/add{url: "https://wiki.eckenrodemuziekopname.com", name: "EM Wiki", interval: 300s}
  @results <- @monitor/check-all
  @print{@monitor/dashboard}
}
```
