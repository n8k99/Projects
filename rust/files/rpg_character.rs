//! RPG Character Stat Creator — class templates + derived stats + level progression.
//!
//! Twelfth Files project. A character sheet is **base stats + class
//! template + level + derived stats + inventory**. The stats model
//! composes:
//!   * Base stats are rolled (here: deterministic seeded RNG for tests).
//!   * Class applies a multiplier/bonus per stat.
//!   * Derived stats (max HP, max MP, armour) compute from the adjusted
//!      base — never stored, always computed like G095's formulae.
//!   * Leveling spends points across stats, bounded by class caps.
//!
//! This is a pure model: no graphics, no game loop, no combat resolution.
//! The test is that derived stats flow correctly from their inputs and
//! that level progression respects the class contract.

use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Stat { Strength, Dexterity, Intellect, Stamina, Luck }

impl Stat {
    pub const ALL: [Stat; 5] = [
        Stat::Strength, Stat::Dexterity, Stat::Intellect, Stat::Stamina, Stat::Luck,
    ];
    pub fn name(&self) -> &'static str {
        match self {
            Stat::Strength => "strength",
            Stat::Dexterity => "dexterity",
            Stat::Intellect => "intellect",
            Stat::Stamina => "stamina",
            Stat::Luck => "luck",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Class { Warrior, Mage, Rogue }

impl Class {
    pub fn multipliers(&self) -> BTreeMap<Stat, f64> {
        let mut m = BTreeMap::new();
        for s in Stat::ALL { m.insert(s, 1.0); }
        match self {
            Class::Warrior => { m.insert(Stat::Strength, 1.5); m.insert(Stat::Stamina, 1.3); }
            Class::Mage => { m.insert(Stat::Intellect, 1.5); m.insert(Stat::Stamina, 0.8); }
            Class::Rogue => { m.insert(Stat::Dexterity, 1.4); m.insert(Stat::Luck, 1.3); }
        }
        m
    }

    pub fn max_per_stat(&self) -> u32 {
        match self {
            Class::Warrior => 99,
            Class::Mage => 99,
            Class::Rogue => 99,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Character {
    pub name: String,
    pub class: Class,
    pub level: u32,
    pub base: BTreeMap<Stat, u32>,
    pub unspent_points: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Derived {
    pub max_hp: u32,
    pub max_mp: u32,
    pub armour: u32,
    pub carry_capacity: u32,
    pub crit_chance: f64,
}

impl Character {
    /// Roll stats with a seeded LCG for deterministic results.
    pub fn roll(name: &str, class: Class, seed: u64) -> Self {
        let mut rng = LcgRng::new(seed);
        let mut base = BTreeMap::new();
        for s in Stat::ALL {
            // 3d6-ish: 3-18 range.
            let roll = (3 + rng.next_in(0, 16)) as u32;
            base.insert(s, roll);
        }
        Self { name: name.into(), class, level: 1, base, unspent_points: 0 }
    }

    /// Adjusted stat = base * class multiplier, rounded down.
    pub fn adjusted(&self, stat: Stat) -> u32 {
        let base = *self.base.get(&stat).unwrap_or(&0);
        let mult = *self.class.multipliers().get(&stat).unwrap_or(&1.0);
        (base as f64 * mult) as u32
    }

    pub fn derived(&self) -> Derived {
        let str_ = self.adjusted(Stat::Strength);
        let int_ = self.adjusted(Stat::Intellect);
        let sta = self.adjusted(Stat::Stamina);
        let dex = self.adjusted(Stat::Dexterity);
        let luk = self.adjusted(Stat::Luck);

        Derived {
            max_hp: 10 + sta * 5 + self.level * 10,
            max_mp: int_ * 3 + self.level * 5,
            armour: str_ / 3 + sta / 5,
            carry_capacity: 10 + str_ * 2,
            crit_chance: (luk as f64 * 0.5 + dex as f64 * 0.2) / 100.0,
        }
    }

    /// Level up: gain 5 stat points to spend manually.
    pub fn level_up(&mut self) {
        self.level += 1;
        self.unspent_points += 5;
    }

    /// Spend a point on a stat. Enforces per-class cap.
    pub fn spend_point(&mut self, stat: Stat, points: u32) -> Result<(), String> {
        if points > self.unspent_points {
            return Err(format!("not enough points: have {}, need {}",
                               self.unspent_points, points));
        }
        let current = *self.base.get(&stat).unwrap_or(&0);
        let new_val = current + points;
        let cap = self.class.max_per_stat();
        if new_val > cap {
            return Err(format!("stat cap exceeded: {new_val} > {cap}"));
        }
        self.base.insert(stat, new_val);
        self.unspent_points -= points;
        Ok(())
    }

    /// Serialise to a simple line-based format.
    pub fn to_text(&self) -> String {
        let mut out = String::new();
        out.push_str(&format!("name: {}\n", self.name));
        out.push_str(&format!("class: {:?}\n", self.class));
        out.push_str(&format!("level: {}\n", self.level));
        out.push_str(&format!("unspent: {}\n", self.unspent_points));
        for (stat, val) in &self.base {
            out.push_str(&format!("stat:{}={}\n", stat.name(), val));
        }
        out
    }
}

/// Simple linear congruential generator (seed-determined, for tests).
pub struct LcgRng { state: u64 }

impl LcgRng {
    pub fn new(seed: u64) -> Self { Self { state: seed.wrapping_add(1) } }
    pub fn next_u32(&mut self) -> u32 {
        self.state = self.state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        (self.state >> 33) as u32
    }
    pub fn next_in(&mut self, lo: u32, range: u32) -> u32 {
        lo + (self.next_u32() % range)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deterministic_rolls_from_same_seed() {
        let a = Character::roll("Hero", Class::Warrior, 42);
        let b = Character::roll("Hero", Class::Warrior, 42);
        assert_eq!(a.base, b.base);
    }

    #[test]
    fn different_seeds_produce_different_rolls() {
        let a = Character::roll("A", Class::Warrior, 1);
        let b = Character::roll("B", Class::Warrior, 2);
        // Not a guarantee in general but true for the LCG used here.
        assert_ne!(a.base, b.base);
    }

    #[test]
    fn warrior_has_strength_multiplier() {
        let c = Character::roll("W", Class::Warrior, 100);
        let base_str = *c.base.get(&Stat::Strength).unwrap();
        let adj = c.adjusted(Stat::Strength);
        assert_eq!(adj, (base_str as f64 * 1.5) as u32);
    }

    #[test]
    fn mage_has_intellect_bonus_and_stamina_penalty() {
        let c = Character::roll("M", Class::Mage, 200);
        let base_int = *c.base.get(&Stat::Intellect).unwrap();
        let base_sta = *c.base.get(&Stat::Stamina).unwrap();
        assert!(c.adjusted(Stat::Intellect) > base_int);
        assert!(c.adjusted(Stat::Stamina) < base_sta);
    }

    #[test]
    fn derived_hp_grows_with_stamina_and_level() {
        let mut c = Character::roll("A", Class::Warrior, 7);
        let hp1 = c.derived().max_hp;
        c.level_up();
        let hp2 = c.derived().max_hp;
        assert_eq!(hp2 - hp1, 10);
    }

    #[test]
    fn derived_mp_grows_with_intellect() {
        let mut c = Character::roll("M", Class::Mage, 50);
        let mp_before = c.derived().max_mp;
        let current_int = *c.base.get(&Stat::Intellect).unwrap();
        c.base.insert(Stat::Intellect, current_int + 10);
        let mp_after = c.derived().max_mp;
        assert!(mp_after > mp_before);
    }

    #[test]
    fn level_up_grants_five_points() {
        let mut c = Character::roll("X", Class::Rogue, 3);
        let before = c.unspent_points;
        c.level_up();
        assert_eq!(c.unspent_points, before + 5);
        assert_eq!(c.level, 2);
    }

    #[test]
    fn spend_point_adds_to_stat() {
        let mut c = Character::roll("X", Class::Warrior, 12);
        c.unspent_points = 5;
        let before = *c.base.get(&Stat::Strength).unwrap();
        c.spend_point(Stat::Strength, 3).unwrap();
        assert_eq!(*c.base.get(&Stat::Strength).unwrap(), before + 3);
        assert_eq!(c.unspent_points, 2);
    }

    #[test]
    fn spend_point_errors_when_insufficient() {
        let mut c = Character::roll("X", Class::Mage, 1);
        c.unspent_points = 2;
        let err = c.spend_point(Stat::Strength, 5);
        assert!(err.is_err());
    }

    #[test]
    fn spend_point_errors_at_cap() {
        let mut c = Character::roll("X", Class::Warrior, 1);
        c.base.insert(Stat::Strength, 97);
        c.unspent_points = 10;
        let err = c.spend_point(Stat::Strength, 5);
        assert!(err.is_err());
    }

    #[test]
    fn to_text_includes_all_stats() {
        let c = Character::roll("Test", Class::Warrior, 42);
        let text = c.to_text();
        for stat in Stat::ALL {
            assert!(text.contains(stat.name()), "missing {}", stat.name());
        }
    }

    #[test]
    fn crit_chance_scales_with_luck_and_dexterity() {
        let mut c = Character::roll("R", Class::Rogue, 11);
        let before = c.derived().crit_chance;
        let luck = *c.base.get(&Stat::Luck).unwrap();
        c.base.insert(Stat::Luck, luck + 20);
        let after = c.derived().crit_chance;
        assert!(after > before);
    }

    #[test]
    fn carry_capacity_scales_with_strength() {
        let c1 = Character { name: "a".into(), class: Class::Warrior, level: 1,
                              base: [(Stat::Strength, 10), (Stat::Dexterity, 10),
                                     (Stat::Intellect, 10), (Stat::Stamina, 10),
                                     (Stat::Luck, 10)].iter().cloned().collect(),
                              unspent_points: 0 };
        let c2 = Character { name: "b".into(), class: Class::Warrior, level: 1,
                              base: [(Stat::Strength, 20), (Stat::Dexterity, 10),
                                     (Stat::Intellect, 10), (Stat::Stamina, 10),
                                     (Stat::Luck, 10)].iter().cloned().collect(),
                              unspent_points: 0 };
        assert!(c2.derived().carry_capacity > c1.derived().carry_capacity);
    }
}
