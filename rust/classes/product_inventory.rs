//! Product Inventory — entities with identity, movements as a ledger, current quantity as aggregate.

use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Product {
    pub sku: String,
    pub name: String,
    pub price: f64,
    pub category: String,
    pub reorder_point: i32,
}

impl Product {
    pub fn new(sku: &str, name: &str, price: f64) -> Self {
        Self {
            sku: sku.to_string(),
            name: name.to_string(),
            price,
            category: String::new(),
            reorder_point: 0,
        }
    }

    pub fn with_category(mut self, category: &str) -> Self {
        self.category = category.to_string();
        self
    }

    pub fn with_reorder_point(mut self, point: i32) -> Self {
        self.reorder_point = point;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MovementKind {
    Receive,
    Sell,
}

#[derive(Debug, Clone)]
pub struct Movement {
    pub sku: String,
    pub kind: MovementKind,
    pub quantity: i32,
    pub note: String,
}

impl Movement {
    pub fn signed(&self) -> i32 {
        match self.kind {
            MovementKind::Receive => self.quantity,
            MovementKind::Sell => -self.quantity,
        }
    }
}

#[derive(Debug)]
pub enum InventoryError {
    UnknownSku(String),
    InsufficientStock { sku: String, on_hand: i32, requested: i32 },
    NonPositiveQuantity,
    DuplicateSku(String),
}

pub struct Inventory {
    pub products: HashMap<String, Product>,
    pub movements: Vec<Movement>,
}

impl Inventory {
    pub fn new() -> Self {
        Self { products: HashMap::new(), movements: Vec::new() }
    }

    pub fn add_product(&mut self, product: Product) -> Result<(), InventoryError> {
        if self.products.contains_key(&product.sku) {
            return Err(InventoryError::DuplicateSku(product.sku));
        }
        self.products.insert(product.sku.clone(), product);
        Ok(())
    }

    pub fn get(&self, sku: &str) -> Result<&Product, InventoryError> {
        self.products
            .get(sku)
            .ok_or_else(|| InventoryError::UnknownSku(sku.to_string()))
    }

    pub fn receive(&mut self, sku: &str, quantity: i32, note: &str) -> Result<(), InventoryError> {
        self.get(sku)?;
        if quantity <= 0 {
            return Err(InventoryError::NonPositiveQuantity);
        }
        self.movements.push(Movement {
            sku: sku.to_string(),
            kind: MovementKind::Receive,
            quantity,
            note: note.to_string(),
        });
        Ok(())
    }

    pub fn sell(&mut self, sku: &str, quantity: i32, note: &str) -> Result<(), InventoryError> {
        self.get(sku)?;
        if quantity <= 0 {
            return Err(InventoryError::NonPositiveQuantity);
        }
        let on_hand = self.quantity(sku)?;
        if on_hand < quantity {
            return Err(InventoryError::InsufficientStock {
                sku: sku.to_string(),
                on_hand,
                requested: quantity,
            });
        }
        self.movements.push(Movement {
            sku: sku.to_string(),
            kind: MovementKind::Sell,
            quantity,
            note: note.to_string(),
        });
        Ok(())
    }

    pub fn quantity(&self, sku: &str) -> Result<i32, InventoryError> {
        self.get(sku)?;
        Ok(self.movements.iter().filter(|m| m.sku == sku).map(|m| m.signed()).sum())
    }

    pub fn history(&self, sku: &str) -> Result<Vec<&Movement>, InventoryError> {
        self.get(sku)?;
        Ok(self.movements.iter().filter(|m| m.sku == sku).collect())
    }

    pub fn total_value(&self) -> f64 {
        self.products
            .values()
            .map(|p| p.price * self.quantity(&p.sku).unwrap_or(0) as f64)
            .sum()
    }

    pub fn restock_needed(&self) -> Vec<&Product> {
        self.products
            .values()
            .filter(|p| p.reorder_point > 0 && self.quantity(&p.sku).unwrap_or(0) <= p.reorder_point)
            .collect()
    }

    pub fn by_category(&self, category: &str) -> Vec<&Product> {
        self.products.values().filter(|p| p.category == category).collect()
    }
}

impl Default for Inventory {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quantity_aggregates_movements() {
        let mut inv = Inventory::new();
        inv.add_product(Product::new("A", "Widget", 1.0).with_reorder_point(5)).unwrap();
        inv.receive("A", 10, "in").unwrap();
        inv.sell("A", 3, "out").unwrap();
        assert_eq!(inv.quantity("A").unwrap(), 7);
    }

    #[test]
    fn sell_refuses_overdraw() {
        let mut inv = Inventory::new();
        inv.add_product(Product::new("A", "Widget", 1.0)).unwrap();
        inv.receive("A", 2, "").unwrap();
        assert!(matches!(inv.sell("A", 3, ""), Err(InventoryError::InsufficientStock { .. })));
    }

    #[test]
    fn restock_lists_products_at_or_below_reorder_point() {
        let mut inv = Inventory::new();
        inv.add_product(Product::new("A", "Widget", 1.0).with_reorder_point(5)).unwrap();
        inv.add_product(Product::new("B", "Gadget", 1.0).with_reorder_point(5)).unwrap();
        inv.receive("A", 4, "").unwrap();
        inv.receive("B", 10, "").unwrap();
        let needed: Vec<&str> = inv.restock_needed().iter().map(|p| p.sku.as_str()).collect();
        assert_eq!(needed, vec!["A"]);
    }
}
