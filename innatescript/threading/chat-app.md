# G067 — Chat Application (Threading)

> The first Rosetta Stone project where **every actor is both producer and consumer** simultaneously. Bidirectional broadcast: one send fans out to every other actor's inbox, and every actor can both send and read. Dynamic membership — actors join and leave while others keep talking.

```yaml
id: G067
title: Chat Application (Threading)
category: threading
requires: [G065-progress-bar, G066-download-manager]
provides: [broadcast, pubsub, bidirectional-flow, total-ordering-via-counter, dynamic-membership]
```

## Insight: Every Actor Is Both Producer and Consumer

G065 separated the producer (worker thread) and the consumer (display thread). G066 had one producer (the manager) and N consumers (workers). G067 is the first project where **every actor is both**: Alice sends to Bob's inbox and reads from her own. Bob sends to Alice's inbox and reads from his own. The graph of communication is fully connected, not a pipeline.

This is what "chat" means structurally. It's also what **peer-to-peer** means, what **federated** means, what **multi-agent** means. Every system where participants talk to each other rather than through a central pipeline has this bidirectional-flow property. G067 is the primitive.

The production systems this mirrors:
- IRC, Matrix, Discord — multi-user broadcast rooms.
- The noosphere's agent mesh — agents that publish events and subscribe to each other's events concurrently.
- The vault's shared document collaboration — multiple editors reading and writing the same document, each seeing the others' changes.
- Gossip protocols — nodes that receive messages and forward them to peers.

All share G067's shape.

## Insight: Broadcast Is Fan-Out on Every Send

A send from Alice doesn't go to "the next available worker" (G066's pull model). It goes to **every joined user's inbox simultaneously**. The fan-out happens *per send*, not once at startup; the set of fan-out targets changes dynamically as users join and leave.

This makes broadcast fundamentally different from work distribution. In G066, each job goes to exactly one worker and the queue drains; in G067, each message goes to every member and the inboxes accumulate. The memory profile is different: a 1,000-user chat room where Alice says "hello" creates 1,000 inbox entries, not 1. Broadcast is expensive at scale, which is why real chat systems use tricks (server-side fan-out, pub/sub brokers, per-room partitioning) to mitigate the cost.

G067 presents the naive fan-out — one message, loop over every inbox, push. That's sufficient for small rooms and is what most chat systems start with.

## Insight: Dynamic Membership Is a New Synchronisation Problem

Users join and leave **while other users are mid-conversation**. This introduces race conditions not present in G065 (fixed single producer/consumer pair) or G066 (fixed worker count):

1. Alice sends while Bob is leaving. Does Bob receive the message?
2. Alice reads her inbox while Carol is joining. Does Carol see the messages sent before she joined?
3. Bob leaves and Alice sends. Does the send fail, or silently skip Bob?

G067 answers all three via a single lock: membership changes and sends serialise through the room lock. If Bob leaves first (lock acquired, removed, released), Alice's later send (lock acquired) sees the post-leave membership and skips Bob. If Alice sends first, Bob receives the message before leaving. There is no "in between" state because the lock makes every operation atomic with respect to the others.

Carol joining mid-conversation does **not** retroactively receive past messages (she wasn't in the inbox-iteration of earlier sends). This is the standard chat-room convention: join-time-forward. If a system wanted "backfill on join," it would consult the log (which always has the full history) and copy past messages into the new inbox at join time. G067 does not; the log is for audit, the inboxes are for delivery.

## Insight: Total Ordering from a Monotonic Counter Under a Single Lock

In a concurrent system with multiple senders, there is no natural total order of messages — wall-clock timestamps can collide or drift; network delivery order depends on the recipient's position. G067 establishes a total order by assigning a **monotonically increasing sequence number** to every message at the point of send, **under the same lock** that protects the user list.

Every recipient sees messages with strictly-increasing sequence numbers. Within a single sender, sequence numbers also strictly increase (since the sender's messages are processed serially by the lock). Across senders, the interleaving is determined by lock-acquisition order — which may be arbitrary, but *once determined, is consistent across all recipients*. Alice's message seq=5 reaches Bob as seq=5 and Carol as seq=5.

This is the **logical clock** pattern at minimal scale. Real distributed systems use Lamport clocks (per-node counters), vector clocks (per-node counters tracked across all nodes), or physical-logical hybrids (HLC). G067 is the single-machine case: one counter, one lock, total order.

The noosphere will need this. Multi-agent choreography with events from many sources can't use wall-clock timestamps safely; a room-scoped monotonic counter is the correct answer for any single-broker scenario. For multi-broker, move up to Lamport or HLC.

## Insight: The Log and the Inboxes Are Two Different Data Structures

The room maintains two structures:
- **`log`** — every message ever sent, append-only, permanent. Audit trail / history.
- **`inbox` per user** — unread messages for that user. Drain-on-read.

Their consistency requirements differ. The log is eventually consistent with the world (every send eventually appears in the log); the inboxes are immediately consistent with the sender's membership view (every send is either in every current inbox or none). Both are maintained atomically inside `send`, but they serve distinct queries: "what did we say?" (log) versus "what do I need to look at?" (inbox).

This split — **journal vs queue** — is everywhere in real systems. The email system has a sent-folder log and per-recipient inbox queues. The bank has transaction journals and account-balance state. The noosphere's conversations table is a log; the agent dispatch queue is an inbox. G067 introduces the primitive: the same events are recorded in two data structures serving different questions.

## Insight: Leave Doesn't Break Ongoing Sends — It Just Removes Future Delivery

When Bob leaves, any previously-delivered messages in Bob's inbox are discarded with Bob (the inbox is GC'd with the user entry). Messages already sent but not yet delivered — there is no such state in G067 because send is atomic (deliver-to-all-current-users inside the lock). So "in flight" messages don't exist; leave is clean.

A production chat system with network delivery *would* have in-flight messages (the server sent to Bob before receiving Bob's LEAVE; Bob's client gets the message anyway). The design space is large: drop on leave, deliver-best-effort-and-tombstone, deliver-to-a-store-and-let-Bob-see-on-rejoin. G067 doesn't model network; send is atomic; leave is simple.

The log survives both join and leave — it's the room's memory, independent of membership. A user who rejoins later has no inbox of past messages but **can query the log** to see what happened while they were away. This separation between membership and memory is deliberate and matches every real chat system.

## Choreographic Case: Agent-to-Agent Broadcast

```innate
(@research-coordinator){
  @room <- @chat/new-room
  @me <- @room/join{name: "Coordinator"}

  @spawn-agents [
    @spawn (@literature-reviewer){
      @id <- @room/join{name: "LiteratureReviewer"}
      @for query in @research-queries {
        @findings <- @search/papers{query}
        @room/send{sender: @id, text: @findings.summary}
      }
    }
    @spawn (@code-analyzer){
      @id <- @room/join{name: "CodeAnalyzer"}
      @for repo in @target-repos {
        @analysis <- @analyze/repo{repo}
        @room/send{sender: @id, text: @analysis.summary}
      }
    }
    @spawn (@synthesizer){
      @id <- @room/join{name: "Synthesizer"}
      @loop {
        @msgs <- @room/read{id: @id}
        @for m in @msgs {
          @synthesis <- @synthesize{inputs: [@prior-state, @m.text]}
          @prior-state <- @synthesis
        }
        @sleep 500ms
      }
    }
  ]

  @wait-for-all-agents-done
  @final-report <- @room/history
}
```

Three agents collaborate through the room: two produce findings, one synthesises. The synthesiser reads its inbox on a schedule, updating its internal state from every message. This is the minimal agent-mesh shape, and it composes directly on G067's primitive.

## Structures

```innate
(defstruct message
  seq        : Int
  from-id    : UserId
  from-name  : String
  text       : String)

(defstruct inbox
  name      : String
  messages  : [Message])

(defstruct chat-room
  users     : {UserId -> Inbox}      ;; protected
  log       : [Message]              ;; full audit trail
  next-id   : Int
  next-seq  : Int)
```

## Resolver Natives

```innate
@chat/new-room                           -> Room
@room/join{room, name}                   -> UserId
@room/leave{room, id}                    -> Unit
@room/send{room, sender, text}           -> Bool    ;; false if sender not a member
@room/read{room, id}                     -> [Message]    ;; drains inbox
@room/history{room}                      -> [Message]    ;; full log
@room/user-count{room}                   -> Int
```

## Demo

```innate
(@demo){
  @r <- @chat/new-room
  @alice <- @room/join{name: "Alice"}
  @bob   <- @room/join{name: "Bob"}
  @carol <- @room/join{name: "Carol"}

  @parallel {
    @for i in 0..20 { @room/send{sender: @alice, text: "A${i}"} }
    @for i in 0..20 { @room/send{sender: @bob,   text: "B${i}"} }
    @for i in 0..20 { @room/send{sender: @carol, text: "C${i}"} }
  }

  @history <- @room/history{room: @r}    ;; -> 60 messages, strictly increasing seq
  @carols  <- @room/read{room: @r, id: @carol}    ;; -> 60 messages (Carol read nothing yet)
}
```

## Where

Every message MUST receive a sequence number assigned under the room lock at send time. Sequence numbers MUST be monotonically increasing within the room; a send that returns without a number is a bug, not a feature. Broadcast MUST deliver to every currently-joined user's inbox INCLUDING the sender's own — this gives the sender a record of what they sent and matches the way real chat clients display "sent" messages. Leave MUST NOT retroactively remove a user's name from past log entries — the log is immutable history. Join MUST NOT retroactively deliver past messages to the new user's inbox — inboxes are join-time-forward; past messages are accessible only through the log. Sends from non-members MUST be rejected (return false / throw), not silently succeed with "from: 0" or similar — membership is a send-time precondition.
