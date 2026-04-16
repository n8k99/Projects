//! Guestbook / Journal — chronological append-only entry log.

use std::fmt;

/// An immutable journal entry.
#[derive(Debug, Clone)]
pub struct Entry {
    pub author: String,
    pub content: String,
    pub timestamp: String, // ISO-8601 style, e.g. "2026-04-16 14:30:00"
}

impl fmt::Display for Entry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}: {}", self.timestamp, self.author, self.content)
    }
}

/// Chronological append-only journal / guestbook.
pub struct Journal {
    entries: Vec<Entry>,
}

impl Journal {
    pub fn new() -> Self {
        Journal {
            entries: Vec::new(),
        }
    }

    /// Generate a simple timestamp string from system time.
    fn now_timestamp() -> String {
        use std::time::{SystemTime, UNIX_EPOCH};
        let secs = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        // Convert epoch seconds to a readable timestamp
        let days = secs / 86400;
        let time_of_day = secs % 86400;
        let hours = time_of_day / 3600;
        let minutes = (time_of_day % 3600) / 60;
        let seconds = time_of_day % 60;

        // Simple date calculation from epoch days
        let (year, month, day) = epoch_days_to_date(days as i64);
        format!(
            "{:04}-{:02}-{:02} {:02}:{:02}:{:02} UTC",
            year, month, day, hours, minutes, seconds
        )
    }

    /// Append a new entry. Returns a reference to the created entry.
    pub fn add_entry(&mut self, author: &str, content: &str) -> &Entry {
        let entry = Entry {
            author: author.to_string(),
            content: content.to_string(),
            timestamp: Self::now_timestamp(),
        };
        self.entries.push(entry);
        self.entries.last().unwrap()
    }

    /// Add an entry with an explicit timestamp (for testing / import).
    pub fn add_entry_with_timestamp(
        &mut self,
        author: &str,
        content: &str,
        timestamp: &str,
    ) -> &Entry {
        let entry = Entry {
            author: author.to_string(),
            content: content.to_string(),
            timestamp: timestamp.to_string(),
        };
        self.entries.push(entry);
        self.entries.last().unwrap()
    }

    /// Return all entries, newest first.
    pub fn get_entries(&self) -> Vec<&Entry> {
        self.entries.iter().rev().collect()
    }

    /// Return entries by a specific author, newest first.
    pub fn get_entries_by_author(&self, author: &str) -> Vec<&Entry> {
        self.entries
            .iter()
            .rev()
            .filter(|e| e.author == author)
            .collect()
    }

    /// Return entries whose timestamp falls within [start, end] (string comparison).
    pub fn get_entries_in_range(&self, start: &str, end: &str) -> Vec<&Entry> {
        self.entries
            .iter()
            .rev()
            .filter(|e| e.timestamp.as_str() >= start && e.timestamp.as_str() <= end)
            .collect()
    }

    /// Return total number of entries.
    pub fn entry_count(&self) -> usize {
        self.entries.len()
    }

    /// Export the full journal as plain text (chronological order).
    pub fn export_text(&self) -> String {
        self.entries
            .iter()
            .map(|e| e.to_string())
            .collect::<Vec<_>>()
            .join("\n")
    }
}

/// Convert days since Unix epoch to (year, month, day).
fn epoch_days_to_date(mut days: i64) -> (i64, u32, u32) {
    // Algorithm based on Howard Hinnant's civil_from_days
    days += 719468;
    let era = if days >= 0 { days } else { days - 146096 } / 146097;
    let doe = (days - era * 146097) as u32;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y, m, d)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_and_count() {
        let mut journal = Journal::new();
        assert_eq!(journal.entry_count(), 0);

        journal.add_entry("Alice", "Hello world");
        assert_eq!(journal.entry_count(), 1);

        journal.add_entry("Bob", "Greetings");
        assert_eq!(journal.entry_count(), 2);
    }

    #[test]
    fn test_entries_newest_first() {
        let mut journal = Journal::new();
        journal.add_entry_with_timestamp("A", "First", "2026-01-01 00:00:00 UTC");
        journal.add_entry_with_timestamp("B", "Second", "2026-01-02 00:00:00 UTC");
        journal.add_entry_with_timestamp("C", "Third", "2026-01-03 00:00:00 UTC");

        let entries = journal.get_entries();
        assert_eq!(entries[0].content, "Third");
        assert_eq!(entries[1].content, "Second");
        assert_eq!(entries[2].content, "First");
    }

    #[test]
    fn test_filter_by_author() {
        let mut journal = Journal::new();
        journal.add_entry_with_timestamp("Alice", "Entry 1", "2026-01-01 00:00:00 UTC");
        journal.add_entry_with_timestamp("Bob", "Entry 2", "2026-01-02 00:00:00 UTC");
        journal.add_entry_with_timestamp("Alice", "Entry 3", "2026-01-03 00:00:00 UTC");

        let alice_entries = journal.get_entries_by_author("Alice");
        assert_eq!(alice_entries.len(), 2);
        assert_eq!(alice_entries[0].content, "Entry 3");
        assert_eq!(alice_entries[1].content, "Entry 1");

        let bob_entries = journal.get_entries_by_author("Bob");
        assert_eq!(bob_entries.len(), 1);
    }

    #[test]
    fn test_date_range() {
        let mut journal = Journal::new();
        journal.add_entry_with_timestamp("A", "Jan", "2026-01-15 12:00:00 UTC");
        journal.add_entry_with_timestamp("A", "Feb", "2026-02-15 12:00:00 UTC");
        journal.add_entry_with_timestamp("A", "Mar", "2026-03-15 12:00:00 UTC");

        let feb_entries = journal.get_entries_in_range(
            "2026-02-01 00:00:00 UTC",
            "2026-02-28 23:59:59 UTC",
        );
        assert_eq!(feb_entries.len(), 1);
        assert_eq!(feb_entries[0].content, "Feb");
    }

    #[test]
    fn test_export_text() {
        let mut journal = Journal::new();
        journal.add_entry_with_timestamp("Alice", "Hello", "2026-01-01 10:00:00 UTC");
        journal.add_entry_with_timestamp("Bob", "World", "2026-01-01 11:00:00 UTC");

        let text = journal.export_text();
        assert!(text.contains("Alice: Hello"));
        assert!(text.contains("Bob: World"));
        // Chronological order in export
        assert!(text.find("Alice").unwrap() < text.find("Bob").unwrap());
    }

    #[test]
    fn test_immutability() {
        let mut journal = Journal::new();
        journal.add_entry("Test", "Content");
        // Entries vector is private, no remove/edit methods exist
        // The only mutation is append via add_entry
        assert_eq!(journal.entry_count(), 1);
    }
}
