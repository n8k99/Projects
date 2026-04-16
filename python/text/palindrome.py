"""Check if Palindrome — detect and find palindromic strings."""

import re


def is_palindrome(s: str) -> bool:
    """Check whether a string reads the same forwards and backwards (exact)."""
    return s == s[::-1]


def is_palindrome_normalized(s: str) -> bool:
    """Check whether a string is a palindrome, ignoring case, spaces, and punctuation."""
    normalized = re.sub(r"[^a-zA-Z0-9]", "", s).lower()
    return normalized == normalized[::-1]


def longest_palindrome_substring(s: str) -> str:
    """Find the longest palindromic substring using expand-around-center."""
    if len(s) < 2:
        return s

    start, end = 0, 0

    def expand(left: int, right: int) -> tuple[int, int]:
        while left >= 0 and right < len(s) and s[left] == s[right]:
            left -= 1
            right += 1
        return left + 1, right - 1

    for i in range(len(s)):
        # Odd-length palindromes
        l1, r1 = expand(i, i)
        if r1 - l1 > end - start:
            start, end = l1, r1

        # Even-length palindromes
        l2, r2 = expand(i, i + 1)
        if r2 - l2 > end - start:
            start, end = l2, r2

    return s[start : end + 1]


if __name__ == "__main__":
    demos_exact = [
        ("racecar", True),
        ("hello", False),
    ]

    demos_normalized = [
        ("A man, a plan, a canal: Panama", True),
        ("Was it a car or a cat I saw?", True),
    ]

    demos_longest = [
        ("babad", ["bab", "aba"]),
        ("forgeeksskeegfor", ["geeksskeeg"]),
    ]

    print("=== Exact palindrome check ===")
    for text, expected in demos_exact:
        result = is_palindrome(text)
        status = "ok" if result == expected else "FAIL"
        print(f"  [{status}] is_palindrome({text!r}) = {result}")

    print()
    print("=== Normalized palindrome check ===")
    for text, expected in demos_normalized:
        result = is_palindrome_normalized(text)
        status = "ok" if result == expected else "FAIL"
        print(f"  [{status}] is_palindrome_normalized({text!r}) = {result}")

    print()
    print("=== Longest palindromic substring ===")
    for text, expected in demos_longest:
        result = longest_palindrome_substring(text)
        status = "ok" if result in expected else "FAIL"
        print(f"  [{status}] longest_palindrome_substring({text!r}) = {result!r}")
