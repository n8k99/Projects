// Recipe Creator and Manager — compositional entities, scaling, pantry checks, shopping lists.
package classes

import (
	"errors"
	"fmt"
)

// Qty is an exact rational quantity. Keep both scaling by 1/3 and integer
// ingredients like "2 eggs" representable without floating point drift.
type Qty struct {
	Num int64
	Den int64
}

// NewQty builds a reduced rational.
func NewQty(num, den int64) Qty {
	if den == 0 {
		panic("denominator cannot be zero")
	}
	if den < 0 {
		num, den = -num, -den
	}
	g := qtyGcd(absInt(num), den)
	if g == 0 {
		g = 1
	}
	return Qty{num / g, den / g}
}

// QtyWhole builds Qty{n, 1}.
func QtyWhole(n int64) Qty { return NewQty(n, 1) }

func qtyGcd(a, b int64) int64 {
	for b != 0 {
		a, b = b, a%b
	}
	if a < 1 {
		return 1
	}
	return a
}

func absInt(x int64) int64 {
	if x < 0 {
		return -x
	}
	return x
}

// Add returns q + r.
func (q Qty) Add(r Qty) Qty { return NewQty(q.Num*r.Den+r.Num*q.Den, q.Den*r.Den) }

// Sub returns q - r.
func (q Qty) Sub(r Qty) Qty { return NewQty(q.Num*r.Den-r.Num*q.Den, q.Den*r.Den) }

// Mul returns q * r.
func (q Qty) Mul(r Qty) Qty { return NewQty(q.Num*r.Num, q.Den*r.Den) }

// Less reports q < r via cross-multiplication.
func (q Qty) Less(r Qty) bool { return q.Num*r.Den < r.Num*q.Den }

// IsZero reports whether q equals zero.
func (q Qty) IsZero() bool { return q.Num == 0 }

// Ingredient is a stockable good.
type Ingredient struct {
	ID          string
	Name        string
	DefaultUnit string
}

// RecipeLine is one ingredient requirement.
type RecipeLine struct {
	IngredientID string
	Quantity     Qty
	Unit         string
}

// Scaled returns a copy with quantity multiplied by factor.
func (l RecipeLine) Scaled(factor Qty) RecipeLine {
	return RecipeLine{IngredientID: l.IngredientID, Quantity: l.Quantity.Mul(factor), Unit: l.Unit}
}

// Recipe is a structured compound entity.
type Recipe struct {
	ID     string
	Name   string
	Lines  []RecipeLine
	Serves int64
	Steps  []string
}

// Scaled returns a pointwise-scaled recipe.
func (r Recipe) Scaled(factor Qty) Recipe {
	lines := make([]RecipeLine, len(r.Lines))
	for i, l := range r.Lines {
		lines[i] = l.Scaled(factor)
	}
	steps := append([]string{}, r.Steps...)
	return Recipe{ID: r.ID, Name: r.Name, Lines: lines, Serves: r.Serves, Steps: steps}
}

// PantryItem is a stocked amount in a specific unit.
type PantryItem struct {
	Quantity Qty
	Unit     string
}

// ShortageReason categorizes why a line can't be satisfied.
type ShortageReason string

const (
	ShortageInsufficient ShortageReason = "insufficient"
	ShortageUnitMismatch ShortageReason = "unit_mismatch"
	ShortageNotStocked   ShortageReason = "not_stocked"
)

// Shortage reports one line's gap between required and available.
type Shortage struct {
	IngredientID string
	Required     Qty
	Available    Qty
	Unit         string
	Reason       ShortageReason
}

// Recipe errors.
var (
	ErrUnknownIngredient = errors.New("unknown ingredient")
	ErrUnknownRecipe     = errors.New("unknown recipe")
	ErrUnitMismatch      = errors.New("unit mismatch")
	ErrInsufficientStock = errors.New("insufficient stock")
)

// RecipeBook holds ingredients, recipes, and the pantry.
type RecipeBook struct {
	Ingredients map[string]Ingredient
	Recipes     map[string]Recipe
	Pantry      map[string]*PantryItem
}

// NewRecipeBook builds an empty book.
func NewRecipeBook() *RecipeBook {
	return &RecipeBook{
		Ingredients: make(map[string]Ingredient),
		Recipes:     make(map[string]Recipe),
		Pantry:      make(map[string]*PantryItem),
	}
}

// AddIngredient registers an ingredient.
func (b *RecipeBook) AddIngredient(i Ingredient) error {
	if _, ok := b.Ingredients[i.ID]; ok {
		return fmt.Errorf("%w: ingredient %s", ErrDuplicate, i.ID)
	}
	b.Ingredients[i.ID] = i
	return nil
}

// AddRecipe registers a recipe; every referenced ingredient must exist.
func (b *RecipeBook) AddRecipe(r Recipe) error {
	if _, ok := b.Recipes[r.ID]; ok {
		return fmt.Errorf("%w: recipe %s", ErrDuplicate, r.ID)
	}
	for _, l := range r.Lines {
		if _, ok := b.Ingredients[l.IngredientID]; !ok {
			return fmt.Errorf("%w: %s", ErrUnknownIngredient, l.IngredientID)
		}
	}
	b.Recipes[r.ID] = r
	return nil
}

// Stock sets (not adds) the pantry amount for an ingredient.
func (b *RecipeBook) Stock(ingredientID string, quantity Qty, unit string) error {
	if _, ok := b.Ingredients[ingredientID]; !ok {
		return fmt.Errorf("%w: %s", ErrUnknownIngredient, ingredientID)
	}
	b.Pantry[ingredientID] = &PantryItem{Quantity: quantity, Unit: unit}
	return nil
}

// AddStock increases pantry stock; fails on unit mismatch.
func (b *RecipeBook) AddStock(ingredientID string, quantity Qty, unit string) error {
	if _, ok := b.Ingredients[ingredientID]; !ok {
		return fmt.Errorf("%w: %s", ErrUnknownIngredient, ingredientID)
	}
	if p, ok := b.Pantry[ingredientID]; ok {
		if p.Unit != unit {
			return fmt.Errorf("%w: %s pantry in %s, adding %s", ErrUnitMismatch, ingredientID, p.Unit, unit)
		}
		p.Quantity = p.Quantity.Add(quantity)
		return nil
	}
	b.Pantry[ingredientID] = &PantryItem{Quantity: quantity, Unit: unit}
	return nil
}

// Scale returns a scaled copy of a recipe.
func (b *RecipeBook) Scale(recipeID string, factor Qty) (Recipe, error) {
	r, ok := b.Recipes[recipeID]
	if !ok {
		return Recipe{}, fmt.Errorf("%w: %s", ErrUnknownRecipe, recipeID)
	}
	return r.Scaled(factor), nil
}

// requiredLines returns the scaled line set for the requested servings.
// Pass servings=0 to use the recipe's native serves value.
func (b *RecipeBook) requiredLines(recipeID string, servings int64) ([]RecipeLine, error) {
	r, ok := b.Recipes[recipeID]
	if !ok {
		return nil, fmt.Errorf("%w: %s", ErrUnknownRecipe, recipeID)
	}
	if servings == 0 || servings == r.Serves {
		return append([]RecipeLine{}, r.Lines...), nil
	}
	factor := NewQty(servings, r.Serves)
	out := make([]RecipeLine, len(r.Lines))
	for i, l := range r.Lines {
		out[i] = l.Scaled(factor)
	}
	return out, nil
}

// CanMake reports (ok, shortages). Pass servings=0 for the native recipe size.
func (b *RecipeBook) CanMake(recipeID string, servings int64) (bool, []Shortage, error) {
	lines, err := b.requiredLines(recipeID, servings)
	if err != nil {
		return false, nil, err
	}
	var shortages []Shortage
	for _, l := range lines {
		p, ok := b.Pantry[l.IngredientID]
		switch {
		case !ok:
			shortages = append(shortages, Shortage{
				IngredientID: l.IngredientID, Required: l.Quantity, Available: QtyWhole(0),
				Unit: l.Unit, Reason: ShortageNotStocked,
			})
		case p.Unit != l.Unit:
			shortages = append(shortages, Shortage{
				IngredientID: l.IngredientID, Required: l.Quantity, Available: QtyWhole(0),
				Unit: l.Unit, Reason: ShortageUnitMismatch,
			})
		case p.Quantity.Less(l.Quantity):
			shortages = append(shortages, Shortage{
				IngredientID: l.IngredientID, Required: l.Quantity, Available: p.Quantity,
				Unit: l.Unit, Reason: ShortageInsufficient,
			})
		}
	}
	return len(shortages) == 0, shortages, nil
}

// ShoppingList returns the gap-report: required minus available, per line.
// Only lines needing more appear.
func (b *RecipeBook) ShoppingList(recipeID string, servings int64) ([]RecipeLine, error) {
	lines, err := b.requiredLines(recipeID, servings)
	if err != nil {
		return nil, err
	}
	var out []RecipeLine
	for _, l := range lines {
		p, ok := b.Pantry[l.IngredientID]
		if !ok || p.Unit != l.Unit {
			out = append(out, l)
			continue
		}
		deficit := l.Quantity.Sub(p.Quantity)
		if !deficit.IsZero() && !deficit.Less(QtyWhole(0)) {
			out = append(out, RecipeLine{IngredientID: l.IngredientID, Quantity: deficit, Unit: l.Unit})
		}
	}
	return out, nil
}

// Make atomically consumes the ingredients required by the scaled recipe.
// All lines must be satisfiable; no partial consumption occurs on failure.
func (b *RecipeBook) Make(recipeID string, servings int64) error {
	ok, shortages, err := b.CanMake(recipeID, servings)
	if err != nil {
		return err
	}
	if !ok {
		return fmt.Errorf("%w: %d shortage(s) on %s", ErrInsufficientStock, len(shortages), recipeID)
	}
	lines, _ := b.requiredLines(recipeID, servings)
	for _, l := range lines {
		b.Pantry[l.IngredientID].Quantity = b.Pantry[l.IngredientID].Quantity.Sub(l.Quantity)
	}
	return nil
}

// RecipesUsing returns every recipe referencing an ingredient.
func (b *RecipeBook) RecipesUsing(ingredientID string) []Recipe {
	var out []Recipe
	for _, r := range b.Recipes {
		for _, l := range r.Lines {
			if l.IngredientID == ingredientID {
				out = append(out, r)
				break
			}
		}
	}
	return out
}

// IngredientsUsed returns the set of ingredients referenced by any recipe.
func (b *RecipeBook) IngredientsUsed() map[string]struct{} {
	out := make(map[string]struct{})
	for _, r := range b.Recipes {
		for _, l := range r.Lines {
			out[l.IngredientID] = struct{}{}
		}
	}
	return out
}
