"""Remote Login — authenticated session management with credentials and tokens."""

from dataclasses import dataclass, field
from datetime import datetime, timezone, timedelta
from typing import Optional, Callable
import hashlib
import secrets


@dataclass
class Credential:
    """Stored credential: username + salted password hash."""
    username: str
    password_hash: str
    salt: str
    role: str = "user"
    locked: bool = False
    failed_attempts: int = 0
    last_login: Optional[datetime] = None

    @staticmethod
    def create(username: str, password: str, role: str = "user") -> "Credential":
        salt = secrets.token_hex(16)
        h = hashlib.sha256((salt + password).encode()).hexdigest()
        return Credential(username=username, password_hash=h, salt=salt, role=role)

    def verify(self, password: str) -> bool:
        h = hashlib.sha256((self.salt + password).encode()).hexdigest()
        return h == self.password_hash


@dataclass
class Session:
    """An authenticated session with a token and expiry."""
    token: str
    username: str
    role: str
    created: datetime
    expires: datetime
    ip_address: str = ""
    is_active: bool = True

    @property
    def is_expired(self) -> bool:
        return datetime.now(timezone.utc) > self.expires

    @property
    def is_valid(self) -> bool:
        return self.is_active and not self.is_expired

    @property
    def remaining_seconds(self) -> float:
        return max(0.0, (self.expires - datetime.now(timezone.utc)).total_seconds())


@dataclass
class LoginAttempt:
    """Record of a login attempt."""
    username: str
    ip_address: str
    timestamp: datetime
    success: bool
    reason: str = ""


@dataclass
class LoginResult:
    """Result of a login operation."""
    success: bool
    session: Optional[Session] = None
    error: str = ""
    attempts_remaining: Optional[int] = None


class AuthServer:
    """Authentication server managing credentials, sessions, and login policy.

    Implements: credential storage with salted hashes, token-based sessions,
    brute-force lockout, session expiry, and audit logging.
    """

    def __init__(self, session_ttl_minutes: int = 30, max_failed_attempts: int = 5,
                 lockout_minutes: int = 15) -> None:
        self.session_ttl = timedelta(minutes=session_ttl_minutes)
        self.max_failed = max_failed_attempts
        self.lockout_duration = timedelta(minutes=lockout_minutes)
        self.credentials: dict[str, Credential] = {}
        self.sessions: dict[str, Session] = {}
        self.audit_log: list[LoginAttempt] = []
        self._lockout_until: dict[str, datetime] = {}

    def register(self, username: str, password: str, role: str = "user") -> bool:
        """Register a new user."""
        if username in self.credentials:
            return False
        self.credentials[username] = Credential.create(username, password, role)
        return True

    def login(self, username: str, password: str, ip_address: str = "") -> LoginResult:
        """Authenticate and create a session."""
        now = datetime.now(timezone.utc)

        # Check lockout
        if username in self._lockout_until:
            if now < self._lockout_until[username]:
                self._log(username, ip_address, False, "account locked")
                remaining = (self._lockout_until[username] - now).seconds
                return LoginResult(success=False, error=f"Account locked. Try again in {remaining}s")
            else:
                del self._lockout_until[username]
                if username in self.credentials:
                    self.credentials[username].failed_attempts = 0

        # Check credentials exist
        cred = self.credentials.get(username)
        if cred is None:
            self._log(username, ip_address, False, "unknown user")
            return LoginResult(success=False, error="Invalid credentials")

        # Check locked
        if cred.locked:
            self._log(username, ip_address, False, "account disabled")
            return LoginResult(success=False, error="Account disabled")

        # Verify password
        if not cred.verify(password):
            cred.failed_attempts += 1
            remaining = self.max_failed - cred.failed_attempts
            if cred.failed_attempts >= self.max_failed:
                self._lockout_until[username] = now + self.lockout_duration
                cred.failed_attempts = 0
                self._log(username, ip_address, False, "locked after max attempts")
                return LoginResult(success=False, error="Account locked due to failed attempts",
                                   attempts_remaining=0)
            self._log(username, ip_address, False, "wrong password")
            return LoginResult(success=False, error="Invalid credentials",
                               attempts_remaining=remaining)

        # Success — create session
        cred.failed_attempts = 0
        cred.last_login = now
        token = secrets.token_hex(32)
        session = Session(
            token=token, username=username, role=cred.role,
            created=now, expires=now + self.session_ttl, ip_address=ip_address,
        )
        self.sessions[token] = session
        self._log(username, ip_address, True, "login successful")
        return LoginResult(success=True, session=session)

    def validate_session(self, token: str) -> Optional[Session]:
        """Validate a session token. Returns the session if valid, None otherwise."""
        session = self.sessions.get(token)
        if session is None:
            return None
        if not session.is_valid:
            session.is_active = False
            return None
        return session

    def logout(self, token: str) -> bool:
        """Invalidate a session."""
        session = self.sessions.get(token)
        if session is None:
            return False
        session.is_active = False
        return True

    def refresh_session(self, token: str) -> Optional[Session]:
        """Extend a valid session's expiry."""
        session = self.validate_session(token)
        if session is None:
            return None
        session.expires = datetime.now(timezone.utc) + self.session_ttl
        return session

    def active_sessions(self, username: Optional[str] = None) -> list[Session]:
        """List active sessions, optionally filtered by username."""
        sessions = [s for s in self.sessions.values() if s.is_valid]
        if username:
            sessions = [s for s in sessions if s.username == username]
        return sessions

    def revoke_all(self, username: str) -> int:
        """Revoke all sessions for a user. Returns count revoked."""
        count = 0
        for s in self.sessions.values():
            if s.username == username and s.is_active:
                s.is_active = False
                count += 1
        return count

    def change_password(self, username: str, old_password: str, new_password: str) -> bool:
        """Change a user's password. Revokes all existing sessions."""
        cred = self.credentials.get(username)
        if cred is None or not cred.verify(old_password):
            return False
        new_cred = Credential.create(username, new_password, cred.role)
        new_cred.last_login = cred.last_login
        self.credentials[username] = new_cred
        self.revoke_all(username)
        return True

    def _log(self, username: str, ip: str, success: bool, reason: str) -> None:
        self.audit_log.append(LoginAttempt(
            username=username, ip_address=ip,
            timestamp=datetime.now(timezone.utc),
            success=success, reason=reason,
        ))

    def failed_attempts_from_ip(self, ip_address: str, window_minutes: int = 60) -> int:
        """Count failed attempts from an IP within a time window."""
        cutoff = datetime.now(timezone.utc) - timedelta(minutes=window_minutes)
        return sum(1 for a in self.audit_log
                   if a.ip_address == ip_address and not a.success and a.timestamp > cutoff)

    def stats(self) -> dict:
        return {
            "users": len(self.credentials),
            "active_sessions": len(self.active_sessions()),
            "total_sessions": len(self.sessions),
            "audit_entries": len(self.audit_log),
        }


# --- demo ---
if __name__ == "__main__":
    server = AuthServer(session_ttl_minutes=30, max_failed_attempts=3)

    server.register("nathan", "rosetta2026", role="admin")
    server.register("kathryn", "finance!", role="exec")

    # Successful login
    result = server.login("nathan", "rosetta2026", ip_address="127.0.0.1")
    print(f"Login: {result.success}, token: {result.session.token[:16]}...")

    # Validate session
    session = server.validate_session(result.session.token)
    print(f"Session valid: {session is not None}, role: {session.role}")

    # Failed login
    result2 = server.login("nathan", "wrong", ip_address="10.0.0.1")
    print(f"Bad login: {result2.success}, error: {result2.error}")

    print(f"Stats: {server.stats()}")
