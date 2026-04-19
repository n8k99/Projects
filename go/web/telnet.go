// Telnet Application — line-oriented protocol with state-gated commands.
package web

import (
	"fmt"
	"sort"
	"strings"
)

// TelnetState is a session's FSM state.
type TelnetState int

const (
	TelnetUnauthenticated TelnetState = iota
	TelnetAuthenticated
	TelnetDisconnected
)

// TelnetSession tracks one client's protocol state.
type TelnetSession struct {
	ID       int64
	State    TelnetState
	Username string
	History  []TelnetEntry
}

// TelnetEntry is one line/reply pair in the session history.
type TelnetEntry struct{ Line, Reply string }

// TelnetOutcomeKind is reply / disconnect / error.
type TelnetOutcomeKind int

const (
	TelnetReply TelnetOutcomeKind = iota
	TelnetDisconnect
	TelnetError
)

// TelnetOutcome is the result of processing one line.
type TelnetOutcome struct {
	Kind TelnetOutcomeKind
	Text string
}

// TelnetServer holds all sessions.
type TelnetServer struct {
	sessions map[int64]*TelnetSession
	nextID   int64
}

// NewTelnetServer returns an empty server.
func NewTelnetServer() *TelnetServer {
	return &TelnetServer{sessions: map[int64]*TelnetSession{}, nextID: 1}
}

// Connect creates a new unauthenticated session.
func (s *TelnetServer) Connect() int64 {
	id := s.nextID
	s.nextID++
	s.sessions[id] = &TelnetSession{ID: id}
	return id
}

// Session returns the session for id, or nil.
func (s *TelnetServer) Session(id int64) *TelnetSession { return s.sessions[id] }

// AuthenticatedUsers returns sorted usernames of authenticated sessions.
func (s *TelnetServer) AuthenticatedUsers() []string {
	var out []string
	for _, sess := range s.sessions {
		if sess.State == TelnetAuthenticated {
			out = append(out, sess.Username)
		}
	}
	sort.Strings(out)
	return out
}

// HandleLine processes one line on the given session.
func (s *TelnetServer) HandleLine(id int64, line string) TelnetOutcome {
	trimmed := strings.TrimSpace(line)
	out := s.dispatch(id, trimmed)
	if sess := s.sessions[id]; sess != nil {
		sess.History = append(sess.History, TelnetEntry{Line: trimmed, Reply: out.Text})
		if out.Kind == TelnetDisconnect {
			sess.State = TelnetDisconnected
		}
	}
	return out
}

func (s *TelnetServer) dispatch(id int64, line string) TelnetOutcome {
	var cmd, rest string
	if i := strings.IndexAny(line, " \t"); i >= 0 {
		cmd = line[:i]
		rest = strings.TrimSpace(line[i+1:])
	} else {
		cmd = line
	}
	if cmd == "" {
		return TelnetOutcome{Kind: TelnetReply, Text: ""}
	}
	sess := s.sessions[id]
	if sess == nil {
		return TelnetOutcome{Kind: TelnetError, Text: fmt.Sprintf("unknown session %d", id)}
	}
	if sess.State == TelnetDisconnected {
		return TelnetOutcome{Kind: TelnetError, Text: "session is disconnected"}
	}

	switch cmd {
	case "help":
		return TelnetOutcome{Kind: TelnetReply,
			Text: "commands: help, login <name>, whoami, who, echo <text>, quit"}
	case "login":
		if rest == "" {
			return TelnetOutcome{Kind: TelnetError, Text: "usage: login <name>"}
		}
		if sess.State == TelnetAuthenticated {
			return TelnetOutcome{Kind: TelnetError,
				Text: fmt.Sprintf("already logged in as %s", sess.Username)}
		}
		sess.State = TelnetAuthenticated
		sess.Username = rest
		return TelnetOutcome{Kind: TelnetReply, Text: fmt.Sprintf("welcome, %s", rest)}
	case "whoami":
		if sess.State != TelnetAuthenticated {
			return TelnetOutcome{Kind: TelnetError, Text: "not logged in"}
		}
		return TelnetOutcome{Kind: TelnetReply, Text: sess.Username}
	case "who":
		users := s.AuthenticatedUsers()
		if len(users) == 0 {
			return TelnetOutcome{Kind: TelnetReply, Text: "no one is logged in"}
		}
		return TelnetOutcome{Kind: TelnetReply, Text: strings.Join(users, ", ")}
	case "echo":
		return TelnetOutcome{Kind: TelnetReply, Text: rest}
	case "quit":
		return TelnetOutcome{Kind: TelnetDisconnect, Text: "goodbye"}
	default:
		return TelnetOutcome{Kind: TelnetError, Text: "unknown command: " + cmd}
	}
}
