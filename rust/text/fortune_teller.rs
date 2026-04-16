//! Fortune Teller — randomized wisdom from categorized pools.

use std::collections::HashMap;

/// The valid fortune categories.
const CATEGORIES: &[&str] = &["wisdom", "humor", "warning", "motivation", "mystery"];

/// A fortune/prediction generator with categorized pools of wisdom.
pub struct FortuneTeller {
    pools: HashMap<String, Vec<String>>,
}

impl FortuneTeller {
    /// Create a new fortune teller pre-loaded with default fortunes.
    pub fn new() -> Self {
        let mut ft = Self {
            pools: CATEGORIES.iter().map(|&c| (c.to_string(), Vec::new())).collect(),
        };
        ft.load_defaults();
        ft
    }

    fn load_defaults(&mut self) {
        let defaults: &[(&str, &[&str])] = &[
            ("wisdom", &[
                "The obstacle is the path.",
                "What you resist persists.",
                "Still water runs deep.",
                "The map is not the territory.",
                "Every expert was once a beginner.",
            ]),
            ("humor", &[
                "You will step on a Lego in the dark tonight.",
                "A closed mouth gathers no foot.",
                "Today is a good day to avoid making eye contact.",
                "Your socks will never match again.",
            ]),
            ("warning", &[
                "Beware the shortcut that saves no time.",
                "Not every open door is an invitation.",
                "The comfortable path leads to the boring destination.",
                "Trust your instincts — they remember what you forgot.",
            ]),
            ("motivation", &[
                "You are closer than you think.",
                "The best time to start was yesterday. The second best is now.",
                "Doubt kills more dreams than failure ever will.",
                "Small steps still move you forward.",
                "Discipline is choosing what you want most over what you want now.",
            ]),
            ("mystery", &[
                "Something lost will find you when you stop looking.",
                "The answer you seek is in a room you haven't entered yet.",
                "A stranger already knows your name.",
                "The next full moon brings clarity.",
            ]),
        ];

        for (cat, fortunes) in defaults {
            if let Some(pool) = self.pools.get_mut(*cat) {
                pool.extend(fortunes.iter().map(|s| s.to_string()));
            }
        }
    }

    /// Return a random fortune from all categories.
    pub fn tell_fortune(&self) -> &str {
        let all: Vec<&str> = self.pools.values()
            .flat_map(|v| v.iter().map(|s| s.as_str()))
            .collect();
        if all.is_empty() {
            return "The spirits are silent.";
        }
        let idx = pseudo_random(all.len());
        all[idx]
    }

    /// Return a random fortune from the specified category.
    pub fn tell_fortune_by_category(&self, category: &str) -> Option<&str> {
        let pool = self.pools.get(category)?;
        if pool.is_empty() {
            return None;
        }
        let idx = pseudo_random(pool.len());
        Some(&pool[idx])
    }

    /// Add a fortune to a category pool.
    pub fn add_fortune(&mut self, text: &str, category: &str) {
        self.pools
            .entry(category.to_string())
            .or_insert_with(Vec::new)
            .push(text.to_string());
    }

    /// Acknowledge the question, then return a random fortune regardless.
    pub fn ask_question(&self, _question: &str) -> &str {
        self.tell_fortune()
    }

    /// Return the total number of fortunes across all categories.
    pub fn fortune_count(&self) -> usize {
        self.pools.values().map(|v| v.len()).sum()
    }
}

/// Simple pseudo-random index (no external crate dependency).
fn pseudo_random(bound: usize) -> usize {
    use std::time::SystemTime;
    let seed = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos() as usize;
    seed % bound
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_pool_count() {
        let ft = FortuneTeller::new();
        // 5 + 4 + 4 + 5 + 4 = 22
        assert_eq!(ft.fortune_count(), 22);
    }

    #[test]
    fn test_tell_fortune_returns_something() {
        let ft = FortuneTeller::new();
        let fortune = ft.tell_fortune();
        assert!(!fortune.is_empty());
    }

    #[test]
    fn test_tell_fortune_by_category() {
        let ft = FortuneTeller::new();
        assert!(ft.tell_fortune_by_category("wisdom").is_some());
        assert!(ft.tell_fortune_by_category("nonexistent").is_none());
    }

    #[test]
    fn test_add_fortune() {
        let mut ft = FortuneTeller::new();
        let before = ft.fortune_count();
        ft.add_fortune("New fortune.", "wisdom");
        assert_eq!(ft.fortune_count(), before + 1);
    }

    #[test]
    fn test_add_fortune_new_category() {
        let mut ft = FortuneTeller::new();
        ft.add_fortune("Custom pool.", "custom");
        assert!(ft.tell_fortune_by_category("custom").is_some());
    }

    #[test]
    fn test_ask_question() {
        let ft = FortuneTeller::new();
        let answer = ft.ask_question("Will it rain?");
        assert!(!answer.is_empty());
    }
}
