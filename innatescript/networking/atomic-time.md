# G034 — Get Atomic Time from Internet Clock

> Time synchronization is **consensus on a shared reference** — two parties agreeing on "now."
> This is the first consensus problem in the Rosetta Stone. Agents need temporal consensus for the temporal chain.

## Insight: Calibration

The offset is **calibration** — measuring drift between local and authoritative state. Every local clock drifts. The protocol doesn't replace the local clock; it measures how far it has wandered and applies a correction. Periodic recalibration keeps the noosphere synchronized.

## Insight: Uncertainty from Latency

Network latency introduces **uncertainty** — the timestamp was accurate when sent, but the message took time to arrive. Every agent response carries latency. NTP compensates by measuring round-trip time and estimating the midpoint. Perfect synchronization is impossible; the goal is bounded error.

## Insight: Choreographic Case

Noosphere clock sync — all agents agree on temporal chain boundaries. When multiple agents operate on shared state, they must agree on ordering. Time synchronization provides the foundation: a shared "now" from which sequence numbers, epochs, and chain boundaries derive meaning.

---

## Structures

```innate
(define-structure TimeServer
  "Authoritative time source. Wraps system clock with optional offset."
  :fields ((offset-seconds :type Float :default 0.0))
  :protocol time-authority)

(define-structure TimeClient
  "NTP-style client that queries a TimeServer and tracks sync state."
  :fields ((server        :type TimeServer)
           (local-offset  :type Float :default 0.0)
           (sync-offset   :type Float :default 0.0)
           (synced        :type Bool  :default false)
           (samples       :type (List Float) :default '()))
  :protocol time-consumer)
```

## Operations

```innate
(define-operation query-time (client)
  "Query the server and return its authoritative timestamp.
   Records the offset sample for averaging."
  :returns Float
  :effects (mutate client.samples)
  :protocol-step (time-consumer -> time-authority -> time-consumer)
  :body
  (let* ((t-send      (local-time client))
         (server-time (get-time (server client)))
         (t-recv      (local-time client))
         (round-trip  (- t-recv t-send))
         (estimated   (+ server-time (/ round-trip 2.0)))
         (sample      (- estimated t-recv)))
    (push! sample (samples client))
    server-time))

(define-operation get-offset (client)
  "Measured offset between local and server clock.
   Averages all collected samples for accuracy.
   Positive means local is behind (needs to advance)."
  :returns Float
  :body
  (when (empty? (samples client))
    (query-time client))
  (average (samples client)))

(define-operation sync (client)
  "Apply measured offset to correct local time estimate."
  :effects (mutate client.sync-offset client.synced)
  :body
  (set! (sync-offset client) (get-offset client))
  (set! (synced client) true))

(define-operation get-synced-time (client)
  "Local time corrected by synchronization offset."
  :returns Float
  :body
  (if (synced client)
      (+ (local-time client) (sync-offset client))
      (local-time client)))
```

## Choreography: Noosphere Clock Sync

```innate
(define-choreography noosphere-clock-sync
  "All agents in a noosphere domain agree on temporal chain boundaries."
  :participants ((authority :role time-authority)
                 (agents    :role time-consumer :cardinality *))
  :protocol
  (sequence
    ;; Phase 1: Each agent queries the authority independently
    (parallel-for agent in agents
      (repeat 3
        (query-time agent)))

    ;; Phase 2: Each agent computes and applies its offset
    (parallel-for agent in agents
      (sync agent))

    ;; Phase 3: Verify bounded error across all agents
    (assert
      (for-all agent in agents
        (< (abs (- (get-synced-time agent)
                   (get-time authority)))
           *max-sync-error*))))

  :invariants
  ((bounded-drift "After sync, all agents are within *max-sync-error* of authority")
   (monotonic     "Synced time never goes backward within an agent")
   (convergent    "Repeated sync rounds reduce average error")))
```

## Pattern: Calibration Protocol

```innate
(define-pattern calibration-protocol
  "Measure drift from authoritative source and apply correction.
   Generalizes beyond time to any quantity with local/remote divergence."
  :parameters ((local-reading  :type (-> Float))
               (remote-reading :type (-> Float))
               (apply-correction :type (Float -> Void)))
  :steps
  (1. "Sample: read both local and remote values")
  (2. "Compute: difference = remote - local")
  (3. "Average: collect multiple samples to reduce noise")
  (4. "Apply: adjust local by averaged offset")
  (5. "Verify: confirm corrected local matches remote within tolerance")
  :applications
  ((time-sync       "Clock offset correction — this exercise")
   (state-sync      "Vault replica reconciliation")
   (consensus-round "Agent agreement on shared value")))
```
