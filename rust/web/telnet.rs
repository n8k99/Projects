//! Telnet Application — line-oriented protocol with state-gated commands.
//!
//! Not a real socket server — the Rosetta Stone focus is the *protocol model*:
//! session state machine, command registry, and line-by-line request/response
//! semantics. Plug this onto a TCP socket in production; test it with raw
//! `handle_line` calls.
//!
//! Each session is an FSM: Unauthenticated → Authenticated (after login) →
//! Disconnected. Commands have a declared set of states they're legal in;
//! calling a command in the wrong state returns an error rather than crashing.
//! This is G062's vending-machine state-gated-operations pattern applied to
//! a protocol, with G067's per-session storage and G021's line-parsing.

use std::collections::HashMap;

pub type SessionId = u64;

#[derive(Debug, Clone)]
pub enum SessionState {
    Unauthenticated,
    Authenticated { username: String },
    Disconnected,
}

#[derive(Debug, Clone)]
pub struct Session {
    pub id: SessionId,
    pub state: SessionState,
    pub history: Vec<(String, String)>,  // (command_line, reply)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LineOutcome {
    Reply(String),
    Disconnect(String),          // farewell reply, then close the session
    Error(String),
}

pub struct Server {
    sessions: HashMap<SessionId, Session>,
    next_id: SessionId,
}

impl Server {
    pub fn new() -> Self { Self { sessions: HashMap::new(), next_id: 1 } }

    pub fn connect(&mut self) -> SessionId {
        let id = self.next_id;
        self.next_id += 1;
        self.sessions.insert(id, Session {
            id, state: SessionState::Unauthenticated, history: Vec::new(),
        });
        id
    }

    pub fn session(&self, id: SessionId) -> Option<&Session> {
        self.sessions.get(&id)
    }

    pub fn authenticated_users(&self) -> Vec<String> {
        let mut names: Vec<String> = self.sessions.values().filter_map(|s|
            match &s.state {
                SessionState::Authenticated { username } => Some(username.clone()),
                _ => None,
            }
        ).collect();
        names.sort();
        names
    }

    /// Process one line on session `id`. Returns the outcome AND mutates the
    /// session's state + history.
    pub fn handle_line(&mut self, id: SessionId, line: &str) -> LineOutcome {
        let trimmed = line.trim();
        let outcome = self.dispatch(id, trimmed);
        // Record history + transition Disconnect → removed session.
        if let Some(session) = self.sessions.get_mut(&id) {
            let reply_text = match &outcome {
                LineOutcome::Reply(r) => r.clone(),
                LineOutcome::Disconnect(r) => r.clone(),
                LineOutcome::Error(e) => e.clone(),
            };
            session.history.push((trimmed.to_string(), reply_text));
            if let LineOutcome::Disconnect(_) = &outcome {
                session.state = SessionState::Disconnected;
            }
        }
        outcome
    }

    fn dispatch(&mut self, id: SessionId, line: &str) -> LineOutcome {
        // Parse: first word is the command, rest is the argument string.
        let (cmd, rest) = match line.find(|c: char| c.is_whitespace()) {
            Some(i) => (&line[..i], line[i..].trim()),
            None => (line, ""),
        };
        if cmd.is_empty() {
            return LineOutcome::Reply(String::new());
        }

        // Look up the session. An unknown id is a hard error — but for a
        // protocol server, that shouldn't happen because we created the id.
        let Some(session) = self.sessions.get(&id) else {
            return LineOutcome::Error(format!("unknown session {id}"));
        };

        if matches!(session.state, SessionState::Disconnected) {
            return LineOutcome::Error("session is disconnected".into());
        }

        match cmd {
            "help" => LineOutcome::Reply(self.cmd_help()),
            "login" => self.cmd_login(id, rest),
            "whoami" => self.cmd_whoami(id),
            "who" => self.cmd_who(),
            "echo" => LineOutcome::Reply(rest.to_string()),
            "quit" => LineOutcome::Disconnect("goodbye".into()),
            other => LineOutcome::Error(format!("unknown command: {other}")),
        }
    }

    fn cmd_help(&self) -> String {
        "commands: help, login <name>, whoami, who, echo <text>, quit".into()
    }

    fn cmd_login(&mut self, id: SessionId, name: &str) -> LineOutcome {
        if name.is_empty() {
            return LineOutcome::Error("usage: login <name>".into());
        }
        let Some(session) = self.sessions.get_mut(&id) else {
            return LineOutcome::Error("no session".into());
        };
        match &session.state {
            SessionState::Unauthenticated => {
                session.state = SessionState::Authenticated { username: name.into() };
                LineOutcome::Reply(format!("welcome, {name}"))
            }
            SessionState::Authenticated { username } => {
                LineOutcome::Error(format!("already logged in as {username}"))
            }
            SessionState::Disconnected => LineOutcome::Error("disconnected".into()),
        }
    }

    fn cmd_whoami(&self, id: SessionId) -> LineOutcome {
        match &self.sessions.get(&id).unwrap().state {
            SessionState::Authenticated { username } => LineOutcome::Reply(username.clone()),
            SessionState::Unauthenticated => LineOutcome::Error("not logged in".into()),
            SessionState::Disconnected => LineOutcome::Error("disconnected".into()),
        }
    }

    fn cmd_who(&self) -> LineOutcome {
        let users = self.authenticated_users();
        if users.is_empty() { LineOutcome::Reply("no one is logged in".into()) }
        else { LineOutcome::Reply(users.join(", ")) }
    }
}

impl Default for Server {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn connect_creates_unauthenticated_session() {
        let mut s = Server::new();
        let id = s.connect();
        assert!(matches!(s.session(id).unwrap().state, SessionState::Unauthenticated));
    }

    #[test]
    fn help_works_in_any_state() {
        let mut s = Server::new();
        let id = s.connect();
        let r = s.handle_line(id, "help");
        assert!(matches!(r, LineOutcome::Reply(_)));
    }

    #[test]
    fn login_authenticates_session() {
        let mut s = Server::new();
        let id = s.connect();
        let r = s.handle_line(id, "login alice");
        assert_eq!(r, LineOutcome::Reply("welcome, alice".into()));
        assert!(matches!(s.session(id).unwrap().state,
                         SessionState::Authenticated { ref username } if username == "alice"));
    }

    #[test]
    fn double_login_rejected() {
        let mut s = Server::new();
        let id = s.connect();
        s.handle_line(id, "login alice");
        let r = s.handle_line(id, "login bob");
        assert!(matches!(r, LineOutcome::Error(_)));
    }

    #[test]
    fn whoami_requires_auth() {
        let mut s = Server::new();
        let id = s.connect();
        assert!(matches!(s.handle_line(id, "whoami"), LineOutcome::Error(_)));
        s.handle_line(id, "login alice");
        assert_eq!(s.handle_line(id, "whoami"), LineOutcome::Reply("alice".into()));
    }

    #[test]
    fn who_lists_authenticated_users_across_sessions() {
        let mut s = Server::new();
        let a = s.connect(); let b = s.connect(); let c = s.connect();
        s.handle_line(a, "login alice");
        s.handle_line(b, "login bob");
        // c stays unauthenticated; should not appear.
        let reply = match s.handle_line(a, "who") {
            LineOutcome::Reply(r) => r, _ => panic!(),
        };
        assert!(reply.contains("alice"));
        assert!(reply.contains("bob"));
        assert_eq!(s.session(c).unwrap().history.len(), 0);
    }

    #[test]
    fn echo_returns_argument() {
        let mut s = Server::new();
        let id = s.connect();
        let r = s.handle_line(id, "echo hello world");
        assert_eq!(r, LineOutcome::Reply("hello world".into()));
    }

    #[test]
    fn quit_disconnects_and_records_history() {
        let mut s = Server::new();
        let id = s.connect();
        s.handle_line(id, "login alice");
        let r = s.handle_line(id, "quit");
        assert!(matches!(r, LineOutcome::Disconnect(_)));
        assert!(matches!(s.session(id).unwrap().state, SessionState::Disconnected));
        // History contains every command issued on this session.
        let history = &s.session(id).unwrap().history;
        assert_eq!(history.len(), 2);
        assert_eq!(history[0].0, "login alice");
        assert_eq!(history[1].0, "quit");
    }

    #[test]
    fn commands_on_disconnected_session_are_rejected() {
        let mut s = Server::new();
        let id = s.connect();
        s.handle_line(id, "quit");
        let r = s.handle_line(id, "help");
        assert!(matches!(r, LineOutcome::Error(_)));
    }

    #[test]
    fn unknown_command_returns_error() {
        let mut s = Server::new();
        let id = s.connect();
        let r = s.handle_line(id, "frobnicate");
        assert!(matches!(r, LineOutcome::Error(_)));
    }

    #[test]
    fn empty_line_returns_empty_reply() {
        let mut s = Server::new();
        let id = s.connect();
        let r = s.handle_line(id, "   ");
        assert_eq!(r, LineOutcome::Reply(String::new()));
    }

    #[test]
    fn sessions_are_isolated() {
        // Alice's state changes must not affect Bob's session.
        let mut s = Server::new();
        let a = s.connect(); let b = s.connect();
        s.handle_line(a, "login alice");
        s.handle_line(a, "quit");
        // Bob is still unauthenticated and connected.
        assert!(matches!(s.session(b).unwrap().state, SessionState::Unauthenticated));
        let r = s.handle_line(b, "help");
        assert!(matches!(r, LineOutcome::Reply(_)));
    }
}
