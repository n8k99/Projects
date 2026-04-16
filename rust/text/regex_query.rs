//! Regex Query Tool — pattern matching, extraction, and replacement.
//!
//! Since the `regex` crate is an external dependency, this module provides
//! pre-built pattern matchers for common structured data (email, URL, phone,
//! date, IP address) using character-level scanning, plus generic helpers
//! that delegate to `regex::Regex` when the crate is available.

/// A match result carrying the matched text and its byte-offset span.
#[derive(Debug, Clone, PartialEq)]
pub struct Match {
    pub text: String,
    pub start: usize,
    pub end: usize,
}

// ---------------------------------------------------------------------------
// Generic regex operations (require the `regex` crate)
// ---------------------------------------------------------------------------

/// Find all non-overlapping matches of `pattern` in `text`.
pub fn find_matches(text: &str, pattern: &str) -> Vec<String> {
    let re = regex::Regex::new(pattern).expect("invalid regex");
    re.find_iter(text).map(|m| m.as_str().to_string()).collect()
}

/// Find all matches with their byte-offset positions.
pub fn find_with_positions(text: &str, pattern: &str) -> Vec<Match> {
    let re = regex::Regex::new(pattern).expect("invalid regex");
    re.find_iter(text)
        .map(|m| Match {
            text: m.as_str().to_string(),
            start: m.start(),
            end: m.end(),
        })
        .collect()
}

/// Replace all occurrences of `pattern` with `replacement`.
pub fn replace(text: &str, pattern: &str, replacement: &str) -> String {
    let re = regex::Regex::new(pattern).expect("invalid regex");
    re.replace_all(text, replacement).into_owned()
}

/// Split `text` on every occurrence of `pattern`.
pub fn split(text: &str, pattern: &str) -> Vec<String> {
    let re = regex::Regex::new(pattern).expect("invalid regex");
    re.split(text).map(|s| s.to_string()).collect()
}

/// Return `true` if `pattern` matches anywhere in `text`.
pub fn is_match(text: &str, pattern: &str) -> bool {
    let re = regex::Regex::new(pattern).expect("invalid regex");
    re.is_match(text)
}

/// Extract capture-group tuples for every match of `pattern`.
///
/// Each inner `Vec<String>` contains the captured groups (excluding
/// the full match at group 0).
pub fn extract_groups(text: &str, pattern: &str) -> Vec<Vec<String>> {
    let re = regex::Regex::new(pattern).expect("invalid regex");
    re.captures_iter(text)
        .map(|caps| {
            (1..caps.len())
                .map(|i| caps.get(i).map_or(String::new(), |m| m.as_str().to_string()))
                .collect()
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Pre-built patterns (as &str constants)
// ---------------------------------------------------------------------------

pub const EMAIL: &str = r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}";
pub const URL: &str = r#"https?://[^\s<>"']+"#;
pub const PHONE: &str = r"\+?1?[-.\s]?\(?\d{3}\)?[-.\s]?\d{3}[-.\s]?\d{4}";
pub const DATE: &str = r"\d{4}-\d{2}-\d{2}";
pub const IP_ADDRESS: &str = r"\b(?:\d{1,3}\.){3}\d{1,3}\b";

// ---------------------------------------------------------------------------
// Standalone validators (no regex crate needed)
// ---------------------------------------------------------------------------

/// Check whether `s` looks like a valid email address (simple heuristic).
pub fn is_email(s: &str) -> bool {
    let parts: Vec<&str> = s.split('@').collect();
    if parts.len() != 2 {
        return false;
    }
    let local = parts[0];
    let domain = parts[1];
    !local.is_empty()
        && !domain.is_empty()
        && domain.contains('.')
        && domain.split('.').last().map_or(false, |tld| tld.len() >= 2)
        && local
            .chars()
            .all(|c| c.is_alphanumeric() || "._%+-".contains(c))
        && domain
            .chars()
            .all(|c| c.is_alphanumeric() || ".-".contains(c))
}

/// Check whether `s` looks like a valid IPv4 address.
pub fn is_ipv4(s: &str) -> bool {
    let octets: Vec<&str> = s.split('.').collect();
    octets.len() == 4
        && octets
            .iter()
            .all(|o| o.parse::<u8>().is_ok() && !o.starts_with('0') || *o == "0")
}

/// Find all email addresses in `text` using character scanning.
pub fn find_emails(text: &str) -> Vec<String> {
    text.split_whitespace()
        .filter_map(|word| {
            let trimmed = word.trim_matches(|c: char| !c.is_alphanumeric() && c != '@' && c != '.' && c != '_' && c != '%' && c != '+' && c != '-');
            if is_email(trimmed) {
                Some(trimmed.to_string())
            } else {
                None
            }
        })
        .collect()
}

/// Find all IPv4 addresses in `text`.
pub fn find_ipv4s(text: &str) -> Vec<String> {
    text.split_whitespace()
        .filter_map(|word| {
            let trimmed = word.trim_matches(|c: char| !c.is_ascii_digit() && c != '.');
            if is_ipv4(trimmed) {
                Some(trimmed.to_string())
            } else {
                None
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_email() {
        assert!(is_email("alice@example.com"));
        assert!(is_email("bob.smith+tag@work.org"));
        assert!(!is_email("not-an-email"));
        assert!(!is_email("@missing-local.com"));
        assert!(!is_email("missing-domain@"));
    }

    #[test]
    fn test_is_ipv4() {
        assert!(is_ipv4("192.168.1.1"));
        assert!(is_ipv4("10.0.0.1"));
        assert!(is_ipv4("0.0.0.0"));
        assert!(!is_ipv4("999.999.999.999"));
        assert!(!is_ipv4("1.2.3"));
        assert!(!is_ipv4("not.an.ip.addr"));
    }

    #[test]
    fn test_find_emails() {
        let text = "Mail alice@example.com and bob@work.org please.";
        let found = find_emails(text);
        assert_eq!(found, vec!["alice@example.com", "bob@work.org"]);
    }

    #[test]
    fn test_find_ipv4s() {
        let text = "Servers at 192.168.1.1 and 10.0.0.1 are up.";
        let found = find_ipv4s(text);
        assert_eq!(found, vec!["192.168.1.1", "10.0.0.1"]);
    }

    #[test]
    fn test_find_matches() {
        let text = "abc 123 def 456";
        let matches = find_matches(text, r"\d+");
        assert_eq!(matches, vec!["123", "456"]);
    }

    #[test]
    fn test_find_with_positions() {
        let text = "hello world";
        let matches = find_with_positions(text, r"\w+");
        assert_eq!(matches.len(), 2);
        assert_eq!(matches[0].text, "hello");
        assert_eq!(matches[0].start, 0);
        assert_eq!(matches[0].end, 5);
    }

    #[test]
    fn test_replace() {
        assert_eq!(
            replace("2026-04-16", r"(\d{4})-(\d{2})-(\d{2})", "$3/$2/$1"),
            "16/04/2026"
        );
    }

    #[test]
    fn test_split() {
        let parts = split("a, b, c", r",\s*");
        assert_eq!(parts, vec!["a", "b", "c"]);
    }

    #[test]
    fn test_is_match() {
        assert!(is_match("hello world", r"wo.ld"));
        assert!(!is_match("hello world", r"^\d+$"));
    }

    #[test]
    fn test_extract_groups() {
        let text = "[2026-04-16 08:30] ERROR: disk full";
        let groups = extract_groups(text, r"\[(\d{4}-\d{2}-\d{2}) (\d{2}:\d{2})\] (\w+): (.+)");
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0], vec!["2026-04-16", "08:30", "ERROR", "disk full"]);
    }
}
