//! Vending Machine — a finite state machine with coin-constrained change-making.

use std::collections::HashMap;

pub const COINS_CENTS: &[i64] = &[100, 25, 10, 5, 1];

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MachineState {
    Idle,
    Accepting,
}

#[derive(Debug, Clone)]
pub struct Slot {
    pub slot_id: String,
    pub product: String,
    pub price_cents: i64,
    pub quantity: i64,
}

#[derive(Debug)]
pub enum VendingError {
    UnknownSlot(String),
    DuplicateSlot(String),
    InvalidPrice,
    SoldOut(String),
    InsufficientFunds { owed: i64, inserted: i64 },
    CannotMakeChange(i64),
    BadState(String),
    InvalidCoin(i64),
    InvalidCount,
}

pub struct VendingMachine {
    pub slots: HashMap<String, Slot>,
    pub coin_inventory: HashMap<i64, i64>,
    pub inserted_cents: i64,
    pub inserted_coins: HashMap<i64, i64>,
    pub state: MachineState,
}

impl VendingMachine {
    pub fn new() -> Self {
        let coins = COINS_CENTS.iter().map(|&d| (d, 0)).collect();
        let inserted = COINS_CENTS.iter().map(|&d| (d, 0)).collect();
        Self {
            slots: HashMap::new(),
            coin_inventory: coins,
            inserted_cents: 0,
            inserted_coins: inserted,
            state: MachineState::Idle,
        }
    }

    pub fn add_slot(&mut self, slot: Slot) -> Result<(), VendingError> {
        if self.slots.contains_key(&slot.slot_id) {
            return Err(VendingError::DuplicateSlot(slot.slot_id));
        }
        if slot.price_cents <= 0 {
            return Err(VendingError::InvalidPrice);
        }
        self.slots.insert(slot.slot_id.clone(), slot);
        Ok(())
    }

    pub fn load_change(&mut self, denom: i64, count: i64) -> Result<(), VendingError> {
        if !COINS_CENTS.contains(&denom) { return Err(VendingError::InvalidCoin(denom)); }
        if count < 0 { return Err(VendingError::InvalidCount); }
        *self.coin_inventory.entry(denom).or_insert(0) += count;
        Ok(())
    }

    pub fn insert_coin(&mut self, denom: i64) -> Result<i64, VendingError> {
        match self.state {
            MachineState::Idle | MachineState::Accepting => {}
        }
        if !COINS_CENTS.contains(&denom) {
            return Err(VendingError::InvalidCoin(denom));
        }
        self.inserted_cents += denom;
        *self.inserted_coins.entry(denom).or_insert(0) += 1;
        self.state = MachineState::Accepting;
        Ok(self.inserted_cents)
    }

    pub fn refund(&mut self) -> HashMap<i64, i64> {
        let refund: HashMap<i64, i64> = self
            .inserted_coins
            .iter()
            .filter(|&(_, &c)| c > 0)
            .map(|(&d, &c)| (d, c))
            .collect();
        self.inserted_cents = 0;
        for (_, c) in self.inserted_coins.iter_mut() { *c = 0; }
        self.state = MachineState::Idle;
        refund
    }

    pub fn select(&mut self, slot_id: &str) -> Result<(String, HashMap<i64, i64>), VendingError> {
        if self.state != MachineState::Accepting {
            return Err(VendingError::BadState(format!("cannot select while {:?}", self.state)));
        }
        let (product, price_cents) = {
            let slot = self.slots.get(slot_id)
                .ok_or_else(|| VendingError::UnknownSlot(slot_id.to_string()))?;
            if slot.quantity <= 0 {
                return Err(VendingError::SoldOut(slot_id.to_string()));
            }
            if self.inserted_cents < slot.price_cents {
                return Err(VendingError::InsufficientFunds {
                    owed: slot.price_cents, inserted: self.inserted_cents,
                });
            }
            (slot.product.clone(), slot.price_cents)
        };
        let change_owed = self.inserted_cents - price_cents;

        // Pool = machine coins + inserted coins (inserted coins are now the machine's).
        let mut pool: HashMap<i64, i64> = self.coin_inventory.clone();
        for (&d, &c) in &self.inserted_coins {
            *pool.entry(d).or_insert(0) += c;
        }
        let change = greedy_change(change_owed, &pool)
            .ok_or(VendingError::CannotMakeChange(change_owed))?;

        // Commit: subtract change from pool, update inventory, dispense product, reset.
        for (&d, &c) in &change {
            *pool.get_mut(&d).unwrap() -= c;
        }
        self.coin_inventory = pool;
        self.slots.get_mut(slot_id).unwrap().quantity -= 1;
        self.inserted_cents = 0;
        for (_, c) in self.inserted_coins.iter_mut() { *c = 0; }
        self.state = MachineState::Idle;
        Ok((product, change.into_iter().filter(|&(_, c)| c > 0).collect()))
    }
}

impl Default for VendingMachine {
    fn default() -> Self { Self::new() }
}

fn greedy_change(amount: i64, pool: &HashMap<i64, i64>) -> Option<HashMap<i64, i64>> {
    let mut result = HashMap::new();
    let mut remaining = amount;
    let mut denoms: Vec<i64> = pool.keys().cloned().collect();
    denoms.sort_by(|a, b| b.cmp(a));
    for d in denoms {
        if d > remaining { continue; }
        let available = *pool.get(&d).unwrap_or(&0);
        let use_count = available.min(remaining / d);
        if use_count > 0 {
            result.insert(d, use_count);
            remaining -= d * use_count;
        }
        if remaining == 0 { break; }
    }
    if remaining == 0 { Some(result) } else { None }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn slot(id: &str, product: &str, price: i64, qty: i64) -> Slot {
        Slot { slot_id: id.into(), product: product.into(),
               price_cents: price, quantity: qty }
    }

    fn setup() -> VendingMachine {
        let mut m = VendingMachine::new();
        m.add_slot(slot("A1", "Cola", 125, 5)).unwrap();
        m.add_slot(slot("C3", "Gum", 50, 10)).unwrap();
        m.load_change(25, 20).unwrap();
        m.load_change(10, 20).unwrap();
        m.load_change(5, 20).unwrap();
        m.load_change(1, 50).unwrap();
        m
    }

    #[test]
    fn select_refused_in_idle_state() {
        let mut m = setup();
        assert!(matches!(m.select("A1"), Err(VendingError::BadState(_))));
    }

    #[test]
    fn insert_coin_transitions_to_accepting() {
        let mut m = setup();
        assert_eq!(m.state, MachineState::Idle);
        m.insert_coin(25).unwrap();
        assert_eq!(m.state, MachineState::Accepting);
    }

    #[test]
    fn successful_purchase_dispenses_and_returns_change() {
        let mut m = setup();
        m.insert_coin(100).unwrap();
        m.insert_coin(25).unwrap();
        m.insert_coin(25).unwrap();
        let (product, change) = m.select("A1").unwrap();
        assert_eq!(product, "Cola");
        let change_sum: i64 = change.iter().map(|(d, c)| d * c).sum();
        assert_eq!(change_sum, 25);
        assert_eq!(m.state, MachineState::Idle);
        assert_eq!(m.inserted_cents, 0);
    }

    #[test]
    fn insufficient_funds_refused_but_state_preserved() {
        let mut m = setup();
        m.insert_coin(25).unwrap();
        let err = m.select("A1");
        assert!(matches!(err, Err(VendingError::InsufficientFunds { .. })));
        // State stays in Accepting — the user can still insert more or refund.
        assert_eq!(m.state, MachineState::Accepting);
        assert_eq!(m.inserted_cents, 25);
    }

    #[test]
    fn refund_returns_all_inserted_coins_and_resets() {
        let mut m = setup();
        m.insert_coin(100).unwrap();
        m.insert_coin(25).unwrap();
        let refunded = m.refund();
        let total: i64 = refunded.iter().map(|(d, c)| d * c).sum();
        assert_eq!(total, 125);
        assert_eq!(m.state, MachineState::Idle);
    }

    #[test]
    fn cannot_make_change_fails_cleanly() {
        let mut m = VendingMachine::new();
        m.add_slot(slot("G1", "Gum", 30, 10)).unwrap();
        // No small change loaded at all.
        m.insert_coin(100).unwrap();
        assert!(matches!(m.select("G1"), Err(VendingError::CannotMakeChange(_))));
    }
}
