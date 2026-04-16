// CD Key Generator -- generate and validate software license keys.

use std::fmt;

const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
const GROUP_LEN: usize = 5;
const DATA_GROUPS: usize = 4;
const PRIME: u64 = 65521;

fn random_u8() -> u8 {
    use std::time::SystemTime;
    let t = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .subsec_nanos();
    // Simple PRNG seeded from time + address of a stack variable
    let stack_var: u8 = 0;
    let addr = &stack_var as *const u8 as u64;
    ((t as u64).wrapping_mul(6364136223846793005).wrapping_add(addr)) as u8
}

struct Rng(u64);

impl Rng {
    fn new() -> Self {
        let seed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64;
        Rng(seed)
    }

    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        self.0
    }

    fn next_usize(&mut self, bound: usize) -> usize {
        (self.next() % bound as u64) as usize
    }
}

fn compute_checksum(groups: &[String]) -> String {
    let raw: Vec<u8> = groups.iter().flat_map(|g| g.bytes()).collect();
    let mut h: u64 = 0;
    for (i, &b) in raw.iter().enumerate() {
        h = h.wrapping_add((b as u64) * (i as u64 + 1));
    }
    h %= PRIME;
    let mut chars = Vec::with_capacity(GROUP_LEN);
    for _ in 0..GROUP_LEN {
        chars.push(CHARSET[(h % CHARSET.len() as u64) as usize]);
        h /= CHARSET.len() as u64;
    }
    String::from_utf8(chars).unwrap()
}

pub fn generate_key() -> String {
    let mut rng = Rng::new();
    let groups: Vec<String> = (0..DATA_GROUPS)
        .map(|_| {
            (0..GROUP_LEN)
                .map(|_| CHARSET[rng.next_usize(CHARSET.len())] as char)
                .collect()
        })
        .collect();
    let checksum = compute_checksum(&groups);
    let mut all = groups;
    all.push(checksum);
    all.join("-")
}

pub fn validate_key(key: &str) -> bool {
    let parts: Vec<&str> = key.trim().split('-').collect();
    if parts.len() != DATA_GROUPS + 1 {
        return false;
    }
    for part in &parts {
        if part.len() != GROUP_LEN {
            return false;
        }
        if !part.bytes().all(|b| CHARSET.contains(&b.to_ascii_uppercase())) {
            return false;
        }
    }
    let groups: Vec<String> = parts.iter().map(|p| p.to_uppercase()).collect();
    let expected = compute_checksum(&groups[..DATA_GROUPS]);
    groups[DATA_GROUPS] == expected
}

pub fn generate_batch(n: usize) -> Vec<String> {
    // Add small delay between keys to ensure different RNG seeds
    (0..n).map(|_| {
        std::thread::sleep(std::time::Duration::from_nanos(100));
        generate_key()
    }).collect()
}

fn main() {
    println!("=== CD Key Generator ===\n");

    let keys = generate_batch(5);
    for key in &keys {
        println!("  {}  valid={}", key, validate_key(key));
    }

    let mut tampered = keys[0].clone();
    if let Some(c) = tampered.get_mut(0..1) {
        let replacement = if &tampered[0..1] == "Z" { "A" } else { "Z" };
        tampered.replace_range(0..1, replacement);
    }
    println!("\n  Tampered: {}  valid={}", tampered, validate_key(&tampered));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_key_format() {
        let key = generate_key();
        let parts: Vec<&str> = key.split('-').collect();
        assert_eq!(parts.len(), 5);
        for part in &parts {
            assert_eq!(part.len(), 5);
            assert!(part.chars().all(|c| c.is_ascii_uppercase() || c.is_ascii_digit()));
        }
    }

    #[test]
    fn test_validate_generated_key() {
        let key = generate_key();
        assert!(validate_key(&key), "Generated key should be valid: {}", key);
    }

    #[test]
    fn test_tampered_key_invalid() {
        let key = generate_key();
        let mut tampered = key.clone();
        let replacement = if &tampered[0..1] == "Z" { "A" } else { "Z" };
        tampered.replace_range(0..1, replacement);
        assert!(!validate_key(&tampered), "Tampered key should be invalid: {}", tampered);
    }

    #[test]
    fn test_invalid_format() {
        assert!(!validate_key(""));
        assert!(!validate_key("AAAAA-BBBBB"));
        assert!(!validate_key("AAAAA-BBBBB-CCCCC-DDDDD-EEEE"));
        assert!(!validate_key("aaaaa-bbbbb-ccccc-ddddd-!!!!!"));
    }

    #[test]
    fn test_generate_batch() {
        let keys = generate_batch(3);
        assert_eq!(keys.len(), 3);
        for key in &keys {
            assert!(validate_key(key));
        }
    }

    #[test]
    fn test_checksum_deterministic() {
        let groups: Vec<String> = vec![
            "ABCDE".to_string(),
            "FGHIJ".to_string(),
            "KLMNO".to_string(),
            "PQRST".to_string(),
        ];
        let c1 = compute_checksum(&groups);
        let c2 = compute_checksum(&groups);
        assert_eq!(c1, c2);
    }
}
