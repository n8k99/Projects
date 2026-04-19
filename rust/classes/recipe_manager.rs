//! Recipe Creator and Manager — compositional entities, scaling, pantry checks, shopping lists.
//!
//! Quantities are (numerator, denominator) rationals so scaling by 1/3 or 0.5
//! stays exact. Units are matched strictly; cross-unit conversion is not this
//! project's concern.

use std::collections::{HashMap, HashSet};

/// Exact quantity as a reduced fraction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Qty {
    pub num: i64,
    pub den: i64,
}

impl Qty {
    pub const ZERO: Qty = Qty { num: 0, den: 1 };

    pub fn new(num: i64, den: i64) -> Qty {
        assert!(den != 0, "denominator cannot be zero");
        let (n, d) = if den < 0 { (-num, -den) } else { (num, den) };
        let g = gcd(n.unsigned_abs() as i64, d);
        Qty { num: n / g.max(1), den: d / g.max(1) }
    }

    pub fn whole(n: i64) -> Qty { Qty::new(n, 1) }
}

fn gcd(a: i64, b: i64) -> i64 {
    let (mut a, mut b) = (a.abs(), b.abs());
    while b != 0 { let t = b; b = a % b; a = t; }
    a.max(1)
}

impl std::ops::Add for Qty {
    type Output = Qty;
    fn add(self, rhs: Qty) -> Qty {
        Qty::new(self.num * rhs.den + rhs.num * self.den, self.den * rhs.den)
    }
}

impl std::ops::Sub for Qty {
    type Output = Qty;
    fn sub(self, rhs: Qty) -> Qty {
        Qty::new(self.num * rhs.den - rhs.num * self.den, self.den * rhs.den)
    }
}

impl std::ops::Mul for Qty {
    type Output = Qty;
    fn mul(self, rhs: Qty) -> Qty {
        Qty::new(self.num * rhs.num, self.den * rhs.den)
    }
}

impl PartialOrd for Qty {
    fn partial_cmp(&self, other: &Qty) -> Option<std::cmp::Ordering> {
        Some((self.num * other.den).cmp(&(other.num * self.den)))
    }
}

#[derive(Debug, Clone)]
pub struct Ingredient {
    pub id: String,
    pub name: String,
    pub default_unit: String,
}

#[derive(Debug, Clone)]
pub struct RecipeLine {
    pub ingredient_id: String,
    pub quantity: Qty,
    pub unit: String,
}

impl RecipeLine {
    pub fn scaled(&self, factor: Qty) -> RecipeLine {
        RecipeLine {
            ingredient_id: self.ingredient_id.clone(),
            quantity: self.quantity * factor,
            unit: self.unit.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Recipe {
    pub id: String,
    pub name: String,
    pub lines: Vec<RecipeLine>,
    pub serves: i64,
    pub steps: Vec<String>,
}

impl Recipe {
    pub fn scaled(&self, factor: Qty) -> Recipe {
        Recipe {
            id: self.id.clone(),
            name: self.name.clone(),
            lines: self.lines.iter().map(|l| l.scaled(factor)).collect(),
            serves: self.serves,
            steps: self.steps.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct PantryItem {
    pub quantity: Qty,
    pub unit: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShortageReason {
    Insufficient,
    UnitMismatch,
    NotStocked,
}

#[derive(Debug, Clone)]
pub struct Shortage {
    pub ingredient_id: String,
    pub required: Qty,
    pub available: Qty,
    pub unit: String,
    pub reason: ShortageReason,
}

#[derive(Debug)]
pub enum RecipeError {
    Duplicate(String),
    UnknownIngredient(String),
    UnknownRecipe(String),
    UnitMismatch { ingredient_id: String, pantry_unit: String, other_unit: String },
    InsufficientStock(Vec<Shortage>),
}

pub struct RecipeBook {
    pub ingredients: HashMap<String, Ingredient>,
    pub recipes: HashMap<String, Recipe>,
    pub pantry: HashMap<String, PantryItem>,
}

impl RecipeBook {
    pub fn new() -> Self {
        Self {
            ingredients: HashMap::new(),
            recipes: HashMap::new(),
            pantry: HashMap::new(),
        }
    }

    pub fn add_ingredient(&mut self, i: Ingredient) -> Result<(), RecipeError> {
        if self.ingredients.contains_key(&i.id) {
            return Err(RecipeError::Duplicate(i.id));
        }
        self.ingredients.insert(i.id.clone(), i);
        Ok(())
    }

    pub fn add_recipe(&mut self, r: Recipe) -> Result<(), RecipeError> {
        if self.recipes.contains_key(&r.id) {
            return Err(RecipeError::Duplicate(r.id));
        }
        for l in &r.lines {
            if !self.ingredients.contains_key(&l.ingredient_id) {
                return Err(RecipeError::UnknownIngredient(l.ingredient_id.clone()));
            }
        }
        self.recipes.insert(r.id.clone(), r);
        Ok(())
    }

    pub fn stock(&mut self, ingredient_id: &str, quantity: Qty, unit: &str) -> Result<(), RecipeError> {
        if !self.ingredients.contains_key(ingredient_id) {
            return Err(RecipeError::UnknownIngredient(ingredient_id.to_string()));
        }
        self.pantry.insert(
            ingredient_id.to_string(),
            PantryItem { quantity, unit: unit.to_string() },
        );
        Ok(())
    }

    pub fn add_stock(&mut self, ingredient_id: &str, quantity: Qty, unit: &str) -> Result<(), RecipeError> {
        if !self.ingredients.contains_key(ingredient_id) {
            return Err(RecipeError::UnknownIngredient(ingredient_id.to_string()));
        }
        match self.pantry.get_mut(ingredient_id) {
            None => {
                self.pantry.insert(
                    ingredient_id.to_string(),
                    PantryItem { quantity, unit: unit.to_string() },
                );
            }
            Some(p) => {
                if p.unit != unit {
                    return Err(RecipeError::UnitMismatch {
                        ingredient_id: ingredient_id.to_string(),
                        pantry_unit: p.unit.clone(),
                        other_unit: unit.to_string(),
                    });
                }
                p.quantity = p.quantity + quantity;
            }
        }
        Ok(())
    }

    pub fn scale(&self, recipe_id: &str, factor: Qty) -> Result<Recipe, RecipeError> {
        let r = self
            .recipes
            .get(recipe_id)
            .ok_or_else(|| RecipeError::UnknownRecipe(recipe_id.to_string()))?;
        Ok(r.scaled(factor))
    }

    fn required_lines(&self, recipe_id: &str, servings: Option<i64>) -> Result<Vec<RecipeLine>, RecipeError> {
        let r = self
            .recipes
            .get(recipe_id)
            .ok_or_else(|| RecipeError::UnknownRecipe(recipe_id.to_string()))?;
        match servings {
            None => Ok(r.lines.clone()),
            Some(s) if s == r.serves => Ok(r.lines.clone()),
            Some(s) => {
                let factor = Qty::new(s, r.serves);
                Ok(r.lines.iter().map(|l| l.scaled(factor)).collect())
            }
        }
    }

    pub fn can_make(&self, recipe_id: &str, servings: Option<i64>) -> Result<(bool, Vec<Shortage>), RecipeError> {
        let lines = self.required_lines(recipe_id, servings)?;
        let mut shortages = Vec::new();
        for line in lines {
            match self.pantry.get(&line.ingredient_id) {
                None => shortages.push(Shortage {
                    ingredient_id: line.ingredient_id,
                    required: line.quantity,
                    available: Qty::ZERO,
                    unit: line.unit,
                    reason: ShortageReason::NotStocked,
                }),
                Some(p) if p.unit != line.unit => shortages.push(Shortage {
                    ingredient_id: line.ingredient_id,
                    required: line.quantity,
                    available: Qty::ZERO,
                    unit: line.unit,
                    reason: ShortageReason::UnitMismatch,
                }),
                Some(p) if p.quantity < line.quantity => shortages.push(Shortage {
                    ingredient_id: line.ingredient_id,
                    required: line.quantity,
                    available: p.quantity,
                    unit: line.unit,
                    reason: ShortageReason::Insufficient,
                }),
                _ => {}
            }
        }
        Ok((shortages.is_empty(), shortages))
    }

    pub fn shopping_list(&self, recipe_id: &str, servings: Option<i64>) -> Result<Vec<RecipeLine>, RecipeError> {
        let lines = self.required_lines(recipe_id, servings)?;
        let mut out = Vec::new();
        for line in lines {
            match self.pantry.get(&line.ingredient_id) {
                None => out.push(line),
                Some(p) if p.unit != line.unit => out.push(line),
                Some(p) => {
                    let deficit = line.quantity - p.quantity;
                    if deficit > Qty::ZERO {
                        out.push(RecipeLine {
                            ingredient_id: line.ingredient_id,
                            quantity: deficit,
                            unit: line.unit,
                        });
                    }
                }
            }
        }
        Ok(out)
    }

    pub fn make(&mut self, recipe_id: &str, servings: Option<i64>) -> Result<(), RecipeError> {
        let (ok, shortages) = self.can_make(recipe_id, servings)?;
        if !ok {
            return Err(RecipeError::InsufficientStock(shortages));
        }
        let lines = self.required_lines(recipe_id, servings)?;
        for line in lines {
            let p = self.pantry.get_mut(&line.ingredient_id).unwrap();
            p.quantity = p.quantity - line.quantity;
        }
        Ok(())
    }

    pub fn recipes_using(&self, ingredient_id: &str) -> Vec<&Recipe> {
        self.recipes
            .values()
            .filter(|r| r.lines.iter().any(|l| l.ingredient_id == ingredient_id))
            .collect()
    }

    pub fn ingredients_used(&self) -> HashSet<String> {
        let mut out = HashSet::new();
        for r in self.recipes.values() {
            for l in &r.lines {
                out.insert(l.ingredient_id.clone());
            }
        }
        out
    }
}

impl Default for RecipeBook {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ing(id: &str, unit: &str) -> Ingredient {
        Ingredient { id: id.into(), name: id.into(), default_unit: unit.into() }
    }

    fn line(id: &str, q: i64, unit: &str) -> RecipeLine {
        RecipeLine { ingredient_id: id.into(), quantity: Qty::whole(q), unit: unit.into() }
    }

    fn setup() -> RecipeBook {
        let mut b = RecipeBook::new();
        b.add_ingredient(ing("flour", "g")).unwrap();
        b.add_ingredient(ing("sugar", "g")).unwrap();
        b.add_recipe(Recipe {
            id: "cake".into(), name: "Cake".into(), serves: 8,
            lines: vec![line("flour", 300, "g"), line("sugar", 200, "g")],
            steps: vec![],
        }).unwrap();
        b
    }

    #[test]
    fn scaling_preserves_shape_scales_quantities() {
        let b = setup();
        let half = b.scale("cake", Qty::new(1, 2)).unwrap();
        assert_eq!(half.lines[0].quantity, Qty::whole(150));
        assert_eq!(half.lines[1].quantity, Qty::whole(100));
    }

    #[test]
    fn can_make_conjunction_over_all_lines() {
        let mut b = setup();
        b.stock("flour", Qty::whole(500), "g").unwrap();
        b.stock("sugar", Qty::whole(100), "g").unwrap(); // short
        let (ok, shortages) = b.can_make("cake", None).unwrap();
        assert!(!ok);
        assert_eq!(shortages.len(), 1);
        assert_eq!(shortages[0].ingredient_id, "sugar");
    }

    #[test]
    fn shopping_list_is_the_delta() {
        let mut b = setup();
        b.stock("flour", Qty::whole(100), "g").unwrap();
        // sugar not stocked.
        let list = b.shopping_list("cake", None).unwrap();
        assert_eq!(list.len(), 2);
        let flour = list.iter().find(|l| l.ingredient_id == "flour").unwrap();
        assert_eq!(flour.quantity, Qty::whole(200)); // 300 - 100
        let sugar = list.iter().find(|l| l.ingredient_id == "sugar").unwrap();
        assert_eq!(sugar.quantity, Qty::whole(200)); // nothing on hand
    }

    #[test]
    fn make_consumes_atomically_or_not_at_all() {
        let mut b = setup();
        b.stock("flour", Qty::whole(500), "g").unwrap();
        b.stock("sugar", Qty::whole(100), "g").unwrap();
        // First attempt fails — insufficient sugar — and pantry is untouched.
        assert!(matches!(b.make("cake", None), Err(RecipeError::InsufficientStock(_))));
        assert_eq!(b.pantry["flour"].quantity, Qty::whole(500));
        assert_eq!(b.pantry["sugar"].quantity, Qty::whole(100));
        // Top up, then retry succeeds.
        b.add_stock("sugar", Qty::whole(200), "g").unwrap();
        b.make("cake", None).unwrap();
        assert_eq!(b.pantry["flour"].quantity, Qty::whole(200));
        assert_eq!(b.pantry["sugar"].quantity, Qty::whole(100));
    }

    #[test]
    fn scaling_by_servings_respected_in_can_make() {
        let mut b = setup();
        b.stock("flour", Qty::whole(100), "g").unwrap();
        b.stock("sugar", Qty::whole(100), "g").unwrap();
        // 8-serve needs 300g flour; 2-serve needs 75g.
        let (ok_full, _) = b.can_make("cake", None).unwrap();
        let (ok_small, _) = b.can_make("cake", Some(2)).unwrap();
        assert!(!ok_full);
        assert!(ok_small);
    }
}
