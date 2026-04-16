"""Vigenere / Vernam / Caesar Ciphers — classical text encryption."""


def caesar_encrypt(text: str, shift: int) -> str:
    """Encrypt text using Caesar cipher with given shift."""
    result = []
    shift = shift % 26
    for ch in text:
        if ch.isalpha():
            base = ord('A') if ch.isupper() else ord('a')
            result.append(chr((ord(ch) - base + shift) % 26 + base))
        else:
            result.append(ch)
    return ''.join(result)


def caesar_decrypt(text: str, shift: int) -> str:
    """Decrypt Caesar cipher by shifting in the opposite direction."""
    return caesar_encrypt(text, -shift)


def vigenere_encrypt(text: str, key: str) -> str:
    """Encrypt text using Vigenere cipher with given keyword."""
    if not key or not key.isalpha():
        raise ValueError("Key must be a non-empty alphabetic string")
    key_upper = key.upper()
    result = []
    ki = 0
    for ch in text:
        if ch.isalpha():
            shift = ord(key_upper[ki % len(key_upper)]) - ord('A')
            base = ord('A') if ch.isupper() else ord('a')
            result.append(chr((ord(ch) - base + shift) % 26 + base))
            ki += 1
        else:
            result.append(ch)
    return ''.join(result)


def vigenere_decrypt(text: str, key: str) -> str:
    """Decrypt Vigenere cipher by inverting each key letter's shift."""
    if not key or not key.isalpha():
        raise ValueError("Key must be a non-empty alphabetic string")
    inverted_key = ''.join(
        chr((26 - (ord(c) - ord('A'))) % 26 + ord('A')) for c in key.upper()
    )
    return vigenere_encrypt(text, inverted_key)


def vernam_encrypt(data: bytes, key: bytes) -> bytes:
    """Encrypt bytes using Vernam (one-time pad) XOR cipher."""
    if len(key) < len(data):
        raise ValueError("Key must be at least as long as the data")
    return bytes(d ^ k for d, k in zip(data, key))


def vernam_decrypt(data: bytes, key: bytes) -> bytes:
    """Decrypt Vernam cipher (XOR is its own inverse)."""
    return vernam_encrypt(data, key)


if __name__ == "__main__":
    # Caesar demo
    plain = "Hello, World!"
    shift = 13
    enc = caesar_encrypt(plain, shift)
    dec = caesar_decrypt(enc, shift)
    print(f"Caesar (shift={shift}):")
    print(f"  Plain:     {plain}")
    print(f"  Encrypted: {enc}")
    print(f"  Decrypted: {dec}")
    assert dec == plain, "Caesar roundtrip failed"
    print()

    # Vigenere demo
    key = "SECRET"
    enc = vigenere_encrypt(plain, key)
    dec = vigenere_decrypt(enc, key)
    print(f"Vigenere (key={key!r}):")
    print(f"  Plain:     {plain}")
    print(f"  Encrypted: {enc}")
    print(f"  Decrypted: {dec}")
    assert dec == plain, "Vigenere roundtrip failed"
    print()

    # Vernam demo
    message = b"Hello, World!"
    otp_key = b"\x4a\x1f\x7c\x33\x0b\x5e\x22\x6d\x11\x44\x59\x08\x3a"
    enc = vernam_encrypt(message, otp_key)
    dec = vernam_decrypt(enc, otp_key)
    print(f"Vernam (one-time pad):")
    print(f"  Plain:     {message}")
    print(f"  Encrypted: {enc.hex()}")
    print(f"  Decrypted: {dec}")
    assert dec == message, "Vernam roundtrip failed"
    print()

    print("All roundtrip tests passed.")
