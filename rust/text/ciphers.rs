//! Vigenere / Vernam / Caesar Ciphers — classical text encryption.

/// Encrypt text using Caesar cipher with given shift.
pub fn caesar_encrypt(text: &str, shift: i32) -> String {
    let s = ((shift % 26) + 26) as u32 % 26;
    text.chars()
        .map(|ch| {
            if ch.is_ascii_alphabetic() {
                let base = if ch.is_ascii_uppercase() { b'A' } else { b'a' } as u32;
                char::from_u32((ch as u32 - base + s) % 26 + base).unwrap()
            } else {
                ch
            }
        })
        .collect()
}

/// Decrypt Caesar cipher by shifting in the opposite direction.
pub fn caesar_decrypt(text: &str, shift: i32) -> String {
    caesar_encrypt(text, -shift)
}

/// Encrypt text using Vigenere cipher with given keyword.
pub fn vigenere_encrypt(text: &str, key: &str) -> String {
    assert!(
        !key.is_empty() && key.chars().all(|c| c.is_ascii_alphabetic()),
        "Key must be a non-empty alphabetic string"
    );
    let key_bytes: Vec<u8> = key.to_ascii_uppercase().bytes().collect();
    let mut ki = 0usize;
    text.chars()
        .map(|ch| {
            if ch.is_ascii_alphabetic() {
                let shift = (key_bytes[ki % key_bytes.len()] - b'A') as u32;
                let base = if ch.is_ascii_uppercase() { b'A' } else { b'a' } as u32;
                ki += 1;
                char::from_u32((ch as u32 - base + shift) % 26 + base).unwrap()
            } else {
                ch
            }
        })
        .collect()
}

/// Decrypt Vigenere cipher by inverting each key letter's shift.
pub fn vigenere_decrypt(text: &str, key: &str) -> String {
    assert!(
        !key.is_empty() && key.chars().all(|c| c.is_ascii_alphabetic()),
        "Key must be a non-empty alphabetic string"
    );
    let inverted: String = key
        .to_ascii_uppercase()
        .bytes()
        .map(|b| ((26 - (b - b'A') as u32) % 26) as u8 + b'A')
        .map(|b| b as char)
        .collect();
    vigenere_encrypt(text, &inverted)
}

/// Encrypt bytes using Vernam (one-time pad) XOR cipher.
/// Key must be at least as long as data.
pub fn vernam_encrypt(data: &[u8], key: &[u8]) -> Vec<u8> {
    assert!(
        key.len() >= data.len(),
        "Key must be at least as long as the data"
    );
    data.iter().zip(key.iter()).map(|(d, k)| d ^ k).collect()
}

/// Decrypt Vernam cipher (XOR is its own inverse).
pub fn vernam_decrypt(data: &[u8], key: &[u8]) -> Vec<u8> {
    vernam_encrypt(data, key)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_caesar_roundtrip() {
        let plain = "Hello, World!";
        for shift in [0, 1, 13, 25, -3] {
            let enc = caesar_encrypt(plain, shift);
            let dec = caesar_decrypt(&enc, shift);
            assert_eq!(dec, plain, "Caesar roundtrip failed for shift={shift}");
        }
    }

    #[test]
    fn test_caesar_preserves_non_alpha() {
        let text = "abc 123 !@#";
        let enc = caesar_encrypt(text, 5);
        assert_eq!(&enc[3..], " 123 !@#");
    }

    #[test]
    fn test_caesar_preserves_case() {
        let enc = caesar_encrypt("AbCd", 1);
        assert_eq!(enc, "BcDe");
    }

    #[test]
    fn test_vigenere_roundtrip() {
        let plain = "Hello, World!";
        let key = "SECRET";
        let enc = vigenere_encrypt(plain, key);
        let dec = vigenere_decrypt(&enc, key);
        assert_eq!(dec, plain);
    }

    #[test]
    fn test_vigenere_known_value() {
        // A with shift S(18) => S
        let enc = vigenere_encrypt("A", "S");
        assert_eq!(enc, "S");
    }

    #[test]
    fn test_vernam_roundtrip() {
        let data = b"Hello, World!";
        let key = b"\x4a\x1f\x7c\x33\x0b\x5e\x22\x6d\x11\x44\x59\x08\x3a";
        let enc = vernam_encrypt(data, key);
        let dec = vernam_decrypt(&enc, key);
        assert_eq!(dec, data);
    }

    #[test]
    #[should_panic(expected = "Key must be at least as long")]
    fn test_vernam_short_key() {
        vernam_encrypt(b"Hello", b"Hi");
    }
}
