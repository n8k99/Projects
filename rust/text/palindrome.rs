//! Check if Palindrome — detect and find palindromic strings.

/// Check whether a string reads the same forwards and backwards (exact).
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

/// Check whether a string is a palindrome, ignoring case, spaces, and punctuation.
pub fn is_palindrome_normalized(s: &str) -> bool {
    let normalized: Vec<char> = s
        .chars()
        .filter(|c| c.is_alphanumeric())
        .map(|c| c.to_ascii_lowercase())
        .collect();
    let len = normalized.len();
    for i in 0..len / 2 {
        if normalized[i] != normalized[len - 1 - i] {
            return false;
        }
    }
    true
}

/// Find the longest palindromic substring using expand-around-center.
pub fn longest_palindrome_substring(s: &str) -> &str {
    if s.len() < 2 {
        return s;
    }

    let bytes = s.as_bytes();
    let len = bytes.len();
    let mut start = 0usize;
    let mut max_len = 1usize;

    let expand = |mut left: usize, mut right: usize| -> (usize, usize) {
        while right < len && bytes[left] == bytes[right] {
            if left == 0 {
                return (0, right - 0 + 1);
            }
            left -= 1;
            right += 1;
        }
        (left + 1, right - left - 1)
    };

    for i in 0..len {
        // Odd-length palindromes
        let (s1, l1) = expand(i, i);
        if l1 > max_len {
            start = s1;
            max_len = l1;
        }

        // Even-length palindromes
        if i + 1 < len {
            let (s2, l2) = expand(i, i + 1);
            if l2 > max_len {
                start = s2;
                max_len = l2;
            }
        }
    }

    &s[start..start + max_len]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_palindrome() {
        assert!(is_palindrome("racecar"));
        assert!(is_palindrome("madam"));
        assert!(is_palindrome("a"));
        assert!(is_palindrome(""));
        assert!(!is_palindrome("hello"));
        assert!(!is_palindrome("Race car"));
    }

    #[test]
    fn test_is_palindrome_normalized() {
        assert!(is_palindrome_normalized("A man, a plan, a canal: Panama"));
        assert!(is_palindrome_normalized("Was it a car or a cat I saw?"));
        assert!(is_palindrome_normalized("racecar"));
        assert!(is_palindrome_normalized(""));
        assert!(!is_palindrome_normalized("hello world"));
    }

    #[test]
    fn test_longest_palindrome_substring() {
        let result = longest_palindrome_substring("babad");
        assert!(result == "bab" || result == "aba");

        assert_eq!(longest_palindrome_substring("forgeeksskeegfor"), "geeksskeeg");
        assert_eq!(longest_palindrome_substring("a"), "a");
        assert_eq!(longest_palindrome_substring(""), "");
        assert_eq!(longest_palindrome_substring("cbbd"), "bb");
    }
}
