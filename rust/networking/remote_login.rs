// Remote Login — authenticated session management with credentials and tokens.

use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

fn sha256_hex(data: &[u8]) -> String {
    // Reuse the SHA-256 from p2p_file_sharing
    super::p2p_file_sharing::sha256_simple(data)
}

fn random_hex(len: usize) -> String {
    // Use system time + counter for unique tokens (not cryptographic, but deterministic-free)
    let seed = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    let mut state = seed;
    let mut out = String::new();
    for _ in 0..len {
        state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        out.push_str(&format!("{:02x}", (state >> 33) as u8));
    }
    out.truncate(len * 2);
    out
}

fn now_secs() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()
}

#[derive(Debug, Clone)]
pub struct Credential {
    pub username: String,
    password_hash: String,
    salt: String,
    pub role: String,
    pub locked: bool,
    pub failed_attempts: u32,
    pub last_login: Option<u64>,
}

impl Credential {
    pub fn create(username: &str, password: &str, role: &str) -> Self {
        let salt = random_hex(16);
        let hash = sha256_hex(format!("{}{}", salt, password).as_bytes());
        Credential {
            username: username.to_string(), password_hash: hash, salt,
            role: role.to_string(), locked: false, failed_attempts: 0, last_login: None,
        }
    }

    pub fn verify(&self, password: &str) -> bool {
        let h = sha256_hex(format!("{}{}", self.salt, password).as_bytes());
        h == self.password_hash
    }
}

#[derive(Debug, Clone)]
pub struct Session {
    pub token: String,
    pub username: String,
    pub role: String,
    pub created: u64,
    pub expires: u64,
    pub ip_address: String,
    pub is_active: bool,
}

impl Session {
    pub fn is_expired(&self) -> bool { now_secs() > self.expires }
    pub fn is_valid(&self) -> bool { self.is_active && !self.is_expired() }
}

#[derive(Debug, Clone)]
pub struct LoginAttempt {
    pub username: String,
    pub ip_address: String,
    pub timestamp: u64,
    pub success: bool,
    pub reason: String,
}

#[derive(Debug)]
pub struct LoginResult {
    pub success: bool,
    pub token: Option<String>,
    pub error: String,
    pub attempts_remaining: Option<u32>,
}

pub struct AuthServer {
    session_ttl_secs: u64,
    max_failed: u32,
    lockout_secs: u64,
    pub credentials: HashMap<String, Credential>,
    pub sessions: HashMap<String, Session>,
    pub audit_log: Vec<LoginAttempt>,
    lockout_until: HashMap<String, u64>,
}

impl AuthServer {
    pub fn new(session_ttl_minutes: u64, max_failed: u32, lockout_minutes: u64) -> Self {
        AuthServer {
            session_ttl_secs: session_ttl_minutes * 60,
            max_failed,
            lockout_secs: lockout_minutes * 60,
            credentials: HashMap::new(),
            sessions: HashMap::new(),
            audit_log: Vec::new(),
            lockout_until: HashMap::new(),
        }
    }

    pub fn register(&mut self, username: &str, password: &str, role: &str) -> bool {
        if self.credentials.contains_key(username) { return false; }
        self.credentials.insert(username.to_string(), Credential::create(username, password, role));
        true
    }

    pub fn login(&mut self, username: &str, password: &str, ip: &str) -> LoginResult {
        let now = now_secs();

        // Check lockout
        if let Some(&until) = self.lockout_until.get(username) {
            if now < until {
                self.log(username, ip, false, "account locked");
                return LoginResult { success: false, token: None,
                    error: "Account locked".into(), attempts_remaining: Some(0) };
            } else {
                self.lockout_until.remove(username);
                if let Some(c) = self.credentials.get_mut(username) { c.failed_attempts = 0; }
            }
        }

        let cred = match self.credentials.get_mut(username) {
            Some(c) => c,
            None => {
                self.log(username, ip, false, "unknown user");
                return LoginResult { success: false, token: None,
                    error: "Invalid credentials".into(), attempts_remaining: None };
            }
        };

        if cred.locked {
            self.log(username, ip, false, "account disabled");
            return LoginResult { success: false, token: None,
                error: "Account disabled".into(), attempts_remaining: None };
        }

        if !cred.verify(password) {
            cred.failed_attempts += 1;
            let remaining = self.max_failed.saturating_sub(cred.failed_attempts);
            if cred.failed_attempts >= self.max_failed {
                self.lockout_until.insert(username.to_string(), now + self.lockout_secs);
                cred.failed_attempts = 0;
                self.log(username, ip, false, "locked after max attempts");
                return LoginResult { success: false, token: None,
                    error: "Account locked".into(), attempts_remaining: Some(0) };
            }
            self.log(username, ip, false, "wrong password");
            return LoginResult { success: false, token: None,
                error: "Invalid credentials".into(), attempts_remaining: Some(remaining) };
        }

        cred.failed_attempts = 0;
        cred.last_login = Some(now);
        let token = random_hex(32);
        let session = Session {
            token: token.clone(), username: username.to_string(),
            role: cred.role.clone(), created: now,
            expires: now + self.session_ttl_secs,
            ip_address: ip.to_string(), is_active: true,
        };
        self.sessions.insert(token.clone(), session);
        self.log(username, ip, true, "login successful");
        LoginResult { success: true, token: Some(token), error: String::new(), attempts_remaining: None }
    }

    pub fn validate_session(&mut self, token: &str) -> Option<&Session> {
        let session = self.sessions.get_mut(token)?;
        if !session.is_valid() {
            session.is_active = false;
            return None;
        }
        Some(session)
    }

    pub fn logout(&mut self, token: &str) -> bool {
        if let Some(s) = self.sessions.get_mut(token) {
            s.is_active = false;
            true
        } else {
            false
        }
    }

    pub fn refresh_session(&mut self, token: &str) -> bool {
        let ttl = self.session_ttl_secs;
        if let Some(s) = self.sessions.get_mut(token) {
            if s.is_valid() {
                s.expires = now_secs() + ttl;
                return true;
            }
        }
        false
    }

    pub fn active_sessions(&self) -> Vec<&Session> {
        self.sessions.values().filter(|s| s.is_valid()).collect()
    }

    pub fn revoke_all(&mut self, username: &str) -> usize {
        let mut count = 0;
        for s in self.sessions.values_mut() {
            if s.username == username && s.is_active {
                s.is_active = false;
                count += 1;
            }
        }
        count
    }

    fn log(&mut self, username: &str, ip: &str, success: bool, reason: &str) {
        self.audit_log.push(LoginAttempt {
            username: username.to_string(), ip_address: ip.to_string(),
            timestamp: now_secs(), success, reason: reason.to_string(),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_credential_create_verify() {
        let cred = Credential::create("alice", "pass123", "user");
        assert!(cred.verify("pass123"));
        assert!(!cred.verify("wrong"));
    }

    #[test]
    fn test_register_and_login() {
        let mut server = AuthServer::new(30, 5, 15);
        assert!(server.register("alice", "pass123", "user"));
        assert!(!server.register("alice", "other", "user")); // duplicate

        let result = server.login("alice", "pass123", "127.0.0.1");
        assert!(result.success);
        assert!(result.token.is_some());
    }

    #[test]
    fn test_session_validation() {
        let mut server = AuthServer::new(30, 5, 15);
        server.register("alice", "pass123", "user");
        let result = server.login("alice", "pass123", "");
        let token = result.token.unwrap();

        let session = server.validate_session(&token);
        assert!(session.is_some());
        assert_eq!(session.unwrap().username, "alice");
    }

    #[test]
    fn test_logout() {
        let mut server = AuthServer::new(30, 5, 15);
        server.register("alice", "pass123", "user");
        let token = server.login("alice", "pass123", "").token.unwrap();

        assert!(server.logout(&token));
        assert!(server.validate_session(&token).is_none());
    }

    #[test]
    fn test_wrong_password_tracking() {
        let mut server = AuthServer::new(30, 3, 15);
        server.register("alice", "pass123", "user");

        let r = server.login("alice", "wrong", "");
        assert!(!r.success);
        assert_eq!(r.attempts_remaining, Some(2));

        let r = server.login("alice", "wrong", "");
        assert_eq!(r.attempts_remaining, Some(1));

        let r = server.login("alice", "wrong", "");
        assert_eq!(r.attempts_remaining, Some(0));
        assert!(r.error.contains("locked"));

        // Locked out — even correct password fails
        let r = server.login("alice", "pass123", "");
        assert!(!r.success);
    }

    #[test]
    fn test_unknown_user() {
        let mut server = AuthServer::new(30, 5, 15);
        let r = server.login("nobody", "pass", "");
        assert!(!r.success);
        assert!(r.error.contains("Invalid"));
    }

    #[test]
    fn test_revoke_all() {
        let mut server = AuthServer::new(30, 5, 15);
        server.register("alice", "pass123", "user");
        server.login("alice", "pass123", "");
        server.login("alice", "pass123", "");
        assert_eq!(server.active_sessions().len(), 2);

        let revoked = server.revoke_all("alice");
        assert_eq!(revoked, 2);
        assert_eq!(server.active_sessions().len(), 0);
    }

    #[test]
    fn test_audit_log() {
        let mut server = AuthServer::new(30, 5, 15);
        server.register("alice", "pass123", "user");
        server.login("alice", "pass123", "10.0.0.1");
        server.login("alice", "wrong", "10.0.0.2");

        assert_eq!(server.audit_log.len(), 2);
        assert!(server.audit_log[0].success);
        assert!(!server.audit_log[1].success);
    }
}
