//! Reverse a String — reverse character order preserving Unicode.

/// Reverse the characters in a string.
///
/// Operates at the `char` level, which handles multi-byte UTF-8 correctly.
/// For full grapheme-cluster reversal, use the `unicode-segmentation` crate.
pub fn reverse_string(s: &str) -> String {
    s.chars().rev().collect()
}

/// Reverse the order of words in a string, keeping each word intact.
pub fn reverse_words(s: &str) -> String {
    s.split_whitespace().rev().collect::<Vec<_>>().join(" ")
}

/// Check whether a string reads the same forwards and backwards.
pub fn is_palindrome(s: &str) -> bool {
    let chars: Vec<char> = s.chars().collect();
    let len = chars.len();
    for i in 0..len / 2 {
        if chars[i] != chars[len - 1 - i] {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reverse_ascii() {
        assert_eq!(reverse_string("Hello, World!"), "!dlroW ,olleH");
    }

    #[test]
    fn test_reverse_unicode() {
        assert_eq!(reverse_string("café"), "éfac");
        assert_eq!(reverse_string("こんにちは"), "はちにんこ");
    }

    #[test]
    fn test_reverse_empty() {
        assert_eq!(reverse_string(""), "");
    }

    #[test]
    fn test_reverse_words() {
        assert_eq!(reverse_words("Hello, World!"), "World! Hello,");
        assert_eq!(reverse_words("one"), "one");
    }

    #[test]
    fn test_palindrome() {
        assert!(is_palindrome("racecar"));
        assert!(is_palindrome("madam"));
        assert!(!is_palindrome("hello"));
        assert!(is_palindrome(""));
        assert!(is_palindrome("a"));
    }
}
