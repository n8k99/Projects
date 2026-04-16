//! Pig Latin — translate English text to Pig Latin.

fn is_vowel(c: char) -> bool {
    matches!(c.to_ascii_lowercase(), 'a' | 'e' | 'i' | 'o' | 'u')
}

/// Convert a single English word to Pig Latin.
///
/// - Consonant cluster start: move cluster to end, add "ay"
/// - Vowel start: add "yay"
/// - Preserve capitalization of original first letter
/// - Preserve trailing punctuation
pub fn pig_latin_word(word: &str) -> String {
    if word.is_empty() {
        return String::new();
    }

    // Split trailing punctuation
    let alpha_end = word.len()
        - word
            .chars()
            .rev()
            .take_while(|c| !c.is_alphabetic())
            .count();
    let core = &word[..alpha_end];
    let trail = &word[alpha_end..];

    if core.is_empty() {
        return word.to_string();
    }

    let was_cap = core.chars().next().unwrap().is_uppercase();
    let lower: String = core.to_lowercase();
    let chars: Vec<char> = lower.chars().collect();

    let result = if is_vowel(chars[0]) {
        format!("{}yay", lower)
    } else {
        let cluster_end = chars.iter().position(|&c| is_vowel(c)).unwrap_or(chars.len());
        let cluster: String = chars[..cluster_end].iter().collect();
        let rest: String = chars[cluster_end..].iter().collect();
        format!("{}{}ay", rest, cluster)
    };

    let result = if was_cap {
        let mut c = result.chars();
        match c.next() {
            Some(first) => first.to_uppercase().to_string() + c.as_str(),
            None => result,
        }
    } else {
        result
    };

    format!("{}{}", result, trail)
}

/// Convert an English sentence to Pig Latin.
pub fn pig_latin(text: &str) -> String {
    text.split(' ')
        .map(|w| pig_latin_word(w))
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_consonant_start() {
        assert_eq!(pig_latin_word("hello"), "ellohay");
    }

    #[test]
    fn test_consonant_cluster() {
        assert_eq!(pig_latin_word("string"), "ingstray");
    }

    #[test]
    fn test_vowel_start() {
        assert_eq!(pig_latin_word("apple"), "appleyay");
        assert_eq!(pig_latin_word("egg"), "eggyay");
    }

    #[test]
    fn test_capitalization() {
        assert_eq!(pig_latin_word("Hello"), "Ellohay");
        assert_eq!(pig_latin_word("Apple"), "Appleyay");
    }

    #[test]
    fn test_punctuation() {
        assert_eq!(pig_latin_word("hello!"), "ellohay!");
        assert_eq!(pig_latin_word("world."), "orldway.");
    }

    #[test]
    fn test_sentence() {
        assert_eq!(pig_latin("Hello World"), "Ellohay Orldway");
    }

    #[test]
    fn test_empty() {
        assert_eq!(pig_latin_word(""), "");
    }
}
