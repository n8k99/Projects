//! Log File Maker — structured append-only log with level filtering and rotation.
//!
//! Tenth Files project. Every production system has logs. The essential
//! pattern: **append-only writes, size-bounded retention, level-based
//! filtering, readback in order**. G094 captures these four properties.
//!
//! Design:
//!   * Entries carry `timestamp`, `level`, `category`, `message`.
//!   * `Logger.log()` appends atomically (no partial writes in concurrent
//!      use — we're single-threaded here, so atomicity is trivial).
//!   * `level_threshold` drops entries below the threshold at write time.
//!   * Rotation caps the active log at `max_entries`; overflow moves to an
//!      archive file (older-than-N entries).
//!   * Querying filters by level/category/substring over the active log.
//!
//! The logger is pure: no real file I/O. `to_text` serialises in a
//! line-based format with one entry per line.

use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Level { Debug, Info, Warn, Error }

impl Level {
    pub fn as_str(&self) -> &'static str {
        match self {
            Level::Debug => "DEBUG",
            Level::Info  => "INFO",
            Level::Warn  => "WARN",
            Level::Error => "ERROR",
        }
    }
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "DEBUG" => Some(Level::Debug),
            "INFO"  => Some(Level::Info),
            "WARN"  => Some(Level::Warn),
            "ERROR" => Some(Level::Error),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    pub timestamp_ms: u64,
    pub level: Level,
    pub category: String,
    pub message: String,
}

impl fmt::Display for Entry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}\t{}\t{}\t{}",
               self.timestamp_ms, self.level.as_str(), self.category, self.message)
    }
}

#[derive(Debug, Clone)]
pub struct Logger {
    pub level_threshold: Level,
    pub max_entries: usize,
    pub active: Vec<Entry>,
    pub archive: Vec<Entry>,
}

impl Logger {
    pub fn new(level_threshold: Level, max_entries: usize) -> Self {
        Self { level_threshold, max_entries, active: Vec::new(), archive: Vec::new() }
    }

    pub fn log(&mut self, ts: u64, level: Level, category: &str, message: &str) {
        if level < self.level_threshold { return; }
        self.active.push(Entry {
            timestamp_ms: ts, level,
            category: category.into(), message: message.into(),
        });
        while self.active.len() > self.max_entries {
            self.archive.push(self.active.remove(0));
        }
    }

    pub fn query_level(&self, min_level: Level) -> Vec<&Entry> {
        self.active.iter().filter(|e| e.level >= min_level).collect()
    }

    pub fn query_category(&self, category: &str) -> Vec<&Entry> {
        self.active.iter().filter(|e| e.category == category).collect()
    }

    pub fn query_message_contains(&self, needle: &str) -> Vec<&Entry> {
        let lc = needle.to_lowercase();
        self.active.iter().filter(|e| e.message.to_lowercase().contains(&lc)).collect()
    }

    pub fn query_time_range(&self, from: u64, to: u64) -> Vec<&Entry> {
        self.active.iter().filter(|e| e.timestamp_ms >= from && e.timestamp_ms <= to).collect()
    }

    /// Serialise active log in order, one entry per line.
    pub fn to_text(&self) -> String {
        self.active.iter().map(|e| e.to_string()).collect::<Vec<_>>().join("\n")
            + if self.active.is_empty() { "" } else { "\n" }
    }

    /// Parse a log text back into entries. Returns None on any malformed line.
    pub fn parse_entries(text: &str) -> Option<Vec<Entry>> {
        let mut out = Vec::new();
        for line in text.lines() {
            if line.is_empty() { continue; }
            let parts: Vec<&str> = line.splitn(4, '\t').collect();
            if parts.len() != 4 { return None; }
            let ts: u64 = parts[0].parse().ok()?;
            let level = Level::parse(parts[1])?;
            out.push(Entry {
                timestamp_ms: ts, level,
                category: parts[2].into(), message: parts[3].into(),
            });
        }
        Some(out)
    }

    pub fn total_archived(&self) -> usize { self.archive.len() }
    pub fn active_len(&self) -> usize { self.active.len() }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn populated() -> Logger {
        let mut l = Logger::new(Level::Debug, 100);
        l.log(100, Level::Info, "boot", "starting up");
        l.log(200, Level::Debug, "boot", "loaded config");
        l.log(300, Level::Warn, "net", "slow response");
        l.log(400, Level::Error, "db", "connection lost");
        l.log(500, Level::Info, "db", "reconnected");
        l
    }

    #[test]
    fn log_respects_level_threshold() {
        let mut l = Logger::new(Level::Warn, 10);
        l.log(1, Level::Debug, "x", "m1");
        l.log(2, Level::Info, "x", "m2");
        l.log(3, Level::Warn, "x", "m3");
        l.log(4, Level::Error, "x", "m4");
        assert_eq!(l.active_len(), 2);
    }

    #[test]
    fn entries_in_order() {
        let l = populated();
        assert_eq!(l.active[0].timestamp_ms, 100);
        assert_eq!(l.active[4].timestamp_ms, 500);
    }

    #[test]
    fn query_level_filters_warn_and_above() {
        let l = populated();
        let w = l.query_level(Level::Warn);
        assert_eq!(w.len(), 2);
    }

    #[test]
    fn query_category_returns_matching_entries() {
        let l = populated();
        let db = l.query_category("db");
        assert_eq!(db.len(), 2);
        assert!(db.iter().all(|e| e.category == "db"));
    }

    #[test]
    fn query_message_substring() {
        let l = populated();
        let lost = l.query_message_contains("LOST");
        assert_eq!(lost.len(), 1);
        assert_eq!(lost[0].message, "connection lost");
    }

    #[test]
    fn query_time_range_inclusive() {
        let l = populated();
        let mid = l.query_time_range(200, 400);
        assert_eq!(mid.len(), 3);  // 200, 300, 400
    }

    #[test]
    fn rotation_moves_old_to_archive() {
        let mut l = Logger::new(Level::Debug, 3);
        for i in 0..5 {
            l.log(i * 10, Level::Info, "c", &format!("m{i}"));
        }
        assert_eq!(l.active_len(), 3);
        assert_eq!(l.total_archived(), 2);
        // Active contains last 3.
        assert_eq!(l.active[0].message, "m2");
        assert_eq!(l.active[2].message, "m4");
    }

    #[test]
    fn to_text_round_trip() {
        let l = populated();
        let text = l.to_text();
        let entries = Logger::parse_entries(&text).unwrap();
        assert_eq!(entries, l.active);
    }

    #[test]
    fn parse_rejects_malformed() {
        assert!(Logger::parse_entries("no tabs here").is_none());
        assert!(Logger::parse_entries("100\tINVALID\tcat\tmsg").is_none());
    }

    #[test]
    fn empty_logger_has_empty_text() {
        let l = Logger::new(Level::Debug, 10);
        assert_eq!(l.to_text(), "");
    }

    #[test]
    fn level_parse_round_trip() {
        for level in [Level::Debug, Level::Info, Level::Warn, Level::Error] {
            assert_eq!(Level::parse(level.as_str()), Some(level));
        }
    }

    #[test]
    fn queries_on_empty_logger_empty() {
        let l = Logger::new(Level::Debug, 10);
        assert!(l.query_level(Level::Info).is_empty());
        assert!(l.query_category("x").is_empty());
    }

    #[test]
    fn raising_threshold_after_init_affects_future_logs_only() {
        let mut l = Logger::new(Level::Debug, 10);
        l.log(1, Level::Debug, "x", "kept");
        l.level_threshold = Level::Warn;
        l.log(2, Level::Debug, "x", "dropped");
        l.log(3, Level::Error, "x", "kept");
        assert_eq!(l.active_len(), 2);
        assert_eq!(l.active[0].message, "kept");
        assert_eq!(l.active[1].message, "kept");
    }
}
