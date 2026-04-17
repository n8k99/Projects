# G040 — Packet Sniffer

> Capture and analyze network packets with filtering and statistics.

```yaml
id: G040
title: Packet Sniffer
category: networking
requires: [G018-count-vowels, G020-count-words, G026-news-ticker, G038-port-scanner]
provides: [passive-observation, traffic-analysis, connection-tracking, anomaly-detection]
```

## Insight: Passive Observation — Watching Without Acting

Every networking project before G040 was **active** — the agent initiated contact, sent requests, probed ports. The packet sniffer is the first **passive** tool: it watches traffic without generating any. It doesn't connect. It doesn't send. It listens.

This is a fundamentally different mode of interaction. Active tools ask questions. The sniffer observes answers flowing between others. In the noosphere, this is the monitoring agent — an entity that watches choreographies execute without participating. Lena's `{nightly_summary}` is a sniffer: she doesn't produce the day's work, she observes what the other agents produced and reports on it.

The architectural insight: not every agent in a choreography needs to act. Some agents exist to observe, measure, and report. The sniffer is the prototype for the observer role.

## Insight: `@breakdown` Returns at the Network Level

G018 (Count Vowels) introduced `@breakdown` — measure a property across elements, return the distribution. G020 (Count Words) generalized it to different granularities. The packet sniffer applies `@breakdown` to network traffic: protocol distribution, bytes per IP, connections per source, bandwidth per second.

The same statistical vocabulary from the Text category now operates on packets. The measurement primitive is truly generic — it doesn't care whether it's counting vowels in a string or TCP segments in a capture. The shape of the operation is identical: partition by property, count per partition, return the distribution.

## Insight: Connection Tracking Is Bidirectional Identity

A connection between A:1234 → B:80 and B:80 → A:1234 is the **same** conversation. The `ConnectionKey` normalizes direction by sorting endpoints. This is the first bidirectional identity in the Rosetta Stone — recognizing that two apparently different flows are the same entity viewed from different sides.

In InnateScript, agent communication is inherently bidirectional. A request from Kathryn to Eliana and Eliana's response to Kathryn are the same conversation. The choreography needs connection tracking to correlate request with response — the same normalization the sniffer does with IP pairs.

## Insight: Anomaly Detection Is Statistical `where`

`detect_syn_flood` and `detect_port_scan` are pattern recognizers — they look for statistical signatures that indicate hostile behavior. A SYN flood is "too many SYN packets without matching ACKs." A port scan is "too many distinct destination ports from one source."

These are `where` expressions over traffic statistics. The `where` doesn't examine individual packets — it evaluates the **aggregate shape** of the traffic. This is the same pattern as G013's credit card validation scaled up: cheap statistical analysis gates expensive response (blocking the attacker, alerting the operator).

The noosphere needs the same pattern. An agent that sends too many requests without processing responses is a SYN flood. An agent that probes every capability without using any is a port scan. The traffic analysis patterns from network security apply directly to agent behavior monitoring.

## Choreographic Case: Network Health Observer

```innate
(@network-observer){
  @capture <- @sniffer{filter: {protocol: :tcp}}
  // The sniffer runs continuously, ingesting packets
  @alarm{every: 60s} -> {
    @stats <- @capture/summary
    @syn_suspects <- @detect_syn_flood{capture: @capture, threshold: 100}
    @scan_suspects <- @detect_port_scan{capture: @capture, threshold: 20}
    where {
      no_floods: @syn_suspects.length == 0
      no_scans: @scan_suspects.length == 0
      bandwidth_normal: @stats.bytes_per_second < @threshold
    }
  }
}
```

The observer runs alongside the network. It doesn't participate. It watches. The `where` evaluates health. When the `where` fails, the observer raises an alert. Passive monitoring with active judgment.

## Structures

```innate
(defstruct packet
  timestamp-ms : Nat
  src-ip       : String
  dst-ip       : String
  src-port     : Nat
  dst-port     : Nat
  protocol     : :tcp | :udp | :icmp | :arp | :dns | :http | :unknown
  size         : Nat
  flags        : [String]
  ttl          : Nat)

(defstruct capture-filter
  src-ip   : String?
  dst-ip   : String?
  protocol : Protocol?
  port     : Nat?
  min-size : Nat
  max-size : Nat)

(defstruct connection-stats
  key          : ConnectionKey
  packet-count : Nat
  total-bytes  : Nat
  first-seen   : Nat
  last-seen    : Nat)
```

## Resolver Natives

```innate
@sniffer{filter: CaptureFilter?}                        -> PacketCapture
@capture/ingest{packet: Packet}                          -> Bool
@capture/summary                                          -> String
@capture/protocol-breakdown                               -> {Protocol -> Nat}
@capture/top-talkers{n: Nat}                             -> [(String, Nat)]
@capture/unique-ips                                       -> [String]
@detect_syn_flood{capture: PacketCapture, threshold: Nat} -> [String]
@detect_port_scan{capture: PacketCapture, threshold: Nat} -> [String]
```

## Demo

```innate
(@demo){
  @cap <- @sniffer{}
  @cap/ingest{packet: {src-ip: "10.0.0.1", dst-ip: "10.0.0.2",
    src-port: 1234, dst-port: 80, protocol: :tcp, size: 100, flags: ["SYN"]}}
  @cap/ingest{packet: {src-ip: "10.0.0.2", dst-ip: "10.0.0.1",
    src-port: 80, dst-port: 1234, protocol: :tcp, size: 60, flags: ["SYN", "ACK"]}}
  @print{@cap/summary}
  ;; => Capture: 2 packets, 160 bytes
  ;;      Unique IPs: 2
  ;;      Connections: 1
  ;;      Protocol breakdown:
  ;;          TCP: 2
}
```
