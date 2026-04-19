-- Telnet Application — line-oriented protocol with state-gated commands.
-- Pure-functional: server state threads through `handleLine`.

namespace Web.Telnet

inductive State where
  | unauthenticated
  | authenticated
  | disconnected
  deriving Repr, DecidableEq, BEq

structure Session where
  id : Nat
  state : State := .unauthenticated
  username : String := ""
  history : List (String × String) := []
  deriving Repr

inductive OutcomeKind where
  | reply | disconnect | error
  deriving Repr, DecidableEq

structure Outcome where
  kind : OutcomeKind
  text : String
  deriving Repr

structure Server where
  sessions : List Session := []
  nextId : Nat := 1
  deriving Repr

def Server.empty : Server := {}

def Server.connect (s : Server) : Server × Nat :=
  let id := s.nextId
  let sess : Session := { id }
  ({ s with sessions := s.sessions ++ [sess], nextId := id + 1 }, id)

def Server.session (s : Server) (id : Nat) : Option Session :=
  s.sessions.find? (·.id == id)

def Server.authenticatedUsers (s : Server) : List String :=
  (s.sessions.filter (·.state == .authenticated)).map (·.username) |>.mergeSort (· < ·)

private def splitCmd (line : String) : String × String :=
  match line.find Char.isWhitespace with
  | none => (line, "")
  | some idx =>
    let cmd := line.extract 0 idx
    let rest := (line.extract (idx + ⟨1⟩) line.endPos).trim
    (cmd, rest)

private def updateSession (s : Server) (id : Nat) (f : Session → Session) : Server :=
  { s with sessions := s.sessions.map fun sess =>
    if sess.id == id then f sess else sess }

def Server.handleLine (s : Server) (id : Nat) (rawLine : String)
    : Server × Outcome :=
  let line := rawLine.trim
  let (cmd, rest) := splitCmd line
  let (server', outcome) :=
    match s.session id with
    | none => (s, { kind := .error, text := s!"unknown session {id}" : Outcome })
    | some sess =>
      if sess.state == .disconnected then
        (s, { kind := .error, text := "session is disconnected" : Outcome })
      else if cmd.isEmpty then
        (s, { kind := .reply, text := "" : Outcome })
      else
        match cmd with
        | "help" =>
          (s, { kind := .reply,
                text := "commands: help, login <name>, whoami, who, echo <text>, quit" : Outcome })
        | "login" =>
          if rest.isEmpty then
            (s, { kind := .error, text := "usage: login <name>" : Outcome })
          else if sess.state == .authenticated then
            (s, { kind := .error, text := s!"already logged in as {sess.username}" : Outcome })
          else
            let s' := updateSession s id fun u =>
              { u with state := .authenticated, username := rest }
            (s', { kind := .reply, text := s!"welcome, {rest}" : Outcome })
        | "whoami" =>
          if sess.state == .authenticated then
            (s, { kind := .reply, text := sess.username : Outcome })
          else
            (s, { kind := .error, text := "not logged in" : Outcome })
        | "who" =>
          let users := s.authenticatedUsers
          let txt := if users.isEmpty then "no one is logged in" else
                     String.intercalate ", " users
          (s, { kind := .reply, text := txt : Outcome })
        | "echo" =>
          (s, { kind := .reply, text := rest : Outcome })
        | "quit" =>
          (s, { kind := .disconnect, text := "goodbye" : Outcome })
        | _ =>
          (s, { kind := .error, text := s!"unknown command: {cmd}" : Outcome })
  -- Record history; flip to disconnected on quit.
  let server'' := updateSession server' id fun u =>
    let history' := u.history ++ [(line, outcome.text)]
    let state' := if outcome.kind == .disconnect then State.disconnected else u.state
    { u with history := history', state := state' }
  (server'', outcome)

end Web.Telnet
