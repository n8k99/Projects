//! Quote Tracker — collect, categorize, and retrieve quotes.

use std::collections::HashSet;

/// A single quote with attribution and categorization.
#[derive(Debug, Clone)]
pub struct Quote {
    pub text: String,
    pub author: String,
    pub source: Option<String>,
    pub tags: Vec<String>,
}

impl std::fmt::Display for Quote {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "\"{}\" — {}", self.text, self.author)?;
        if let Some(ref src) = self.source {
            write!(f, " ({})", src)?;
        }
        if !self.tags.is_empty() {
            write!(f, " [{}]", self.tags.join(", "))?;
        }
        Ok(())
    }
}

/// A curated collection of quotes with attribution, tags, and search.
pub struct QuoteTracker {
    quotes: Vec<Quote>,
}

impl QuoteTracker {
    /// Create an empty quote tracker.
    pub fn new() -> Self {
        Self { quotes: Vec::new() }
    }

    /// Add a quote to the collection.
    pub fn add_quote(&mut self, text: &str, author: &str, source: Option<&str>, tags: &[&str]) {
        self.quotes.push(Quote {
            text: text.to_string(),
            author: author.to_string(),
            source: source.map(|s| s.to_string()),
            tags: tags.iter().map(|t| t.to_string()).collect(),
        });
    }

    /// Return a random quote, or None if empty.
    pub fn random_quote(&self) -> Option<&Quote> {
        if self.quotes.is_empty() {
            return None;
        }
        // Simple deterministic "random" using collection size as seed
        let idx = (self.quotes.len() * 7 + 3) % self.quotes.len();
        Some(&self.quotes[idx])
    }

    /// Return all quotes by the given author (case-insensitive substring match).
    pub fn search_by_author(&self, author: &str) -> Vec<&Quote> {
        let needle = author.to_lowercase();
        self.quotes
            .iter()
            .filter(|q| q.author.to_lowercase().contains(&needle))
            .collect()
    }

    /// Return all quotes with the given tag (case-insensitive).
    pub fn search_by_tag(&self, tag: &str) -> Vec<&Quote> {
        let needle = tag.to_lowercase();
        self.quotes
            .iter()
            .filter(|q| q.tags.iter().any(|t| t.to_lowercase() == needle))
            .collect()
    }

    /// Return sorted list of unique authors.
    pub fn list_authors(&self) -> Vec<String> {
        let mut authors: Vec<String> = self
            .quotes
            .iter()
            .map(|q| q.author.clone())
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();
        authors.sort();
        authors
    }

    /// Return sorted list of unique tags.
    pub fn list_tags(&self) -> Vec<String> {
        let mut tags: Vec<String> = self
            .quotes
            .iter()
            .flat_map(|q| q.tags.iter().cloned())
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();
        tags.sort();
        tags
    }

    /// Return total number of quotes.
    pub fn count(&self) -> usize {
        self.quotes.len()
    }
}

impl Default for QuoteTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_tracker() -> QuoteTracker {
        let mut t = QuoteTracker::new();
        t.add_quote("Talk is cheap. Show me the code.", "Linus Torvalds", None, &["programming", "work"]);
        t.add_quote("The best way to predict the future is to invent it.", "Alan Kay", Some("1971"), &["innovation", "programming"]);
        t.add_quote("Simplicity is the ultimate sophistication.", "Leonardo da Vinci", None, &["design", "philosophy"]);
        t
    }

    #[test]
    fn test_add_and_count() {
        let t = sample_tracker();
        assert_eq!(t.count(), 3);
    }

    #[test]
    fn test_search_by_author() {
        let t = sample_tracker();
        let results = t.search_by_author("alan");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].author, "Alan Kay");
    }

    #[test]
    fn test_search_by_author_case_insensitive() {
        let t = sample_tracker();
        let results = t.search_by_author("LINUS");
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_search_by_tag() {
        let t = sample_tracker();
        let results = t.search_by_tag("programming");
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_search_by_tag_case_insensitive() {
        let t = sample_tracker();
        let results = t.search_by_tag("DESIGN");
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_list_authors() {
        let t = sample_tracker();
        let authors = t.list_authors();
        assert_eq!(authors.len(), 3);
        assert_eq!(authors[0], "Alan Kay");
    }

    #[test]
    fn test_list_tags() {
        let t = sample_tracker();
        let tags = t.list_tags();
        assert!(tags.contains(&"programming".to_string()));
        assert!(tags.contains(&"design".to_string()));
        assert!(tags.contains(&"philosophy".to_string()));
    }

    #[test]
    fn test_random_quote_nonempty() {
        let t = sample_tracker();
        assert!(t.random_quote().is_some());
    }

    #[test]
    fn test_random_quote_empty() {
        let t = QuoteTracker::new();
        assert!(t.random_quote().is_none());
    }

    #[test]
    fn test_no_results() {
        let t = sample_tracker();
        assert!(t.search_by_author("nobody").is_empty());
        assert!(t.search_by_tag("nonexistent").is_empty());
    }
}
