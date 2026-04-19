# G073 — Telnet Application

> The Rosetta Stone's first **interactive protocol server** — line-oriented, stateful per connection, command-dispatched. Not a socket-level implementation; a *protocol model* that slots onto any byte transport. G062's state-gated operations applied to a remote caller over a wire-format boundary.

```yaml
id: G073
title: Telnet Application
category: web
requires: [G062-vending-machine, G067-chat-app, G071-page-scraper]
provides: [line-oriented-protocol, session-fsm, command-dispatch-map, state-gated-commands, per-session-history]
```

## Insight: Protocol ≠ Transport

A telnet *application* is not a telnet *server*. The server binds a socket, reads bytes, writes bytes; the application decides what a line means once the socket delivers it. G073 implements the application. The socket is a detail slotted underneath.

This separation is the single most important lesson of G073. Every real protocol — HTTP, SMTP, IRC, Redis's RESP, Postgres's wire protocol — gains testability the moment the protocol logic is split from the byte transport. You can test the protocol with unit tests that call `handle_line(session, "login alice")` directly; you can test the transport with a mock that replays recorded byte streams; the two tests never need each other. First Rosetta Stone project where **the protocol/transport split** is the design.

Every integration test of the noosphere's future agent-dispatch over IPC will use this shape: the protocol is `{cmd, args, response}`; the transport is a Unix socket today, a TCP socket tomorrow, a WebSocket the day after. The protocol stays the same.

## Insight: Session State Is a Three-State FSM

`Unauthenticated → Authenticated → Disconnected`. Only `login` crosses the first boundary; only `quit` crosses the second. Everything else is a legal or illegal operation within a state. G062's vending machine had two states (Idle, Accepting); G073 has three, and the third (Disconnected) is terminal — no operation brings a session back from Disconnected; the client must reconnect.

**Terminal state is load-bearing.** A naive server might let commands run on disconnected sessions "because the data is still there." That's a bug, not a feature: Disconnected means the protocol has ended; subsequent commands represent either a logic error in the server or an adversary holding onto a session id. G073 refuses all commands on Disconnected with an error rather than silently processing them.

This is the first Rosetta Stone protocol with a **terminal state**. G062 Vending Machine's terminal states (Completed, Cancelled, Failed) were optional for the FSM's purpose; G073's Disconnected is mandatory — every session eventually reaches it. Every real connection-oriented protocol has this: TCP's CLOSED, HTTP/2's closed stream, IRC's QUIT, Postgres's terminated connection.

## Insight: Commands Are a Dispatch Map, Not a Switch

G073 implements commands as a `cmd → handler` lookup (the Python/CL/Rust/Go impls could easily make this a real map; the minimal versions use `match`/`switch` because the command set is fixed for the demo). The pattern is: parse `line` into `(cmd, args)`, look up `cmd`, delegate to the handler.

The handler-map pattern makes the command set **data, not code**. Adding a new command is adding an entry; removing is deleting an entry. Production protocol servers (Redis, Postgres, most chat servers) implement commands exactly this way. G073 presents the minimal model.

This matters for the noosphere: every choreography resolver is this pattern. `@agent/dispatch{cmd, args}` looks up `cmd` in the resolver table, delegates to the handler, returns the result. The resolver's command set is extensible at runtime; new natives can be registered without recompiling the resolver.

## Insight: State-Gated Commands Map the Security Model

- `help` works in any state — zero-trust command.
- `login <name>` only in Unauthenticated — idempotence is wrong here; double-login is an error, not a silent reassignment.
- `whoami` only in Authenticated — unauthenticated sessions get an error.
- `who` only in Authenticated — unauthenticated sessions cannot enumerate users (a real information-disclosure concern).
- `echo` in any state — trivially safe.
- `quit` in any non-terminal state — every session must be able to disconnect.

**The legal-in-state table is the protocol's security policy.** What commands are available to unauthenticated callers *is* the attack surface; anything more than `help` and `login` leaks information to unauthenticated observers. Every protocol bug in history has had a command that was accidentally legal in a state where it shouldn't have been.

First Rosetta Stone project where **access control is expressed through state-gating**, not through a separate permission system. Real servers often have both (RBAC + session-state); G073 shows that state-gating alone is sufficient for many protocols.

## Insight: Per-Session History Is Audit, Not Just Replay

Every line is recorded in the session's history (`(line, reply)` pairs). This is not a replay mechanism — G073 doesn't let you re-execute history. It's an **audit trail**: who typed what, what the server answered. The history survives until the session is garbage-collected.

This is the **journal-vs-queue** pattern from G067 Chat App applied to the per-session scope. The session history is a journal (permanent, append-only); the session's *current state* is the queue-like output of replaying the journal. If the state ever gets corrupted, the journal is the recovery source.

Production analogues: HTTP access logs, Postgres query logs, IRC channel logs, the vault's future choreography-execution log. All append-only, all per-session-scoped, all independent of whatever application-level state the log describes.

## Insight: Sessions Are Isolated — This Is the Whole Point

Alice's login does not affect Bob's session. Alice's quit does not terminate Bob's session. Alice's `who` sees the global authenticated list (which may include Bob) but her query doesn't mutate Bob. G073's `sessions_are_isolated` test verifies this explicitly because a multi-user server without isolation is not a multi-user server — it's a single-user server with multiple confused views of the same state.

First Rosetta Stone project where **state isolation across callers is a correctness property**. G067 had similar isolation for chat inboxes but the inboxes were symmetric data structures. G073 has asymmetric state (one user's login state) and must guarantee independence — a regression that let Alice's login leak into Bob's session would be a protocol-level security failure.

## Choreographic Case: Agent Command REPL

```innate
(@agent-repl){
  @srv <- @telnet/new-server
  @register-command{srv: @srv, name: "deploy",
                    handler: @agent/deploy-handler,
                    legal-states: [Authenticated]}
  @register-command{srv: @srv, name: "rollback",
                    handler: @agent/rollback-handler,
                    legal-states: [Authenticated]}

  @on-socket-accept (@conn){
    @sid <- @telnet/connect{srv: @srv}
    @loop {
      @line <- @conn/read-line
      @outcome <- @telnet/handle-line{srv: @srv, sid: @sid, line: @line}
      @conn/write-line{text: @outcome.text}
      break-if (@outcome.kind == "disconnect")
    }
  }
}
```

A custom agent-command REPL is a thin wrapper over G073's primitive: register commands, connect sessions, hand each incoming line to `handle-line`, write the response. The authentication/authorization model is free because state-gating is built in.

## Structures

```innate
(defenum state Unauthenticated | Authenticated | Disconnected)

(defstruct session
  id       : Int
  state    : State
  username : String?
  history  : [(String, String)])                ;; (line, reply) pairs

(defstruct outcome
  kind     : "reply" | "disconnect" | "error"
  text     : String)

(defstruct server
  sessions : {Int -> Session}
  next-id  : Int)
```

## Resolver Natives

```innate
@telnet/new-server                               -> Server
@telnet/connect{server}                          -> SessionId
@telnet/handle-line{server, sid, line}           -> Outcome
@telnet/session{server, sid}                     -> Session?
@telnet/authenticated-users{server}              -> [String]
```

## Demo

```innate
(@demo){
  @srv <- @telnet/new-server
  @a <- @telnet/connect{server: @srv}
  @b <- @telnet/connect{server: @srv}

  @telnet/handle-line{server: @srv, sid: @a, line: "login alice"}
  @telnet/handle-line{server: @srv, sid: @b, line: "login bob"}
  @telnet/handle-line{server: @srv, sid: @a, line: "who"}
    ;; -> reply: "alice, bob"
  @telnet/handle-line{server: @srv, sid: @b, line: "quit"}
    ;; -> disconnect: "goodbye"
  @telnet/handle-line{server: @srv, sid: @a, line: "who"}
    ;; -> reply: "alice"       (bob is disconnected, dropped from the list)
}
```

## Where

Commands MUST be parsed as `cmd [args]` — the first whitespace-separated token is the command, the rest is the argument string (trimmed). Empty input MUST produce an empty reply (not an error), matching the behaviour of every telnet-family protocol on blank lines. Commands legal only in certain states MUST check state BEFORE operating — calling `whoami` on an unauthenticated session MUST return an error, not an empty string. The Disconnected state MUST be terminal — no command brings a session back from Disconnected; the client MUST reconnect to get a new session id. History MUST be recorded for every line regardless of outcome — errors are just as auditable as replies. Session isolation MUST be a guarantee — no command on session A should mutate session B's state; the server's global state (authenticated-users list) is the only cross-session query.
