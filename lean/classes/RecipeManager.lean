-- Recipe Creator and Manager — compositional entities, scaling, pantry checks, shopping lists.
--
-- Quantities are represented as (num, den) pairs with reduction on construction,
-- keeping scaling exact without floating point drift.

namespace Classes

/-- Exact rational quantity. -/
structure Qty where
  num : Int
  den : Nat
  deriving Repr

namespace Qty
partial def gcdNat (a b : Nat) : Nat :=
  if b == 0 then if a == 0 then 1 else a else gcdNat b (a % b)

def reduce (n : Int) (d : Nat) : Qty :=
  if d == 0 then { num := n, den := 1 }
  else
    let g := gcdNat n.natAbs d
    { num := n / g, den := d / g }

def zero : Qty := { num := 0, den := 1 }
def whole (n : Int) : Qty := { num := n, den := 1 }

def add (a b : Qty) : Qty :=
  reduce (a.num * (b.den : Int) + b.num * (a.den : Int)) (a.den * b.den)

def sub (a b : Qty) : Qty :=
  reduce (a.num * (b.den : Int) - b.num * (a.den : Int)) (a.den * b.den)

def mul (a b : Qty) : Qty :=
  reduce (a.num * b.num) (a.den * b.den)

/-- Compare via cross-multiplication. -/
def lt (a b : Qty) : Bool :=
  a.num * (b.den : Int) < b.num * (a.den : Int)

def isZero (a : Qty) : Bool := a.num == 0

end Qty

structure Ingredient where
  id : String
  name : String
  defaultUnit : String := ""
  deriving Repr

structure RecipeLine where
  ingredientId : String
  quantity : Qty
  unit : String
  deriving Repr

def RecipeLine.scaled (l : RecipeLine) (factor : Qty) : RecipeLine :=
  { l with quantity := Qty.mul l.quantity factor }

structure Recipe where
  id : String
  name : String
  lines : List RecipeLine := []
  serves : Nat := 1
  steps : List String := []
  deriving Repr

def Recipe.scaled (r : Recipe) (factor : Qty) : Recipe :=
  { r with lines := r.lines.map (·.scaled factor) }

structure PantryItem where
  quantity : Qty
  unit : String
  deriving Repr

inductive ShortageReason
  | insufficient
  | unitMismatch
  | notStocked
  deriving Repr, DecidableEq

structure Shortage where
  ingredientId : String
  required : Qty
  available : Qty
  unit : String
  reason : ShortageReason
  deriving Repr

structure RecipeBook where
  ingredients : List Ingredient := []
  recipes : List Recipe := []
  pantry : List (String × PantryItem) := []
  deriving Repr

def RecipeBook.empty : RecipeBook := { }

def RecipeBook.findIngredient (b : RecipeBook) (id : String) : Option Ingredient :=
  b.ingredients.find? (·.id == id)

def RecipeBook.findRecipe (b : RecipeBook) (id : String) : Option Recipe :=
  b.recipes.find? (·.id == id)

def RecipeBook.pantryOf (b : RecipeBook) (id : String) : Option PantryItem :=
  (b.pantry.find? (fun p => p.fst == id)).map (·.snd)

def RecipeBook.addIngredient (b : RecipeBook) (i : Ingredient) : Except String RecipeBook :=
  if b.ingredients.any (·.id == i.id)
  then .error s!"duplicate ingredient: {i.id}"
  else .ok { b with ingredients := b.ingredients ++ [i] }

def RecipeBook.addRecipe (b : RecipeBook) (r : Recipe) : Except String RecipeBook :=
  if b.recipes.any (·.id == r.id) then
    .error s!"duplicate recipe: {r.id}"
  else if r.lines.any fun l => (b.findIngredient l.ingredientId).isNone then
    .error "unknown ingredient in recipe lines"
  else
    .ok { b with recipes := b.recipes ++ [r] }

def RecipeBook.stock (b : RecipeBook) (id : String) (q : Qty) (unit : String)
    : Except String RecipeBook :=
  if (b.findIngredient id).isNone then
    .error s!"unknown ingredient: {id}"
  else
    let filtered := b.pantry.filter fun p => p.fst ≠ id
    .ok { b with pantry := filtered ++ [(id, { quantity := q, unit })] }

def RecipeBook.scale (b : RecipeBook) (id : String) (factor : Qty) : Except String Recipe :=
  match b.findRecipe id with
  | none => .error s!"unknown recipe: {id}"
  | some r => .ok (r.scaled factor)

private def requiredLines (r : Recipe) (servings : Option Nat) : List RecipeLine :=
  match servings with
  | none => r.lines
  | some s =>
    if s == r.serves then r.lines
    else
      let factor := Qty.reduce (s : Int) r.serves
      r.lines.map (·.scaled factor)

/-- Conjunction over recipe lines: each must be satisfiable from the pantry. -/
def RecipeBook.canMake (b : RecipeBook) (recipeId : String) (servings : Option Nat := none)
    : Except String (Bool × List Shortage) :=
  match b.findRecipe recipeId with
  | none => .error s!"unknown recipe: {recipeId}"
  | some r =>
    let lines := requiredLines r servings
    let shortages := lines.filterMap fun l =>
      match b.pantryOf l.ingredientId with
      | none => some {
          ingredientId := l.ingredientId, required := l.quantity,
          available := Qty.zero, unit := l.unit, reason := .notStocked }
      | some p =>
        if p.unit ≠ l.unit then
          some { ingredientId := l.ingredientId, required := l.quantity,
                 available := Qty.zero, unit := l.unit, reason := .unitMismatch }
        else if Qty.lt p.quantity l.quantity then
          some { ingredientId := l.ingredientId, required := l.quantity,
                 available := p.quantity, unit := l.unit, reason := .insufficient }
        else none
    .ok (shortages.isEmpty, shortages)

/-- Gap report: required minus available, per line. -/
def RecipeBook.shoppingList (b : RecipeBook) (recipeId : String)
    (servings : Option Nat := none) : Except String (List RecipeLine) :=
  match b.findRecipe recipeId with
  | none => .error s!"unknown recipe: {recipeId}"
  | some r =>
    let lines := requiredLines r servings
    .ok <| lines.filterMap fun l =>
      match b.pantryOf l.ingredientId with
      | none => some l
      | some p =>
        if p.unit ≠ l.unit then some l
        else
          let deficit := Qty.sub l.quantity p.quantity
          if ¬ deficit.isZero && ¬ Qty.lt deficit Qty.zero
            then some { l with quantity := deficit }
            else none

/-- Atomic consumption: all or nothing. -/
def RecipeBook.makeIt (b : RecipeBook) (recipeId : String) (servings : Option Nat := none)
    : Except String RecipeBook := do
  let (ok, shortages) ← b.canMake recipeId servings
  if ¬ ok then throw s!"insufficient: {shortages.length} shortage(s)"
  let r ← (b.findRecipe recipeId).elim (throw s!"unknown recipe: {recipeId}") pure
  let lines := requiredLines r servings
  let newPantry := b.pantry.map fun (id, p) =>
    match lines.find? (·.ingredientId == id) with
    | some line => (id, { p with quantity := Qty.sub p.quantity line.quantity })
    | none => (id, p)
  pure { b with pantry := newPantry }

def RecipeBook.recipesUsing (b : RecipeBook) (ingredientId : String) : List Recipe :=
  b.recipes.filter fun r => r.lines.any (·.ingredientId == ingredientId)

def RecipeBook.ingredientsUsed (b : RecipeBook) : List String :=
  let all := b.recipes.flatMap fun r => r.lines.map (·.ingredientId)
  all.eraseDups

end Classes
