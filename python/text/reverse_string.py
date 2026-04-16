"""Reverse a String — reverse character order preserving Unicode."""


def reverse_string(s: str) -> str:
    """Reverse the characters in a string, preserving full Unicode codepoints."""
    return s[::-1]


def reverse_words(s: str) -> str:
    """Reverse the order of words in a string, keeping each word intact."""
    return " ".join(s.split()[::-1])


def is_palindrome(s: str) -> bool:
    """Check whether a string reads the same forwards and backwards."""
    return s == s[::-1]


if __name__ == "__main__":
    demos = ["Hello, World!", "café", "こんにちは", "race car"]

    for text in demos:
        rev = reverse_string(text)
        words = reverse_words(text)
        print(f"  original: {text}")
        print(f"  reversed: {rev}")
        print(f"  words:    {words}")
        if is_palindrome(text.replace(" ", "")):
            print(f"  ** palindrome (ignoring spaces)")
        print()
