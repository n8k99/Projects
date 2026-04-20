//! Card Collector — multiset inventory with valuation and trade balancing.
//!
//! Third Databases project. Introduces **multiset semantics**: a
//! collection holds multiple copies of the same card, each at a recorded
//! condition. Card identity is (set_code, number); the same card in
//! Mint and Played condition are the same card, different inventory
//! entries.
//!
//! Distinctive moves:
//!   * **Multiset inventory** — `card_id → count` (by condition).
//!   * **Valuation function** — `base_price * rarity_multiplier * condition_multiplier`.
//!   * **Trade balance** — two-sided bundle comparison with fairness verdict.
//!   * **Missing list** — set difference of full catalogue vs owned.

use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Rarity { Common, Uncommon, Rare, Mythic }

impl Rarity {
    pub fn multiplier(&self) -> f64 {
        match self {
            Rarity::Common => 1.0,
            Rarity::Uncommon => 2.0,
            Rarity::Rare => 8.0,
            Rarity::Mythic => 25.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Condition { Mint, NearMint, Played, Damaged }

impl Condition {
    pub fn multiplier(&self) -> f64 {
        match self {
            Condition::Mint => 1.0,
            Condition::NearMint => 0.8,
            Condition::Played => 0.4,
            Condition::Damaged => 0.15,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Card {
    pub set_code: String,
    pub number: u32,
    pub name: String,
    pub rarity: Rarity,
    pub base_price_cents: u32,  // baseline in mint, common-adjusted pricing
}

impl Card {
    pub fn id(&self) -> String {
        format!("{}-{:03}", self.set_code, self.number)
    }
    pub fn value_cents(&self, condition: Condition) -> u32 {
        let v = self.base_price_cents as f64
            * self.rarity.multiplier()
            * condition.multiplier();
        v.round() as u32
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct InventoryKey {
    pub card_id: String,
    pub condition: Condition,
}

#[derive(Debug, Clone)]
pub struct Catalogue {
    pub cards: BTreeMap<String, Card>,  // card_id → card
}

impl Catalogue {
    pub fn new() -> Self { Self { cards: BTreeMap::new() } }
    pub fn add(&mut self, card: Card) {
        self.cards.insert(card.id(), card);
    }
    pub fn get(&self, card_id: &str) -> Option<&Card> {
        self.cards.get(card_id)
    }
    pub fn all_ids(&self) -> Vec<String> {
        self.cards.keys().cloned().collect()
    }
}

impl Default for Catalogue { fn default() -> Self { Self::new() } }

#[derive(Debug, Clone)]
pub struct Collection {
    pub counts: BTreeMap<InventoryKey, u32>,
}

impl Collection {
    pub fn new() -> Self { Self { counts: BTreeMap::new() } }

    pub fn add(&mut self, card_id: &str, condition: Condition, qty: u32) {
        let key = InventoryKey { card_id: card_id.into(), condition };
        *self.counts.entry(key).or_insert(0) += qty;
    }

    pub fn remove(&mut self, card_id: &str, condition: Condition, qty: u32) -> Result<(), String> {
        let key = InventoryKey { card_id: card_id.into(), condition };
        let current = self.counts.get(&key).copied().unwrap_or(0);
        if current < qty {
            return Err(format!("not enough {} {}× — have {current}, need {qty}",
                card_id, match condition {
                    Condition::Mint => "M",
                    Condition::NearMint => "NM",
                    Condition::Played => "P",
                    Condition::Damaged => "D",
                }));
        }
        if current == qty {
            self.counts.remove(&key);
        } else {
            self.counts.insert(key, current - qty);
        }
        Ok(())
    }

    pub fn total_cards(&self) -> u32 {
        self.counts.values().sum()
    }

    pub fn unique_cards(&self) -> usize {
        let mut ids = std::collections::BTreeSet::new();
        for k in self.counts.keys() { ids.insert(k.card_id.clone()); }
        ids.len()
    }

    pub fn total_value_cents(&self, catalogue: &Catalogue) -> u32 {
        let mut total = 0;
        for (key, count) in &self.counts {
            if let Some(card) = catalogue.get(&key.card_id) {
                total += card.value_cents(key.condition) * count;
            }
        }
        total
    }

    /// Cards in catalogue not present (in any condition) in this collection.
    pub fn missing(&self, catalogue: &Catalogue) -> Vec<String> {
        let owned_ids: std::collections::BTreeSet<String> =
            self.counts.keys().map(|k| k.card_id.clone()).collect();
        catalogue.all_ids().into_iter()
            .filter(|id| !owned_ids.contains(id))
            .collect()
    }
}

impl Default for Collection { fn default() -> Self { Self::new() } }

#[derive(Debug, Clone)]
pub struct TradeBundle {
    pub cards: Vec<(String, Condition, u32)>,  // (card_id, condition, qty)
}

impl TradeBundle {
    pub fn new() -> Self { Self { cards: Vec::new() } }
    pub fn add(&mut self, card_id: &str, condition: Condition, qty: u32) {
        self.cards.push((card_id.into(), condition, qty));
    }
    pub fn value_cents(&self, catalogue: &Catalogue) -> u32 {
        let mut total = 0;
        for (id, cond, qty) in &self.cards {
            if let Some(c) = catalogue.get(id) {
                total += c.value_cents(*cond) * qty;
            }
        }
        total
    }
}

impl Default for TradeBundle { fn default() -> Self { Self::new() } }

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TradeVerdict {
    Fair,
    OfferedMoreValuable { diff_cents: u32 },
    AskedMoreValuable { diff_cents: u32 },
}

/// Evaluate a trade. Fair if values within `tolerance_cents` of each other.
pub fn evaluate_trade(
    offered: &TradeBundle,
    asked: &TradeBundle,
    catalogue: &Catalogue,
    tolerance_cents: u32,
) -> TradeVerdict {
    let o = offered.value_cents(catalogue);
    let a = asked.value_cents(catalogue);
    if o >= a && o - a <= tolerance_cents { return TradeVerdict::Fair; }
    if a >= o && a - o <= tolerance_cents { return TradeVerdict::Fair; }
    if o > a { TradeVerdict::OfferedMoreValuable { diff_cents: o - a } }
    else { TradeVerdict::AskedMoreValuable { diff_cents: a - o } }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_catalogue() -> Catalogue {
        let mut c = Catalogue::new();
        c.add(Card {
            set_code: "DPN".into(), number: 1, name: "Common One".into(),
            rarity: Rarity::Common, base_price_cents: 25,
        });
        c.add(Card {
            set_code: "DPN".into(), number: 2, name: "Uncommon Two".into(),
            rarity: Rarity::Uncommon, base_price_cents: 25,
        });
        c.add(Card {
            set_code: "DPN".into(), number: 3, name: "Rare Three".into(),
            rarity: Rarity::Rare, base_price_cents: 25,
        });
        c.add(Card {
            set_code: "DPN".into(), number: 4, name: "Mythic Four".into(),
            rarity: Rarity::Mythic, base_price_cents: 100,
        });
        c
    }

    #[test]
    fn card_id_format() {
        let c = Card { set_code: "DPN".into(), number: 42, name: "x".into(),
                       rarity: Rarity::Common, base_price_cents: 10 };
        assert_eq!(c.id(), "DPN-042");
    }

    #[test]
    fn card_value_scales_with_rarity() {
        let cat = sample_catalogue();
        assert_eq!(cat.get("DPN-001").unwrap().value_cents(Condition::Mint), 25);
        assert_eq!(cat.get("DPN-002").unwrap().value_cents(Condition::Mint), 50);
        assert_eq!(cat.get("DPN-003").unwrap().value_cents(Condition::Mint), 200);
        assert_eq!(cat.get("DPN-004").unwrap().value_cents(Condition::Mint), 2500);
    }

    #[test]
    fn card_value_scales_with_condition() {
        let cat = sample_catalogue();
        let rare = cat.get("DPN-003").unwrap();
        assert_eq!(rare.value_cents(Condition::Mint), 200);
        assert_eq!(rare.value_cents(Condition::NearMint), 160);
        assert_eq!(rare.value_cents(Condition::Played), 80);
        assert_eq!(rare.value_cents(Condition::Damaged), 30);
    }

    #[test]
    fn collection_add_and_total() {
        let mut c = Collection::new();
        c.add("DPN-001", Condition::Mint, 3);
        c.add("DPN-002", Condition::NearMint, 2);
        assert_eq!(c.total_cards(), 5);
        assert_eq!(c.unique_cards(), 2);
    }

    #[test]
    fn collection_add_same_key_stacks() {
        let mut c = Collection::new();
        c.add("DPN-001", Condition::Mint, 2);
        c.add("DPN-001", Condition::Mint, 3);
        assert_eq!(c.total_cards(), 5);
        assert_eq!(c.unique_cards(), 1);
    }

    #[test]
    fn collection_different_conditions_are_separate() {
        let mut c = Collection::new();
        c.add("DPN-001", Condition::Mint, 2);
        c.add("DPN-001", Condition::Played, 3);
        assert_eq!(c.total_cards(), 5);
        assert_eq!(c.unique_cards(), 1);  // same card_id
        assert_eq!(c.counts.len(), 2);     // two distinct inventory keys
    }

    #[test]
    fn collection_remove_decreases_count() {
        let mut c = Collection::new();
        c.add("DPN-001", Condition::Mint, 5);
        c.remove("DPN-001", Condition::Mint, 2).unwrap();
        assert_eq!(c.total_cards(), 3);
    }

    #[test]
    fn collection_remove_to_zero_drops_key() {
        let mut c = Collection::new();
        c.add("DPN-001", Condition::Mint, 2);
        c.remove("DPN-001", Condition::Mint, 2).unwrap();
        assert_eq!(c.counts.len(), 0);
    }

    #[test]
    fn collection_remove_too_many_errors() {
        let mut c = Collection::new();
        c.add("DPN-001", Condition::Mint, 1);
        assert!(c.remove("DPN-001", Condition::Mint, 2).is_err());
    }

    #[test]
    fn collection_total_value() {
        let cat = sample_catalogue();
        let mut c = Collection::new();
        c.add("DPN-001", Condition::Mint, 4);       // 25 × 4 = 100
        c.add("DPN-003", Condition::NearMint, 1);   // 160 × 1 = 160
        c.add("DPN-004", Condition::Mint, 1);       // 2500
        assert_eq!(c.total_value_cents(&cat), 2760);
    }

    #[test]
    fn collection_missing_lists_unowned() {
        let cat = sample_catalogue();
        let mut c = Collection::new();
        c.add("DPN-001", Condition::Mint, 1);
        c.add("DPN-004", Condition::Played, 1);
        let missing = c.missing(&cat);
        assert_eq!(missing, vec!["DPN-002", "DPN-003"]);
    }

    #[test]
    fn trade_bundle_value() {
        let cat = sample_catalogue();
        let mut b = TradeBundle::new();
        b.add("DPN-004", Condition::Mint, 1);  // 2500
        b.add("DPN-001", Condition::Mint, 2);  // 50
        assert_eq!(b.value_cents(&cat), 2550);
    }

    #[test]
    fn trade_fair_when_values_match() {
        let cat = sample_catalogue();
        let mut offered = TradeBundle::new();
        let mut asked = TradeBundle::new();
        offered.add("DPN-004", Condition::Mint, 1);  // 2500
        asked.add("DPN-003", Condition::Mint, 12);   // 200 × 12 = 2400
        asked.add("DPN-001", Condition::Mint, 4);    // 25 × 4 = 100
        assert_eq!(evaluate_trade(&offered, &asked, &cat, 100), TradeVerdict::Fair);
    }

    #[test]
    fn trade_offered_more_valuable() {
        let cat = sample_catalogue();
        let mut offered = TradeBundle::new();
        let mut asked = TradeBundle::new();
        offered.add("DPN-004", Condition::Mint, 1);  // 2500
        asked.add("DPN-001", Condition::Mint, 1);    // 25
        let verdict = evaluate_trade(&offered, &asked, &cat, 10);
        assert!(matches!(verdict,
            TradeVerdict::OfferedMoreValuable { diff_cents: 2475 }));
    }

    #[test]
    fn trade_asked_more_valuable() {
        let cat = sample_catalogue();
        let mut offered = TradeBundle::new();
        let mut asked = TradeBundle::new();
        offered.add("DPN-001", Condition::Mint, 1);
        asked.add("DPN-004", Condition::Mint, 1);
        let verdict = evaluate_trade(&offered, &asked, &cat, 10);
        assert!(matches!(verdict,
            TradeVerdict::AskedMoreValuable { diff_cents: 2475 }));
    }
}
