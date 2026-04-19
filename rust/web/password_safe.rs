//! Password Safe — encrypted-at-rest credential store with session FSM.
//!
//! **Not a real cryptographic password manager.** The encryption here is a
//! deterministic XOR keystream derived from a byte-sum hash — *demonstrates*
//! the structure of encryption-at-rest (keyed stream, key lives only when
//! unlocked, stored bytes are unreadable without the master) without
//! pretending to be secure. A production version would use Argon2id for
//! key derivation and AES-GCM or ChaCha20-Poly1305 for authenticated
//! encryption.
//!
//! What G077 *does* teach:
//! - Encryption-at-rest as a primitive: data stored in the Safe is unreadable
//!   without a key derived from the master password.
//! - Lock/unlock as an FSM (extends G073's protocol-state pattern).
//! - Auto-lock on inactivity (time-based state transition).
//! - Password generation with a declarative policy.
//! - Password strength scoring as a separate, testable concern.

use std::collections::HashSet;

pub type EntryId = u64;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum State {
    Locked,
    Unlocked { key: Vec<u8>, last_activity_ms: u64 },
}

#[derive(Debug, Clone)]
pub struct Entry {
    pub id: EntryId,
    pub service: String,
    pub username: String,
    encrypted_password: Vec<u8>,
    pub notes: String,
    pub created_at_ms: u64,
    pub modified_at_ms: u64,
}

/// Entry projection for search results — never exposes the password.
#[derive(Debug, Clone)]
pub struct EntrySummary {
    pub id: EntryId,
    pub service: String,
    pub username: String,
    pub notes: String,
    pub created_at_ms: u64,
    pub modified_at_ms: u64,
}

#[derive(Debug, Clone)]
pub struct GeneratePolicy {
    pub length: usize,
    pub lower: bool,
    pub upper: bool,
    pub digits: bool,
    pub symbols: bool,
}

impl Default for GeneratePolicy {
    fn default() -> Self {
        Self { length: 16, lower: true, upper: true, digits: true, symbols: true }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Strength {
    Weak,
    Fair,
    Strong,
    VeryStrong,
}

pub struct Safe {
    entries: Vec<Entry>,
    master_verifier: Vec<u8>,   // hash of master+salt; proves the password on unlock
    salt: Vec<u8>,
    state: State,
    next_id: EntryId,
    pub auto_lock_after_ms: u64,
}

impl Safe {
    /// Create a new safe with the given master password. Safe starts **locked**;
    /// call `unlock` with the master to access operations.
    pub fn new(master: &str, auto_lock_after_ms: u64) -> Self {
        let salt = derive_salt();
        let verifier = verifier_hash(master, &salt);
        Self {
            entries: Vec::new(),
            master_verifier: verifier,
            salt,
            state: State::Locked,
            next_id: 1,
            auto_lock_after_ms,
        }
    }

    pub fn state(&self) -> &State { &self.state }
    pub fn is_locked(&self) -> bool { matches!(self.state, State::Locked) }

    /// Attempt to unlock with the given master password. Returns true on success.
    /// Records current time as last-activity for auto-lock calculations.
    pub fn unlock(&mut self, master: &str, now_ms: u64) -> bool {
        let v = verifier_hash(master, &self.salt);
        if v != self.master_verifier { return false; }
        self.state = State::Unlocked {
            key: derive_key(master, &self.salt),
            last_activity_ms: now_ms,
        };
        true
    }

    pub fn lock(&mut self) { self.state = State::Locked; }

    /// Check whether auto-lock should fire for the current time. Mutates to
    /// Locked if elapsed since last activity exceeds `auto_lock_after_ms`.
    pub fn check_auto_lock(&mut self, now_ms: u64) {
        if let State::Unlocked { last_activity_ms, .. } = &self.state {
            if now_ms >= last_activity_ms + self.auto_lock_after_ms {
                self.lock();
            }
        }
    }

    /// Add a new entry. Requires unlocked state. Returns None if locked.
    pub fn add_entry(&mut self, service: &str, username: &str, password: &str,
                     notes: &str, now_ms: u64) -> Option<EntryId> {
        let key = self.key_and_touch(now_ms)?;
        let id = self.next_id;
        self.next_id += 1;
        self.entries.push(Entry {
            id, service: service.into(), username: username.into(),
            encrypted_password: xor_encrypt(password.as_bytes(), &key),
            notes: notes.into(),
            created_at_ms: now_ms, modified_at_ms: now_ms,
        });
        Some(id)
    }

    /// Retrieve the decrypted password. Requires unlocked state.
    pub fn get_password(&mut self, id: EntryId, now_ms: u64) -> Option<String> {
        let key = self.key_and_touch(now_ms)?;
        let entry = self.entries.iter().find(|e| e.id == id)?;
        let bytes = xor_encrypt(&entry.encrypted_password, &key);
        String::from_utf8(bytes).ok()
    }

    /// Summary of a single entry — NEVER includes the password.
    pub fn get_summary(&self, id: EntryId) -> Option<EntrySummary> {
        self.entries.iter().find(|e| e.id == id).map(|e| EntrySummary {
            id: e.id, service: e.service.clone(), username: e.username.clone(),
            notes: e.notes.clone(),
            created_at_ms: e.created_at_ms, modified_at_ms: e.modified_at_ms,
        })
    }

    /// Remove an entry. Requires unlocked. Returns whether removed.
    pub fn remove_entry(&mut self, id: EntryId, now_ms: u64) -> bool {
        if self.key_and_touch(now_ms).is_none() { return false; }
        match self.entries.iter().position(|e| e.id == id) {
            Some(pos) => { self.entries.remove(pos); true }
            None => false,
        }
    }

    /// Search by substring in service, username, or notes. Requires unlocked.
    /// Never exposes the password — returns summaries only.
    pub fn search(&mut self, query: &str, now_ms: u64) -> Vec<EntrySummary> {
        if self.key_and_touch(now_ms).is_none() { return Vec::new(); }
        let q = query.to_lowercase();
        self.entries.iter().filter(|e| {
            e.service.to_lowercase().contains(&q)
                || e.username.to_lowercase().contains(&q)
                || e.notes.to_lowercase().contains(&q)
        }).map(|e| EntrySummary {
            id: e.id, service: e.service.clone(), username: e.username.clone(),
            notes: e.notes.clone(),
            created_at_ms: e.created_at_ms, modified_at_ms: e.modified_at_ms,
        }).collect()
    }

    pub fn entry_count(&self) -> usize { self.entries.len() }

    /// Return the current key and update last-activity timestamp.
    /// None if locked.
    fn key_and_touch(&mut self, now_ms: u64) -> Option<Vec<u8>> {
        match &mut self.state {
            State::Locked => None,
            State::Unlocked { key, last_activity_ms } => {
                *last_activity_ms = now_ms;
                Some(key.clone())
            }
        }
    }
}

/// Deterministic pseudo-salt derived from bytes of the master length.
/// A real safe would use a CSPRNG-generated salt stored with the safe.
fn derive_salt() -> Vec<u8> {
    // Fixed 16-byte "salt" for deterministic tests. Production must use
    // a fresh random salt per safe.
    (0..16u8).collect()
}

fn verifier_hash(master: &str, salt: &[u8]) -> Vec<u8> {
    // Byte-sum hash, NOT cryptographic. Real code: Argon2id.
    let mut v = Vec::with_capacity(32);
    for round in 0..32u8 {
        let mut h: u64 = round as u64;
        for &b in master.as_bytes() {
            h = h.wrapping_mul(131).wrapping_add(b as u64);
        }
        for &b in salt {
            h = h.wrapping_mul(131).wrapping_add(b as u64);
        }
        v.push((h & 0xff) as u8);
    }
    v
}

fn derive_key(master: &str, salt: &[u8]) -> Vec<u8> {
    // Different from verifier so the verifier doesn't double as the key.
    let mut k = Vec::with_capacity(64);
    for round in 128..192u8 {
        let mut h: u64 = round as u64;
        for &b in master.as_bytes() {
            h = h.wrapping_mul(197).wrapping_add(b as u64);
        }
        for &b in salt {
            h = h.wrapping_mul(197).wrapping_add(b as u64);
        }
        k.push((h & 0xff) as u8);
    }
    k
}

fn xor_encrypt(data: &[u8], key: &[u8]) -> Vec<u8> {
    data.iter().enumerate().map(|(i, &b)| b ^ key[i % key.len()]).collect()
}

/// Deterministic pseudo-random password generator for tests. Real code uses
/// an OS CSPRNG. G077 uses a hash-based stream seeded from now_ms + policy
/// so tests are reproducible.
pub fn generate_password(policy: &GeneratePolicy, seed: u64) -> String {
    let mut alphabet = String::new();
    if policy.lower   { alphabet.push_str("abcdefghijklmnopqrstuvwxyz"); }
    if policy.upper   { alphabet.push_str("ABCDEFGHIJKLMNOPQRSTUVWXYZ"); }
    if policy.digits  { alphabet.push_str("0123456789"); }
    if policy.symbols { alphabet.push_str("!@#$%^&*()-_=+[]{}"); }
    if alphabet.is_empty() { return String::new(); }

    let chars: Vec<char> = alphabet.chars().collect();
    let mut out = String::with_capacity(policy.length);
    let mut state = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
    for _ in 0..policy.length {
        state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        let idx = (state >> 32) as usize % chars.len();
        out.push(chars[idx]);
    }
    out
}

pub fn password_strength(pw: &str) -> Strength {
    let len = pw.chars().count();
    let classes: HashSet<&'static str> = {
        let mut s = HashSet::new();
        if pw.chars().any(|c| c.is_ascii_lowercase()) { s.insert("lower"); }
        if pw.chars().any(|c| c.is_ascii_uppercase()) { s.insert("upper"); }
        if pw.chars().any(|c| c.is_ascii_digit())     { s.insert("digit"); }
        if pw.chars().any(|c| !c.is_ascii_alphanumeric()) { s.insert("symbol"); }
        s
    };
    match (len, classes.len()) {
        (l, _) if l < 8 => Strength::Weak,
        (l, c) if l < 12 && c < 3 => Strength::Weak,
        (l, _) if l >= 20 => Strength::VeryStrong,
        (l, c) if l >= 14 && c >= 3 => Strength::Strong,
        (_, c) if c >= 4 => Strength::Strong,
        _ => Strength::Fair,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_safe_starts_locked() {
        let s = Safe::new("master", 60_000);
        assert!(s.is_locked());
    }

    #[test]
    fn wrong_master_refuses_unlock() {
        let mut s = Safe::new("correct", 60_000);
        assert!(!s.unlock("wrong", 1000));
        assert!(s.is_locked());
    }

    #[test]
    fn correct_master_unlocks() {
        let mut s = Safe::new("master", 60_000);
        assert!(s.unlock("master", 1000));
        assert!(!s.is_locked());
    }

    #[test]
    fn add_requires_unlocked() {
        let mut s = Safe::new("m", 60_000);
        assert!(s.add_entry("svc", "u", "p", "", 1000).is_none());
        s.unlock("m", 1000);
        assert!(s.add_entry("svc", "u", "p", "", 1000).is_some());
    }

    #[test]
    fn password_round_trips_through_encryption() {
        let mut s = Safe::new("m", 60_000);
        s.unlock("m", 1000);
        let id = s.add_entry("example.com", "alice", "s3cret!", "note", 1000).unwrap();
        assert_eq!(s.get_password(id, 2000).unwrap(), "s3cret!");
    }

    #[test]
    fn encrypted_bytes_differ_from_plaintext() {
        let mut s = Safe::new("m", 60_000);
        s.unlock("m", 1000);
        let id = s.add_entry("svc", "u", "hello", "", 1000).unwrap();
        // Peek the encrypted bytes.
        let entry = s.entries.iter().find(|e| e.id == id).unwrap();
        assert_ne!(entry.encrypted_password, b"hello");
    }

    #[test]
    fn summary_never_includes_password() {
        let mut s = Safe::new("m", 60_000);
        s.unlock("m", 1000);
        let id = s.add_entry("svc", "u", "supersecret", "", 1000).unwrap();
        let sum = s.get_summary(id).unwrap();
        let as_json = format!("{sum:?}");
        assert!(!as_json.contains("supersecret"));
    }

    #[test]
    fn get_password_while_locked_returns_none() {
        let mut s = Safe::new("m", 60_000);
        s.unlock("m", 1000);
        let id = s.add_entry("svc", "u", "p", "", 1000).unwrap();
        s.lock();
        assert!(s.get_password(id, 2000).is_none());
    }

    #[test]
    fn auto_lock_fires_after_inactivity() {
        let mut s = Safe::new("m", 1000);  // 1 second auto-lock
        s.unlock("m", 0);
        s.check_auto_lock(500);            // 500ms — still unlocked
        assert!(!s.is_locked());
        s.check_auto_lock(1500);           // > 1000ms — should lock
        assert!(s.is_locked());
    }

    #[test]
    fn activity_resets_auto_lock_timer() {
        let mut s = Safe::new("m", 1000);
        s.unlock("m", 0);
        s.add_entry("svc", "u", "p", "", 800);   // activity at 800
        s.check_auto_lock(1500);                 // 700ms since activity — safe
        assert!(!s.is_locked());
    }

    #[test]
    fn search_filters_substring_case_insensitive() {
        let mut s = Safe::new("m", 60_000);
        s.unlock("m", 1000);
        s.add_entry("GitHub", "alice@example.com", "p1", "", 1000);
        s.add_entry("Gitlab", "bob",  "p2", "code host", 1000);
        s.add_entry("Amazon", "carol", "p3", "", 1000);
        let results = s.search("git", 1000);
        assert_eq!(results.len(), 2);
        let hosts = s.search("host", 1000);
        assert_eq!(hosts.len(), 1);
    }

    #[test]
    fn generate_password_respects_length_and_alphabet() {
        let p = GeneratePolicy { length: 20, lower: true, upper: false,
                                 digits: false, symbols: false };
        let pw = generate_password(&p, 42);
        assert_eq!(pw.chars().count(), 20);
        assert!(pw.chars().all(|c| c.is_ascii_lowercase()));
    }

    #[test]
    fn password_strength_classifies() {
        assert_eq!(password_strength("short"), Strength::Weak);
        assert_eq!(password_strength("longbutonlylower"), Strength::Fair);
        assert_eq!(password_strength("Longer_W1th_Classes"), Strength::Strong);
        assert_eq!(password_strength("AbsurdlyLongPassword_With_Mult1ple_Classes_!@#"),
                   Strength::VeryStrong);
    }
}
