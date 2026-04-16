//! Count Words in a String — word-level text analysis.

use std::collections::HashMap;

/// Count words in text (split on whitespace).
pub fn word_count(text: &str) -> usize {
    text.split_whitespace().count()
}

/// Strip leading and trailing punctuation from a word and lowercase it.
fn clean_word(word: &str) -> String {
    word.trim_matches(|c: char| c.is_ascii_punctuation())
        .to_lowercase()
}

/// Count occurrences of each word (case-insensitive, strip punctuation).
pub fn word_frequency(text: &str) -> HashMap<String, usize> {
    let mut counts = HashMap::new();
    for word in text.split_whitespace() {
        let cleaned = clean_word(word);
        if !cleaned.is_empty() {
            *counts.entry(cleaned).or_insert(0) += 1;
        }
    }
    counts
}

/// Count distinct words (case-insensitive, strip punctuation).
pub fn unique_words(text: &str) -> usize {
    word_frequency(text).len()
}

/// Return the mean word length (punctuation stripped).
///
/// Returns 0.0 if there are no words.
pub fn average_word_length(text: &str) -> f64 {
    let words: Vec<String> = text
        .split_whitespace()
        .map(|w| clean_word(w))
        .filter(|w| !w.is_empty())
        .collect();
    if words.is_empty() {
        return 0.0;
    }
    let total_len: usize = words.iter().map(|w| w.len()).sum();
    total_len as f64 / words.len() as f64
}

fn main() {
    let demos = [
        "The quick brown fox jumps over the lazy dog",
        "To be or not to be, that is the question.",
    ];
    for phrase in &demos {
        println!("\n--- {:?} ---", phrase);
        println!("  word count:        {}", word_count(phrase));
        println!("  word frequency:    {:?}", word_frequency(phrase));
        println!("  unique words:      {}", unique_words(phrase));
        println!("  avg word length:   {:.2}", average_word_length(phrase));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_word_count_basic() {
        assert_eq!(word_count("hello world"), 2);
        assert_eq!(word_count("The quick brown fox"), 4);
    }

    #[test]
    fn test_word_count_empty() {
        assert_eq!(word_count(""), 0);
        assert_eq!(word_count("   "), 0);
    }

    #[test]
    fn test_word_frequency_case_insensitive() {
        let freq = word_frequency("To be or not to be");
        assert_eq!(freq["to"], 2);
        assert_eq!(freq["be"], 2);
        assert_eq!(freq["or"], 1);
        assert_eq!(freq["not"], 1);
    }

    #[test]
    fn test_word_frequency_strips_punctuation() {
        let freq = word_frequency("hello, world! hello.");
        assert_eq!(freq["hello"], 2);
        assert_eq!(freq["world"], 1);
    }

    #[test]
    fn test_unique_words() {
        assert_eq!(unique_words("the the the"), 1);
        assert_eq!(unique_words("a b c d e"), 5);
        assert_eq!(unique_words("To be or not to be"), 4);
    }

    #[test]
    fn test_average_word_length() {
        // "hi" = 2, "there" = 5 => avg = 3.5
        let avg = average_word_length("hi there");
        assert!((avg - 3.5).abs() < 1e-10);
    }

    #[test]
    fn test_average_word_length_empty() {
        assert_eq!(average_word_length(""), 0.0);
    }

    #[test]
    fn test_average_word_length_strips_punctuation() {
        // "hello" = 5 => avg = 5.0
        let avg = average_word_length("hello!");
        assert!((avg - 5.0).abs() < 1e-10);
    }
}
