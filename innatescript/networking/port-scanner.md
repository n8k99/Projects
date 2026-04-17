# G038 — Port Scanner

> Probe a host's ports to discover listening services.

```yaml
id: G038
title: Port Scanner
category: networking
requires: [G033-ftp-protocol, G013-credit-card-validator, G015-dijkstra]
provides: [boundary-probing, service-discovery, concurrent-exploration, change-detection]
```

## Insight: Systematic Exploration of a Boundary

Every networking project before G038 *knew* what it was connecting to. FTP connected to a file server. NTP connected to a time server. Chat connected to a chat server. Weather connected to an API endpoint. The port scanner doesn't know — it **discovers** what's on the other side by probing systematically.

This is the first time the Rosetta Stone builds a tool for **boundary exploration**: given a host, what services exist? The scanner doesn't assume. It tests. Each port probe is a question: "is anyone listening here?" The answers — open, closed, filtered — build a map of the host's surface.

In the noosphere, this is agent capability discovery. Given a new agent, what operations does it support? The resolver could probe: `@agent/list-capabilities` is the port scan. Each capability is an open port. The scan builds a capability profile before the choreography begins.

## Insight: Three States, Not Two

A port isn't just open or closed. It can be **filtered** — something between the scanner and the port is dropping packets. The probe doesn't time out because the port is slow; it times out because a firewall silently ate the connection attempt. The three-state model (open/closed/filtered) is richer than binary success/failure.

This maps to InnateScript's resolution model. An `@reference` can resolve (open), fail with an error (closed), or hang with no response (filtered). The resolver needs to distinguish between "the agent said no" and "the agent didn't respond." G013's structural validation was binary — pass or fail. G038 adds a third state: *unknown*. The `<-` gate needs a timeout that distinguishes refusal from silence.

## Insight: Well-Known Ports Are a Service Registry

Port 22 is SSH. Port 80 is HTTP. Port 5432 is PostgreSQL. The well-known port table is a global service registry — a convention that maps numbers to meanings. The scanner doesn't just find open ports; it maps them to services using the registry.

In the noosphere, the resolver's namespace serves the same function. `@weather` maps to the weather service. `@calendar` maps to the calendar service. The namespace is the service registry. Port scanning discovers which entries in the registry have live implementations behind them.

## Insight: Concurrent Probing Is the Natural Model

Scanning 1024 ports sequentially takes 1024 × timeout. Scanning concurrently takes roughly timeout × (1024 / workers). The speedup is linear with parallelism because each probe is independent — no shared state, no ordering dependency. This is `@map` from G017 applied to network probing: apply the same operation to each port, collect results, no inter-probe coordination needed.

The scanner's thread pool is the resolver's `concurrent` block. Each probe runs independently. The results `join` into a scan report. The `where` evaluates the aggregate: how many ports are open? Which services are exposed? Is the host's surface larger or smaller than expected?

## Insight: Scan Comparison Is Change Detection

`compare_scans(before, after)` finds newly opened ports, newly closed ports, and ports that stayed the same. This is the first **temporal diff** in the Networking category — not comparing two different hosts, but comparing the *same host at two different times*. The diff reveals change: a new service appeared, an old one disappeared.

This connects to G025's journal (append-only event log) and G034's time synchronization. The scan report is a snapshot. Two snapshots with timestamps form a diff. A sequence of diffs is a changelog. The port scanner, run periodically, becomes a service monitoring system — G011's alarm clock triggering G038's scan, producing G025's journal entries about what changed.

## Choreographic Case: Infrastructure Audit

```innate
(@infra-audit){
  concurrent {
    @droplet_scan <- @scan{host: "144.126.251.126", ports: @common-ports}
    @local_scan   <- @scan{host: "127.0.0.1", ports: @common-ports}
  }
  join {
    @droplet_diff <- @compare{before: @last_droplet_scan, after: @droplet_scan}
    @local_diff   <- @compare{before: @last_local_scan, after: @local_scan}
  }
  where {
    no_unexpected_ports: @droplet_diff.newly-opened.length == 0
    expected_services_alive: @droplet_scan.open-ports.contains(5432)
      AND @droplet_scan.open-ports.contains(8080)
    // The where doesn't judge whether ports should be open.
    // It judges whether the surface changed unexpectedly.
  }
}
```

The `where` is a security gate: if new ports appeared that nobody opened, something is wrong. The scan comparison is a `<-` gate on infrastructure state — cheap to compute, catches drift before it becomes a breach.

## Structures

```innate
(defstruct port-result
  port       : Nat
  status     : :open | :closed | :filtered
  service    : String
  banner     : String
  latency-ms : Float)

(defstruct scan-report
  host           : String
  ports-scanned  : Nat
  open-ports     : [PortResult]
  closed-ports   : Nat
  filtered-ports : Nat
  duration-ms    : Float)
```

## Resolver Natives

```innate
@scan{host: String, ports: [Nat]}                 -> ScanReport
@scan{host: String, range: (Nat, Nat)}            -> ScanReport
@scan{host: String, ports: @common-ports}          -> ScanReport
@compare{before: ScanReport, after: ScanReport}    -> ScanComparison
@lookup-service{port: Nat}                         -> String
```

## Demo

```innate
(@demo){
  @scanner <- @port-scanner{timeout-ms: 500}
  @report <- @scanner/scan-common{host: "127.0.0.1"}
  @print{@report.summary}
  ;; => Scan report for 127.0.0.1
  ;;      Scanned 31 ports in 450ms
  ;;      Open: 3, Closed: 27, Filtered: 1
  ;;      Open ports:
  ;;        22/open (ssh) (0.5ms)
  ;;        5432/open (postgresql) (0.8ms)
  ;;        5433/open (postgresql-alt) (1.2ms)
}
```
