//! CAPTCHA Maker — challenge-response with expiration and attempt limits.
//!
//! Closes the Web category. A CAPTCHA is an ephemeral, single-use, time-
//! bounded challenge: issue a prompt (with a known expected answer),
//! accept answers for a limited window with a bounded retry count, refuse
//! after solved / expired / exhausted. The primitive is small but
//! combines several patterns from earlier Web projects:
//!
//! - Time-bounded validity (G077 auto-lock, G082 scheduled publish).
//! - Attempt counters with refusal (G067 room-scope rate limiting).
//! - Outcome-typed results for UI dispatch (G083 validation issues).
//! - Natural short-lived identity (the challenge id is the handle).

pub type ChallengeId = u64;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChallengeKind {
    Math,            // "3 + 5 = ?"
    WordRecall,      // "type: butterfly"
    ReverseString,   // "reverse: hello"
    SequenceCount,   // "count digits in: AB3C42"
}

#[derive(Debug, Clone)]
pub struct Challenge {
    pub id: ChallengeId,
    pub kind: ChallengeKind,
    pub prompt: String,
    pub expected: String,
    pub created_at_ms: u64,
    pub expires_at_ms: u64,
    pub attempts_used: u32,
    pub max_attempts: u32,
    pub solved: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VerifyResult {
    Success,
    WrongAnswer { attempts_remaining: u32 },
    Expired,
    AlreadySolved,
    TooManyAttempts,
    Unknown,
}

pub struct CaptchaService {
    challenges: Vec<Challenge>,
    next_id: ChallengeId,
    default_ttl_ms: u64,
    max_attempts: u32,
    /// Deterministic RNG state so the service is testable.
    rng_state: u64,
}

impl CaptchaService {
    pub fn new(default_ttl_ms: u64, max_attempts: u32, seed: u64) -> Self {
        Self {
            challenges: Vec::new(),
            next_id: 1,
            default_ttl_ms,
            max_attempts,
            rng_state: seed,
        }
    }

    /// Issue a new challenge of the given kind. Returns the full challenge
    /// record (including the expected answer — real production would NOT
    /// return the expected answer; here we do for testability).
    pub fn issue(&mut self, kind: ChallengeKind, now_ms: u64) -> Challenge {
        let id = self.next_id;
        self.next_id += 1;
        let (prompt, expected) = self.generate(kind);
        let challenge = Challenge {
            id, kind, prompt, expected,
            created_at_ms: now_ms,
            expires_at_ms: now_ms + self.default_ttl_ms,
            attempts_used: 0,
            max_attempts: self.max_attempts,
            solved: false,
        };
        self.challenges.push(challenge.clone());
        challenge
    }

    /// Verify a user's answer to a challenge.
    pub fn verify(&mut self, id: ChallengeId, user_answer: &str, now_ms: u64)
        -> VerifyResult {
        let Some(c) = self.challenges.iter_mut().find(|c| c.id == id) else {
            return VerifyResult::Unknown;
        };
        if c.solved {
            return VerifyResult::AlreadySolved;
        }
        if now_ms > c.expires_at_ms {
            return VerifyResult::Expired;
        }
        if c.attempts_used >= c.max_attempts {
            return VerifyResult::TooManyAttempts;
        }
        c.attempts_used += 1;
        if answers_match(&c.expected, user_answer) {
            c.solved = true;
            VerifyResult::Success
        } else {
            let remaining = c.max_attempts.saturating_sub(c.attempts_used);
            VerifyResult::WrongAnswer { attempts_remaining: remaining }
        }
    }

    pub fn challenge(&self, id: ChallengeId) -> Option<&Challenge> {
        self.challenges.iter().find(|c| c.id == id)
    }

    /// Remove challenges that have expired or been solved; returns count removed.
    pub fn cleanup(&mut self, now_ms: u64) -> usize {
        let before = self.challenges.len();
        self.challenges.retain(|c| !c.solved && now_ms <= c.expires_at_ms);
        before - self.challenges.len()
    }

    pub fn active_count(&self) -> usize { self.challenges.len() }

    fn next_random(&mut self) -> u64 {
        self.rng_state = self.rng_state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        self.rng_state >> 32
    }

    fn generate(&mut self, kind: ChallengeKind) -> (String, String) {
        match kind {
            ChallengeKind::Math => {
                let a = (self.next_random() % 20 + 1) as u32;
                let b = (self.next_random() % 20 + 1) as u32;
                let op = self.next_random() % 3;
                let (prompt, answer) = match op {
                    0 => (format!("{a} + {b}"), (a + b).to_string()),
                    1 => {
                        let (hi, lo) = if a >= b { (a, b) } else { (b, a) };
                        (format!("{hi} - {lo}"), (hi - lo).to_string())
                    }
                    _ => (format!("{a} × {b}"), (a * b).to_string()),
                };
                (format!("What is {prompt}?"), answer)
            }
            ChallengeKind::WordRecall => {
                let words = ["butterfly", "chronicle", "lighthouse", "symphony", "mountain"];
                let idx = (self.next_random() as usize) % words.len();
                let w = words[idx];
                (format!("Type this word: {w}"), w.to_string())
            }
            ChallengeKind::ReverseString => {
                let inputs = ["hello", "rosetta", "captcha", "innate"];
                let idx = (self.next_random() as usize) % inputs.len();
                let w = inputs[idx];
                let reversed: String = w.chars().rev().collect();
                (format!("Reverse this: {w}"), reversed)
            }
            ChallengeKind::SequenceCount => {
                // A short mixed sequence; count the digits.
                let len = (self.next_random() % 6 + 5) as usize;
                let mut seq = String::with_capacity(len);
                let mut digit_count = 0u32;
                for _ in 0..len {
                    let r = self.next_random() % 36;
                    if r < 10 {
                        seq.push((b'0' + r as u8) as char);
                        digit_count += 1;
                    } else {
                        seq.push((b'A' + (r - 10) as u8) as char);
                    }
                }
                (format!("How many digits in: {seq}?"), digit_count.to_string())
            }
        }
    }
}

/// Answers match if they're equal after trimming + lowercasing.
fn answers_match(expected: &str, user: &str) -> bool {
    expected.trim().to_lowercase() == user.trim().to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn issued_challenge_has_expected_shape() {
        let mut s = CaptchaService::new(60_000, 3, 42);
        let c = s.issue(ChallengeKind::Math, 1000);
        assert!(c.prompt.contains("What is"));
        assert!(!c.solved);
        assert_eq!(c.attempts_used, 0);
        assert_eq!(c.max_attempts, 3);
        assert_eq!(c.expires_at_ms, 1000 + 60_000);
    }

    #[test]
    fn correct_answer_succeeds() {
        let mut s = CaptchaService::new(60_000, 3, 7);
        let c = s.issue(ChallengeKind::Math, 1000);
        let result = s.verify(c.id, &c.expected, 2000);
        assert_eq!(result, VerifyResult::Success);
        assert!(s.challenge(c.id).unwrap().solved);
    }

    #[test]
    fn wrong_answer_decrements_attempts() {
        let mut s = CaptchaService::new(60_000, 3, 7);
        let c = s.issue(ChallengeKind::Math, 1000);
        let result = s.verify(c.id, "9999999", 2000);
        assert_eq!(result, VerifyResult::WrongAnswer { attempts_remaining: 2 });
        let result = s.verify(c.id, "9999999", 3000);
        assert_eq!(result, VerifyResult::WrongAnswer { attempts_remaining: 1 });
    }

    #[test]
    fn exhausted_attempts_refused() {
        let mut s = CaptchaService::new(60_000, 2, 7);
        let c = s.issue(ChallengeKind::Math, 1000);
        s.verify(c.id, "wrong1", 2000);
        s.verify(c.id, "wrong2", 3000);
        let result = s.verify(c.id, &c.expected, 4000);
        assert_eq!(result, VerifyResult::TooManyAttempts);
    }

    #[test]
    fn expired_challenge_refused() {
        let mut s = CaptchaService::new(1000, 3, 7);
        let c = s.issue(ChallengeKind::Math, 1000);
        let result = s.verify(c.id, &c.expected, 5000); // 5s later, TTL=1s
        assert_eq!(result, VerifyResult::Expired);
    }

    #[test]
    fn already_solved_refuses_reverify() {
        let mut s = CaptchaService::new(60_000, 3, 7);
        let c = s.issue(ChallengeKind::Math, 1000);
        s.verify(c.id, &c.expected, 2000);
        let result = s.verify(c.id, &c.expected, 3000);
        assert_eq!(result, VerifyResult::AlreadySolved);
    }

    #[test]
    fn unknown_challenge_returns_unknown() {
        let mut s = CaptchaService::new(60_000, 3, 7);
        let result = s.verify(9999, "anything", 1000);
        assert_eq!(result, VerifyResult::Unknown);
    }

    #[test]
    fn answer_comparison_is_case_insensitive_and_trimmed() {
        let mut s = CaptchaService::new(60_000, 3, 7);
        let c = s.issue(ChallengeKind::WordRecall, 1000);
        let answer_with_noise = format!("  {}  ", c.expected.to_uppercase());
        let result = s.verify(c.id, &answer_with_noise, 2000);
        assert_eq!(result, VerifyResult::Success);
    }

    #[test]
    fn cleanup_removes_solved_and_expired() {
        let mut s = CaptchaService::new(1000, 3, 7);
        let a = s.issue(ChallengeKind::Math, 1000);
        let b = s.issue(ChallengeKind::Math, 1000);
        let _c = s.issue(ChallengeKind::Math, 1000);
        s.verify(a.id, &a.expected, 1500);   // a solved
        // b's TTL is (1000..2000); at now=5000 it's expired
        let removed = s.cleanup(5000);
        assert_eq!(removed, 3);  // a solved, b+c expired
        assert_eq!(s.active_count(), 0);
    }

    #[test]
    fn math_challenge_has_numeric_answer() {
        let mut s = CaptchaService::new(60_000, 3, 123);
        let c = s.issue(ChallengeKind::Math, 1000);
        let _: u64 = c.expected.parse().unwrap();
    }

    #[test]
    fn sequence_count_has_numeric_answer() {
        let mut s = CaptchaService::new(60_000, 3, 456);
        let c = s.issue(ChallengeKind::SequenceCount, 1000);
        let _: u32 = c.expected.parse().unwrap();
    }

    #[test]
    fn reverse_string_round_trips() {
        let mut s = CaptchaService::new(60_000, 3, 789);
        let c = s.issue(ChallengeKind::ReverseString, 1000);
        let re_reversed: String = c.expected.chars().rev().collect();
        // The expected answer, re-reversed, should match some part of the prompt.
        assert!(c.prompt.contains(&re_reversed));
    }
}
