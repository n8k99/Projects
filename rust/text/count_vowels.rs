//! Count Vowels — analyze vowel distribution in text.

use std::collections::HashMap;

/// Returns true if the character is a vowel (optionally including 'y').
fn is_vowel(ch: char, include_y: bool) -> bool {
    matches!(ch.to_ascii_lowercase(), 'a' | 'e' | 'i' | 'o' | 'u')
        || (include_y && ch.to_ascii_lowercase() == 'y')
}

/// Count total vowels in text.
pub fn count_vowels(text: &str, include_y: bool) -> usize {
    text.chars().filter(|&ch| is_vowel(ch, include_y)).count()
}

/// Return per-vowel frequency counts.
pub fn vowel_breakdown(text: &str, include_y: bool) -> HashMap<char, usize> {
    let keys: &[char] = if include_y {
        &['a', 'e', 'i', 'o', 'u', 'y']
    } else {
        &['a', 'e', 'i', 'o', 'u']
    };
    let mut counts: HashMap<char, usize> = keys.iter().map(|&k| (k, 0)).collect();
    for ch in text.chars() {
        let lower = ch.to_ascii_lowercase();
        if let Some(entry) = counts.get_mut(&lower) {
            *entry += 1;
        }
    }
    counts
}

/// Return the ratio of vowels to total alphabetic characters.
///
/// Returns 0.0 if there are no alphabetic characters.
pub fn vowel_ratio(text: &str) -> f64 {
    let alpha_count = text.chars().filter(|ch| ch.is_alphabetic()).count();
    if alpha_count == 0 {
        return 0.0;
    }
    count_vowels(text, false) as f64 / alpha_count as f64
}

fn main() {
    let demos = [
        "Hello World",
        "The quick brown fox",
        "Why try fly by my dry cry?",
    ];
    for phrase in &demos {
        println!("\n--- {:?} ---", phrase);
        println!("  vowels:      {}", count_vowels(phrase, false));
        println!("  vowels (+y): {}", count_vowels(phrase, true));
        println!("  breakdown:   {:?}", vowel_breakdown(phrase, false));
        println!("  breakdown (+y): {:?}", vowel_breakdown(phrase, true));
        println!("  ratio:       {:.3}", vowel_ratio(phrase));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_count_vowels_basic() {
        assert_eq!(count_vowels("hello", false), 2);
        assert_eq!(count_vowels("HELLO", false), 2);
        assert_eq!(count_vowels("bcdfg", false), 0);
    }

    #[test]
    fn test_count_vowels_with_y() {
        assert_eq!(count_vowels("happy", false), 1);
        assert_eq!(count_vowels("happy", true), 2);
        assert_eq!(count_vowels("Why try fly", true), 3);
    }

    #[test]
    fn test_count_vowels_empty() {
        assert_eq!(count_vowels("", false), 0);
        assert_eq!(count_vowels("123!@#", false), 0);
    }

    #[test]
    fn test_vowel_breakdown() {
        let bd = vowel_breakdown("aardvark", false);
        assert_eq!(bd[&'a'], 3);
        assert_eq!(bd[&'e'], 0);
        assert_eq!(bd[&'i'], 0);
        assert_eq!(bd[&'o'], 0);
        assert_eq!(bd[&'u'], 0);
    }

    #[test]
    fn test_vowel_breakdown_with_y() {
        let bd = vowel_breakdown("yay", true);
        assert_eq!(bd[&'a'], 1);
        assert_eq!(bd[&'y'], 2);
    }

    #[test]
    fn test_vowel_ratio() {
        // "hello" has 2 vowels out of 5 letters
        let ratio = vowel_ratio("hello");
        assert!((ratio - 0.4).abs() < 1e-10);
    }

    #[test]
    fn test_vowel_ratio_empty() {
        assert_eq!(vowel_ratio(""), 0.0);
        assert_eq!(vowel_ratio("123"), 0.0);
    }

    #[test]
    fn test_vowel_ratio_all_vowels() {
        let ratio = vowel_ratio("aeiou");
        assert!((ratio - 1.0).abs() < 1e-10);
    }
}
