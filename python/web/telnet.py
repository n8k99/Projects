"""Telnet Application — line-oriented protocol with state-gated commands.

Not a socket server — the focus is the *protocol model*. Session FSM:
Unauthenticated → Authenticated → Disconnected. Commands have declared
legal states; calling a command in the wrong state returns an error
rather than crashing. G062's vending-machine state-gated-operations
applied to a protocol.
"""

from __future__ import annotations
from dataclasses import dataclass, field
from enum import Enum


class State(Enum):
    UNAUTHENTICATED = "unauthenticated"
    AUTHENTICATED = "authenticated"
    DISCONNECTED = "disconnected"


@dataclass
class Session:
    id: int
    state: State = State.UNAUTHENTICATED
    username: str | None = None
    history: list[tuple[str, str]] = field(default_factory=list)  # (line, reply)


@dataclass
class LineOutcome:
    kind: str  # "reply" | "disconnect" | "error"
    text: str


class Server:
    def __init__(self) -> None:
        self._sessions: dict[int, Session] = {}
        self._next_id = 1

    def connect(self) -> int:
        sid = self._next_id
        self._next_id += 1
        self._sessions[sid] = Session(id=sid)
        return sid

    def session(self, sid: int) -> Session | None:
        return self._sessions.get(sid)

    def authenticated_users(self) -> list[str]:
        return sorted(s.username for s in self._sessions.values()
                      if s.state is State.AUTHENTICATED and s.username)

    def handle_line(self, sid: int, line: str) -> LineOutcome:
        trimmed = line.strip()
        outcome = self._dispatch(sid, trimmed)
        sess = self._sessions.get(sid)
        if sess is not None:
            sess.history.append((trimmed, outcome.text))
            if outcome.kind == "disconnect":
                sess.state = State.DISCONNECTED
        return outcome

    def _dispatch(self, sid: int, line: str) -> LineOutcome:
        if " " in line:
            cmd, rest = line.split(" ", 1)
            rest = rest.strip()
        else:
            cmd, rest = line, ""
        if not cmd:
            return LineOutcome("reply", "")
        sess = self._sessions.get(sid)
        if sess is None:
            return LineOutcome("error", f"unknown session {sid}")
        if sess.state is State.DISCONNECTED:
            return LineOutcome("error", "session is disconnected")

        if cmd == "help":
            return LineOutcome("reply",
                "commands: help, login <name>, whoami, who, echo <text>, quit")
        if cmd == "login":
            if not rest:
                return LineOutcome("error", "usage: login <name>")
            if sess.state is State.AUTHENTICATED:
                return LineOutcome("error", f"already logged in as {sess.username}")
            sess.state = State.AUTHENTICATED
            sess.username = rest
            return LineOutcome("reply", f"welcome, {rest}")
        if cmd == "whoami":
            if sess.state is not State.AUTHENTICATED:
                return LineOutcome("error", "not logged in")
            return LineOutcome("reply", sess.username or "")
        if cmd == "who":
            users = self.authenticated_users()
            return LineOutcome("reply", ", ".join(users) if users else "no one is logged in")
        if cmd == "echo":
            return LineOutcome("reply", rest)
        if cmd == "quit":
            return LineOutcome("disconnect", "goodbye")
        return LineOutcome("error", f"unknown command: {cmd}")


if __name__ == "__main__":
    srv = Server()
    a = srv.connect()
    b = srv.connect()

    transcript = [
        (a, "help"),
        (a, "login alice"),
        (a, "whoami"),
        (b, "who"),            # b still unauth — shows only alice
        (b, "login bob"),
        (a, "who"),            # now shows both
        (a, "echo greetings earthlings"),
        (b, "quit"),
        (a, "who"),            # bob has disconnected
    ]
    for sid, line in transcript:
        outcome = srv.handle_line(sid, line)
        print(f"[{sid}] {line}\n    -> {outcome.kind}: {outcome.text}")
