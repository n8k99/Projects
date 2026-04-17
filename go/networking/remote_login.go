// Remote Login — authenticated session management with credentials and tokens.
package networking

import (
	"crypto/rand"
	"crypto/sha256"
	"encoding/hex"
	"fmt"
	"sync"
	"time"
)

// AuthCredential stores a user's hashed password.
type AuthCredential struct {
	Username     string
	passwordHash string
	salt         string
	Role         string
	Locked       bool
	FailedAttempts int
	LastLogin    *time.Time
}

func newCredential(username, password, role string) *AuthCredential {
	salt := randomHex(16)
	hash := sha256Hex(salt + password)
	return &AuthCredential{Username: username, passwordHash: hash, salt: salt, Role: role}
}

func (c *AuthCredential) verify(password string) bool {
	return sha256Hex(c.salt+password) == c.passwordHash
}

// AuthSession represents an active login session.
type AuthSession struct {
	Token     string
	Username  string
	Role      string
	Created   time.Time
	Expires   time.Time
	IPAddress string
	IsActive  bool
}

// IsValid checks if the session is still usable.
func (s *AuthSession) IsValid() bool {
	return s.IsActive && time.Now().UTC().Before(s.Expires)
}

// AuthLoginAttempt records a login attempt.
type AuthLoginAttempt struct {
	Username  string
	IPAddress string
	Timestamp time.Time
	Success   bool
	Reason    string
}

// AuthLoginResult holds the outcome of a login.
type AuthLoginResult struct {
	Success           bool
	Token             string
	Error             string
	AttemptsRemaining *int
}

// RemoteAuthServer manages authentication and sessions.
type RemoteAuthServer struct {
	mu           sync.Mutex
	SessionTTL   time.Duration
	MaxFailed    int
	LockoutDur   time.Duration
	credentials  map[string]*AuthCredential
	sessions     map[string]*AuthSession
	AuditLog     []AuthLoginAttempt
	lockoutUntil map[string]time.Time
}

// NewRemoteAuthServer creates an auth server.
func NewRemoteAuthServer(sessionTTLMin, maxFailed, lockoutMin int) *RemoteAuthServer {
	return &RemoteAuthServer{
		SessionTTL:   time.Duration(sessionTTLMin) * time.Minute,
		MaxFailed:    maxFailed,
		LockoutDur:   time.Duration(lockoutMin) * time.Minute,
		credentials:  make(map[string]*AuthCredential),
		sessions:     make(map[string]*AuthSession),
		lockoutUntil: make(map[string]time.Time),
	}
}

// Register adds a new user.
func (s *RemoteAuthServer) Register(username, password, role string) bool {
	s.mu.Lock()
	defer s.mu.Unlock()
	if _, ok := s.credentials[username]; ok {
		return false
	}
	s.credentials[username] = newCredential(username, password, role)
	return true
}

// Login authenticates and creates a session.
func (s *RemoteAuthServer) Login(username, password, ip string) AuthLoginResult {
	s.mu.Lock()
	defer s.mu.Unlock()
	now := time.Now().UTC()

	if until, ok := s.lockoutUntil[username]; ok {
		if now.Before(until) {
			s.log(username, ip, false, "account locked")
			return AuthLoginResult{Error: "Account locked"}
		}
		delete(s.lockoutUntil, username)
		if c, ok := s.credentials[username]; ok {
			c.FailedAttempts = 0
		}
	}

	cred, ok := s.credentials[username]
	if !ok {
		s.log(username, ip, false, "unknown user")
		return AuthLoginResult{Error: "Invalid credentials"}
	}
	if cred.Locked {
		s.log(username, ip, false, "account disabled")
		return AuthLoginResult{Error: "Account disabled"}
	}
	if !cred.verify(password) {
		cred.FailedAttempts++
		remaining := s.MaxFailed - cred.FailedAttempts
		if cred.FailedAttempts >= s.MaxFailed {
			s.lockoutUntil[username] = now.Add(s.LockoutDur)
			cred.FailedAttempts = 0
			s.log(username, ip, false, "locked after max attempts")
			zero := 0
			return AuthLoginResult{Error: "Account locked", AttemptsRemaining: &zero}
		}
		s.log(username, ip, false, "wrong password")
		return AuthLoginResult{Error: "Invalid credentials", AttemptsRemaining: &remaining}
	}

	cred.FailedAttempts = 0
	cred.LastLogin = &now
	token := randomHex(32)
	session := &AuthSession{
		Token: token, Username: username, Role: cred.Role,
		Created: now, Expires: now.Add(s.SessionTTL), IPAddress: ip, IsActive: true,
	}
	s.sessions[token] = session
	s.log(username, ip, true, "login successful")
	return AuthLoginResult{Success: true, Token: token}
}

// ValidateSession checks if a token is valid.
func (s *RemoteAuthServer) ValidateSession(token string) *AuthSession {
	s.mu.Lock()
	defer s.mu.Unlock()
	session, ok := s.sessions[token]
	if !ok || !session.IsValid() {
		return nil
	}
	return session
}

// Logout invalidates a session.
func (s *RemoteAuthServer) Logout(token string) bool {
	s.mu.Lock()
	defer s.mu.Unlock()
	if session, ok := s.sessions[token]; ok {
		session.IsActive = false
		return true
	}
	return false
}

// RevokeAll invalidates all sessions for a user.
func (s *RemoteAuthServer) RevokeAll(username string) int {
	s.mu.Lock()
	defer s.mu.Unlock()
	count := 0
	for _, sess := range s.sessions {
		if sess.Username == username && sess.IsActive {
			sess.IsActive = false
			count++
		}
	}
	return count
}

func (s *RemoteAuthServer) log(user, ip string, success bool, reason string) {
	s.AuditLog = append(s.AuditLog, AuthLoginAttempt{
		Username: user, IPAddress: ip, Timestamp: time.Now().UTC(),
		Success: success, Reason: reason,
	})
}

func sha256Hex(s string) string {
	h := sha256.Sum256([]byte(s))
	return hex.EncodeToString(h[:])
}

func randomHex(n int) string {
	b := make([]byte, n)
	rand.Read(b)
	return hex.EncodeToString(b)
}
