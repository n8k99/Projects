//! Flower Shop Ordering — build-up/commit cart with frozen prices and reversible orders.

use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OrderStatus {
    Draft,
    Placed,
    Fulfilled,
    Cancelled,
}

#[derive(Debug, Clone)]
pub struct Flower {
    pub sku: String,
    pub name: String,
    pub unit_price_cents: i64,
    pub stock: i64,
}

#[derive(Debug, Clone)]
pub struct CartLine {
    pub sku: String,
    pub quantity: i64,
    /// Frozen at add time.
    pub unit_price_cents: i64,
}

impl CartLine {
    pub fn line_total_cents(&self) -> i64 {
        self.quantity * self.unit_price_cents
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiscountKind {
    PercentageOff,
    FixedOff,
    FreeShipping,
}

#[derive(Debug, Clone)]
pub struct DiscountRule {
    pub kind: DiscountKind,
    pub value: i64,
    pub min_subtotal_cents: i64,
    pub note: String,
}

impl DiscountRule {
    pub fn discount_cents(&self, subtotal: i64, shipping: i64) -> i64 {
        if subtotal < self.min_subtotal_cents {
            return 0;
        }
        match self.kind {
            DiscountKind::PercentageOff => subtotal * self.value / 100,
            DiscountKind::FixedOff => self.value.min(subtotal),
            DiscountKind::FreeShipping => if subtotal >= self.value { shipping } else { 0 },
        }
    }
}

#[derive(Debug, Clone)]
pub struct Cart {
    pub id: String,
    pub owner_id: String,
    pub lines: Vec<CartLine>,
    pub discount_rules: Vec<DiscountRule>,
    pub shipping_cents: i64,
    pub tax_rate_bps: i64,
    pub status: OrderStatus,
}

#[derive(Debug, Clone)]
pub struct Order {
    pub id: u64,
    pub owner_id: String,
    pub lines: Vec<CartLine>,
    pub discount_rules: Vec<DiscountRule>,
    pub shipping_cents: i64,
    pub tax_rate_bps: i64,
    pub status: OrderStatus,
}

#[derive(Debug)]
pub enum ShopError {
    UnknownFlower(String),
    UnknownCart(String),
    UnknownOrder(u64),
    DuplicateFlower(String),
    DuplicateCart(String),
    InvalidQuantity,
    OutOfStock { sku: String, available: i64, requested: i64 },
    BadTransition(String),
    EmptyCart,
    NotInCart(String),
}

pub struct Shop {
    pub flowers: HashMap<String, Flower>,
    pub carts: HashMap<String, Cart>,
    pub orders: HashMap<u64, Order>,
    next_order_id: u64,
}

impl Shop {
    pub fn new() -> Self {
        Self {
            flowers: HashMap::new(),
            carts: HashMap::new(),
            orders: HashMap::new(),
            next_order_id: 1,
        }
    }

    pub fn add_flower(&mut self, f: Flower) -> Result<(), ShopError> {
        if self.flowers.contains_key(&f.sku) {
            return Err(ShopError::DuplicateFlower(f.sku));
        }
        self.flowers.insert(f.sku.clone(), f);
        Ok(())
    }

    pub fn stock(&mut self, sku: &str, quantity: i64) -> Result<(), ShopError> {
        if quantity < 0 { return Err(ShopError::InvalidQuantity); }
        let f = self
            .flowers
            .get_mut(sku)
            .ok_or_else(|| ShopError::UnknownFlower(sku.to_string()))?;
        f.stock += quantity;
        Ok(())
    }

    pub fn new_cart(&mut self, cart_id: &str, owner_id: &str,
                    shipping_cents: i64, tax_rate_bps: i64) -> Result<(), ShopError> {
        if self.carts.contains_key(cart_id) {
            return Err(ShopError::DuplicateCart(cart_id.to_string()));
        }
        self.carts.insert(cart_id.to_string(), Cart {
            id: cart_id.to_string(),
            owner_id: owner_id.to_string(),
            lines: Vec::new(),
            discount_rules: Vec::new(),
            shipping_cents, tax_rate_bps,
            status: OrderStatus::Draft,
        });
        Ok(())
    }

    fn require_draft<'a>(&'a mut self, cart_id: &str) -> Result<&'a mut Cart, ShopError> {
        let cart = self
            .carts
            .get_mut(cart_id)
            .ok_or_else(|| ShopError::UnknownCart(cart_id.to_string()))?;
        if cart.status != OrderStatus::Draft {
            return Err(ShopError::BadTransition(
                format!("cart {cart_id} is not draft")));
        }
        Ok(cart)
    }

    pub fn add_to_cart(&mut self, cart_id: &str, sku: &str, quantity: i64) -> Result<(), ShopError> {
        if quantity <= 0 { return Err(ShopError::InvalidQuantity); }
        let flower = self
            .flowers
            .get(sku)
            .ok_or_else(|| ShopError::UnknownFlower(sku.to_string()))?
            .clone();
        let cart = self.require_draft(cart_id)?;
        let already: i64 = cart.lines.iter().filter(|l| l.sku == sku).map(|l| l.quantity).sum();
        if flower.stock < already + quantity {
            return Err(ShopError::OutOfStock {
                sku: sku.to_string(),
                available: flower.stock,
                requested: already + quantity,
            });
        }
        if let Some(line) = cart.lines.iter_mut().find(|l| l.sku == sku) {
            line.quantity += quantity;
            return Ok(());
        }
        cart.lines.push(CartLine {
            sku: sku.to_string(),
            quantity,
            unit_price_cents: flower.unit_price_cents,
        });
        Ok(())
    }

    pub fn update_quantity(&mut self, cart_id: &str, sku: &str, new_quantity: i64) -> Result<(), ShopError> {
        if new_quantity < 0 { return Err(ShopError::InvalidQuantity); }
        let stock = self.flowers.get(sku)
            .ok_or_else(|| ShopError::UnknownFlower(sku.to_string()))?
            .stock;
        let cart = self.require_draft(cart_id)?;
        if new_quantity > 0 && stock < new_quantity {
            return Err(ShopError::OutOfStock {
                sku: sku.to_string(),
                available: stock,
                requested: new_quantity,
            });
        }
        if let Some(pos) = cart.lines.iter().position(|l| l.sku == sku) {
            if new_quantity == 0 {
                cart.lines.remove(pos);
            } else {
                cart.lines[pos].quantity = new_quantity;
            }
            Ok(())
        } else {
            Err(ShopError::NotInCart(sku.to_string()))
        }
    }

    pub fn apply_discount(&mut self, cart_id: &str, rule: DiscountRule) -> Result<(), ShopError> {
        let cart = self.require_draft(cart_id)?;
        cart.discount_rules.push(rule);
        Ok(())
    }

    // --- totals ---

    pub fn subtotal_cents(lines: &[CartLine]) -> i64 {
        lines.iter().map(|l| l.line_total_cents()).sum()
    }

    fn discount_for(rules: &[DiscountRule], subtotal: i64, shipping: i64) -> i64 {
        rules.iter().map(|r| r.discount_cents(subtotal, shipping)).sum()
    }

    fn tax_for(lines: &[CartLine], rules: &[DiscountRule], bps: i64, shipping: i64) -> i64 {
        let sub = Self::subtotal_cents(lines);
        let disc = Self::discount_for(rules, sub, shipping);
        let taxable = (sub - disc).max(0);
        taxable * bps / 10_000
    }

    pub fn cart_total_cents(cart: &Cart) -> i64 {
        let sub = Self::subtotal_cents(&cart.lines);
        let disc = Self::discount_for(&cart.discount_rules, sub, cart.shipping_cents);
        let tax = Self::tax_for(&cart.lines, &cart.discount_rules, cart.tax_rate_bps, cart.shipping_cents);
        (sub - disc + tax + cart.shipping_cents).max(0)
    }

    pub fn order_total_cents(order: &Order) -> i64 {
        let sub = Self::subtotal_cents(&order.lines);
        let disc = Self::discount_for(&order.discount_rules, sub, order.shipping_cents);
        let tax = Self::tax_for(&order.lines, &order.discount_rules, order.tax_rate_bps, order.shipping_cents);
        (sub - disc + tax + order.shipping_cents).max(0)
    }

    pub fn checkout(&mut self, cart_id: &str) -> Result<u64, ShopError> {
        let (lines, owner_id, discount_rules, shipping_cents, tax_rate_bps) = {
            let cart = self.require_draft(cart_id)?;
            if cart.lines.is_empty() { return Err(ShopError::EmptyCart); }
            (
                cart.lines.clone(),
                cart.owner_id.clone(),
                cart.discount_rules.clone(),
                cart.shipping_cents,
                cart.tax_rate_bps,
            )
        };
        // Pre-flight inventory check across all lines at once.
        for line in &lines {
            let f = self
                .flowers
                .get(&line.sku)
                .ok_or_else(|| ShopError::UnknownFlower(line.sku.clone()))?;
            if f.stock < line.quantity {
                return Err(ShopError::OutOfStock {
                    sku: line.sku.clone(),
                    available: f.stock,
                    requested: line.quantity,
                });
            }
        }
        // Commit: decrement all stocks, create Order.
        for line in &lines {
            let f = self.flowers.get_mut(&line.sku).unwrap();
            f.stock -= line.quantity;
        }
        let id = self.next_order_id;
        self.next_order_id += 1;
        self.orders.insert(id, Order {
            id, owner_id,
            lines, discount_rules,
            shipping_cents, tax_rate_bps,
            status: OrderStatus::Placed,
        });
        self.carts.get_mut(cart_id).unwrap().status = OrderStatus::Placed;
        Ok(id)
    }

    pub fn cancel_order(&mut self, order_id: u64) -> Result<(), ShopError> {
        let lines = {
            let order = self
                .orders
                .get_mut(&order_id)
                .ok_or(ShopError::UnknownOrder(order_id))?;
            if order.status != OrderStatus::Placed {
                return Err(ShopError::BadTransition(
                    format!("order {order_id} is not placed")));
            }
            order.status = OrderStatus::Cancelled;
            order.lines.clone()
        };
        // Reverse the atomic decrement.
        for line in lines {
            if let Some(f) = self.flowers.get_mut(&line.sku) {
                f.stock += line.quantity;
            }
        }
        Ok(())
    }

    pub fn fulfill_order(&mut self, order_id: u64) -> Result<(), ShopError> {
        let order = self
            .orders
            .get_mut(&order_id)
            .ok_or(ShopError::UnknownOrder(order_id))?;
        if order.status != OrderStatus::Placed {
            return Err(ShopError::BadTransition(
                format!("order {order_id} is not placed")));
        }
        order.status = OrderStatus::Fulfilled;
        Ok(())
    }
}

impl Default for Shop {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() -> Shop {
        let mut s = Shop::new();
        s.add_flower(Flower { sku: "R".into(), name: "Rose".into(),
            unit_price_cents: 450, stock: 0 }).unwrap();
        s.add_flower(Flower { sku: "T".into(), name: "Tulip".into(),
            unit_price_cents: 300, stock: 0 }).unwrap();
        s.stock("R", 50).unwrap();
        s.stock("T", 30).unwrap();
        s.new_cart("C", "owner", 500, 825).unwrap();
        s
    }

    #[test]
    fn cart_freezes_price_at_add_time() {
        let mut s = setup();
        s.add_to_cart("C", "R", 2).unwrap();
        // Price changes after add — the cart line is unchanged.
        s.flowers.get_mut("R").unwrap().unit_price_cents = 9999;
        let cart = s.carts.get("C").unwrap();
        assert_eq!(cart.lines[0].unit_price_cents, 450);
    }

    #[test]
    fn add_to_cart_respects_stock() {
        let mut s = setup();
        s.stock("R", -50).unwrap_err();                  // can't set negative stock
        // Flower R has 50 in stock. Try to add 51.
        assert!(matches!(
            s.add_to_cart("C", "R", 51),
            Err(ShopError::OutOfStock { .. })
        ));
    }

    #[test]
    fn checkout_commits_atomically_and_decrements_stock() {
        let mut s = setup();
        s.add_to_cart("C", "R", 6).unwrap();
        s.add_to_cart("C", "T", 4).unwrap();
        let id = s.checkout("C").unwrap();
        assert_eq!(s.flowers["R"].stock, 50 - 6);
        assert_eq!(s.flowers["T"].stock, 30 - 4);
        assert_eq!(s.orders[&id].status, OrderStatus::Placed);
    }

    #[test]
    fn cancel_reverses_inventory_decrement() {
        let mut s = setup();
        s.add_to_cart("C", "R", 6).unwrap();
        let id = s.checkout("C").unwrap();
        assert_eq!(s.flowers["R"].stock, 44);
        s.cancel_order(id).unwrap();
        assert_eq!(s.flowers["R"].stock, 50);
        assert_eq!(s.orders[&id].status, OrderStatus::Cancelled);
    }

    #[test]
    fn discount_percent_applies_above_minimum() {
        let mut s = setup();
        s.add_to_cart("C", "R", 10).unwrap();        // 10 * 450 = 4500c
        s.apply_discount("C", DiscountRule {
            kind: DiscountKind::PercentageOff, value: 10,
            min_subtotal_cents: 4000, note: "".into(),
        }).unwrap();
        let cart = s.carts.get("C").unwrap();
        let total = Shop::cart_total_cents(cart);
        // subtotal 4500 − 10% (450) = 4050; tax @ 8.25% of 4050 = 334; ship 500; total ~4884
        assert_eq!(total, 4500 - 450 + (4050 * 825 / 10_000) + 500);
    }

    #[test]
    fn discount_skipped_below_minimum() {
        let mut s = setup();
        s.add_to_cart("C", "R", 2).unwrap();         // 900c subtotal, below min
        s.apply_discount("C", DiscountRule {
            kind: DiscountKind::PercentageOff, value: 10,
            min_subtotal_cents: 4000, note: "".into(),
        }).unwrap();
        let cart = s.carts.get("C").unwrap();
        let sub = Shop::subtotal_cents(&cart.lines);
        let disc = cart.discount_rules[0].discount_cents(sub, cart.shipping_cents);
        assert_eq!(disc, 0);
    }

    #[test]
    fn empty_cart_refuses_checkout() {
        let mut s = setup();
        assert!(matches!(s.checkout("C"), Err(ShopError::EmptyCart)));
    }
}
