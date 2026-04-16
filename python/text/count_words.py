"""Count Words in a String — word-level text analysis."""

import re
import string


def word_count(text: str) -> int:
    """Count words in text (split on whitespace).

    Args:
        text: The string to analyze.

    Returns:
        Total number of words found.
    """
    return len(text.split())


def word_frequency(text: str) -> dict:
    """Count occurrences of each word (case-insensitive, strip punctuation).

    Args:
        text: The string to analyze.

    Returns:
        Dict mapping each lowercase word to its count.
    """
    counts = {}
    for word in text.split():
        cleaned = word.strip(string.punctuation).lower()
        if cleaned:
            counts[cleaned] = counts.get(cleaned, 0) + 1
    return counts


def unique_words(text: str) -> int:
    """Count distinct words (case-insensitive, strip punctuation).

    Args:
        text: The string to analyze.

    Returns:
        Number of unique words.
    """
    return len(word_frequency(text))


def average_word_length(text: str) -> float:
    """Return the mean word length (punctuation stripped).

    Args:
        text: The string to analyze.

    Returns:
        Average length of words, or 0.0 if no words.
    """
    words = [w.strip(string.punctuation) for w in text.split()]
    words = [w for w in words if w]
    if not words:
        return 0.0
    return sum(len(w) for w in words) / len(words)


if __name__ == "__main__":
    demos = [
        "The quick brown fox jumps over the lazy dog",
        "To be or not to be, that is the question.",
    ]
    for phrase in demos:
        print(f"\n--- {phrase!r} ---")
        print(f"  word count:        {word_count(phrase)}")
        print(f"  word frequency:    {word_frequency(phrase)}")
        print(f"  unique words:      {unique_words(phrase)}")
        print(f"  avg word length:   {average_word_length(phrase):.2f}")
