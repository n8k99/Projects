//! Random Gift Suggestions — attribute-based gift recommendation.

use std::collections::HashSet;

/// A gift with category, price range, and compatible traits.
#[derive(Debug, Clone)]
pub struct Gift {
    pub name: String,
    pub category: String,
    pub price_range: String, // "budget", "mid", "premium"
    pub suitable_for: Vec<String>,
}

impl std::fmt::Display for Gift {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} [{}] ({}) — suits: {}",
            self.name,
            self.category,
            self.price_range,
            self.suitable_for.join(", ")
        )
    }
}

/// Attribute-based gift recommendation engine.
pub struct GiftSuggester {
    gifts: Vec<Gift>,
}

impl GiftSuggester {
    /// Create a new suggester pre-loaded with default gifts.
    pub fn new() -> Self {
        let mut gs = Self { gifts: Vec::new() };
        gs.load_defaults();
        gs
    }

    fn load_defaults(&mut self) {
        let defaults = vec![
            // Tech
            ("Mechanical Keyboard", "tech", "mid", vec!["techie", "creative"]),
            ("Raspberry Pi Kit", "tech", "budget", vec!["techie", "creative"]),
            ("Noise-Cancelling Headphones", "tech", "premium", vec!["techie", "music-lover"]),
            ("Smart Home Starter Kit", "tech", "mid", vec!["techie"]),
            // Books
            ("Leather-Bound Journal", "books", "mid", vec!["bookworm", "creative"]),
            ("Complete Tolkien Collection", "books", "premium", vec!["bookworm", "adventurer"]),
            ("Pocket Poetry Anthology", "books", "budget", vec!["bookworm", "creative"]),
            ("Cookbook: World Cuisines", "books", "mid", vec!["bookworm", "foodie"]),
            // Outdoor
            ("Hammock", "outdoor", "budget", vec!["adventurer"]),
            ("Hiking Backpack", "outdoor", "mid", vec!["adventurer"]),
            ("Camping Cookset", "outdoor", "mid", vec!["adventurer", "foodie"]),
            ("Trail Running Shoes", "outdoor", "premium", vec!["adventurer"]),
            // Cooking
            ("Cast Iron Skillet", "cooking", "budget", vec!["foodie"]),
            ("Spice Collection Box", "cooking", "mid", vec!["foodie", "adventurer"]),
            ("Chef's Knife Set", "cooking", "premium", vec!["foodie"]),
            ("Pasta Maker", "cooking", "mid", vec!["foodie", "creative"]),
            // Music
            ("Vinyl Record Starter Pack", "music", "budget", vec!["music-lover"]),
            ("Concert Tickets", "music", "mid", vec!["music-lover", "adventurer"]),
            ("MIDI Controller", "music", "mid", vec!["music-lover", "techie", "creative"]),
            ("Turntable", "music", "premium", vec!["music-lover"]),
            // Art
            ("Watercolor Set", "art", "budget", vec!["creative"]),
            ("Drawing Tablet", "art", "mid", vec!["creative", "techie"]),
            ("Museum Membership", "art", "mid", vec!["creative", "bookworm"]),
            ("Oil Paint Master Set", "art", "premium", vec!["creative"]),
        ];
        for (name, cat, price, traits) in defaults {
            self.gifts.push(Gift {
                name: name.to_string(),
                category: cat.to_string(),
                price_range: price.to_string(),
                suitable_for: traits.into_iter().map(String::from).collect(),
            });
        }
    }

    /// Add a gift to the catalog.
    pub fn add_gift(&mut self, name: &str, category: &str, price_range: &str, suitable_for: &[&str]) {
        self.gifts.push(Gift {
            name: name.to_string(),
            category: category.to_string(),
            price_range: price_range.to_string(),
            suitable_for: suitable_for.iter().map(|s| s.to_string()).collect(),
        });
    }

    /// Suggest gifts matching traits, sorted by relevance (descending).
    /// If budget is Some, only gifts in that price range are returned.
    pub fn suggest(&self, traits: &[&str], budget: Option<&str>) -> Vec<&Gift> {
        let trait_set: HashSet<String> = traits.iter().map(|t| t.to_lowercase()).collect();
        let mut scored: Vec<(usize, &Gift)> = self
            .gifts
            .iter()
            .filter(|g| budget.map_or(true, |b| g.price_range == b))
            .filter_map(|g| {
                let gift_traits: HashSet<String> =
                    g.suitable_for.iter().map(|s| s.to_lowercase()).collect();
                let matches = trait_set.intersection(&gift_traits).count();
                if matches > 0 { Some((matches, g)) } else { None }
            })
            .collect();
        scored.sort_by(|a, b| b.0.cmp(&a.0));
        scored.into_iter().map(|(_, g)| g).collect()
    }

    /// Return a random gift from the catalog.
    pub fn random_suggestion(&self) -> Option<&Gift> {
        if self.gifts.is_empty() {
            return None;
        }
        use std::time::SystemTime;
        let seed = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .subsec_nanos() as usize;
        Some(&self.gifts[seed % self.gifts.len()])
    }

    /// Return all gifts in a given category.
    pub fn suggest_by_category(&self, category: &str) -> Vec<&Gift> {
        let cat = category.to_lowercase();
        self.gifts.iter().filter(|g| g.category.to_lowercase() == cat).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_catalog_size() {
        let gs = GiftSuggester::new();
        assert_eq!(gs.gifts.len(), 24);
    }

    #[test]
    fn test_suggest_single_trait() {
        let gs = GiftSuggester::new();
        let results = gs.suggest(&["foodie"], None);
        assert!(!results.is_empty());
        for gift in &results {
            assert!(gift.suitable_for.iter().any(|t| t == "foodie"));
        }
    }

    #[test]
    fn test_suggest_with_budget() {
        let gs = GiftSuggester::new();
        let results = gs.suggest(&["techie"], Some("budget"));
        for gift in &results {
            assert_eq!(gift.price_range, "budget");
        }
    }

    #[test]
    fn test_suggest_relevance_order() {
        let gs = GiftSuggester::new();
        let results = gs.suggest(&["techie", "music-lover", "creative"], None);
        assert!(!results.is_empty());
        assert_eq!(results[0].name, "MIDI Controller");
    }

    #[test]
    fn test_suggest_by_category() {
        let gs = GiftSuggester::new();
        let cooking = gs.suggest_by_category("cooking");
        assert_eq!(cooking.len(), 4);
        for gift in &cooking {
            assert_eq!(gift.category, "cooking");
        }
    }

    #[test]
    fn test_add_gift() {
        let mut gs = GiftSuggester::new();
        gs.add_gift("Custom Widget", "tech", "premium", &["techie"]);
        assert_eq!(gs.gifts.len(), 25);
        let results = gs.suggest_by_category("tech");
        assert!(results.iter().any(|g| g.name == "Custom Widget"));
    }

    #[test]
    fn test_random_suggestion() {
        let gs = GiftSuggester::new();
        assert!(gs.random_suggestion().is_some());
    }

    #[test]
    fn test_no_matches() {
        let gs = GiftSuggester::new();
        let results = gs.suggest(&["nonexistent-trait"], None);
        assert!(results.is_empty());
    }
}
