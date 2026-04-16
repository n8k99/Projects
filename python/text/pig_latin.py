"""Pig Latin — translate English text to Pig Latin."""

VOWELS = set("aeiouAEIOU")


def pig_latin_word(word: str) -> str:
    """Convert a single English word to Pig Latin.

    Rules:
    - Consonant cluster start: move cluster to end, add 'ay'
    - Vowel start: add 'yay'
    - Preserve capitalization of original first letter
    - Preserve trailing punctuation
    """
    if not word:
        return word

    # Strip trailing punctuation
    trail = ""
    core = word
    while core and not core[-1].isalpha():
        trail = core[-1] + trail
        core = core[:-1]

    if not core:
        return word

    was_cap = core[0].isupper()

    lower = core.lower()

    if lower[0] in "aeiou":
        result = lower + "yay"
    else:
        # Find consonant cluster
        i = 0
        while i < len(lower) and lower[i] not in "aeiou":
            i += 1
        cluster = lower[:i]
        rest = lower[i:]
        result = rest + cluster + "ay"

    # Restore capitalization
    if was_cap:
        result = result[0].upper() + result[1:]

    return result + trail


def pig_latin(text: str) -> str:
    """Convert an English sentence to Pig Latin."""
    words = text.split(" ")
    return " ".join(pig_latin_word(w) for w in words)


if __name__ == "__main__":
    demos = [
        "Hello World",
        "The quick brown fox jumps over the lazy dog",
        "Apple pie is great",
    ]
    for sentence in demos:
        print(f"{sentence}")
        print(f"  -> {pig_latin(sentence)}")
        print()
