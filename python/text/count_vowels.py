"""Count Vowels — analyze vowel distribution in text."""

VOWELS = set("aeiou")
VOWELS_WITH_Y = set("aeiouy")


def count_vowels(text: str, include_y: bool = False) -> int:
    """Count total vowels in text.

    Args:
        text: The string to analyze.
        include_y: Whether to count 'y' as a vowel.

    Returns:
        Total number of vowels found.
    """
    targets = VOWELS_WITH_Y if include_y else VOWELS
    return sum(1 for ch in text.lower() if ch in targets)


def vowel_breakdown(text: str, include_y: bool = False) -> dict:
    """Return per-vowel frequency counts.

    Args:
        text: The string to analyze.
        include_y: Whether to include 'y' in the breakdown.

    Returns:
        Dict mapping each vowel to its count, e.g. {'a': 3, 'e': 5, ...}.
    """
    keys = "aeiouy" if include_y else "aeiou"
    counts = {v: 0 for v in keys}
    for ch in text.lower():
        if ch in counts:
            counts[ch] += 1
    return counts


def vowel_ratio(text: str) -> float:
    """Return the ratio of vowels to total alphabetic characters.

    Args:
        text: The string to analyze.

    Returns:
        A float between 0.0 and 1.0, or 0.0 if no alphabetic characters.
    """
    alpha_count = sum(1 for ch in text if ch.isalpha())
    if alpha_count == 0:
        return 0.0
    return count_vowels(text) / alpha_count


if __name__ == "__main__":
    demos = [
        "Hello World",
        "The quick brown fox",
        "Why try fly by my dry cry?",
    ]
    for phrase in demos:
        print(f"\n--- {phrase!r} ---")
        print(f"  vowels:      {count_vowels(phrase)}")
        print(f"  vowels (+y): {count_vowels(phrase, include_y=True)}")
        print(f"  breakdown:   {vowel_breakdown(phrase)}")
        print(f"  breakdown (+y): {vowel_breakdown(phrase, include_y=True)}")
        print(f"  ratio:       {vowel_ratio(phrase):.3f}")
