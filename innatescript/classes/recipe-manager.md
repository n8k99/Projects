# G055 — Recipe Creator and Manager

> The recipe is the first entity in the Rosetta Stone with a *structured field* — an ordered list of line items — rather than a collection of atomic scalars. Operations over recipes are therefore operations over that structure.

```yaml
id: G055
title: Recipe Creator and Manager
category: classes
requires: [G048-product-inventory, G052-bank-account]
provides: [compositional-entity, structured-scaling, n-ary-conjunction, gap-report, atomic-multi-consumption]
```

## Insight: The Entity Is Its Inner Structure

Every prior entity in the Rosetta Stone had scalar fields. Products had prices, accounts had balances, appointments had start/end. G055's recipe has a **list of line items**, each itself a small structure (ingredient + quantity + unit). The recipe's identity is its name; its *substance* is the ordered multiset inside.

Operations on scalar-field entities are trivially typed — add a price, set a status. Operations on structure-field entities are *maps and folds over the inner collection*. Scale a recipe: map every line. Check can_make: fold the lines against the pantry. Build a shopping list: filter-map the lines. The operation is not a single write — it is a traversal.

This is how every rich vault entity works. A project has a list of goals (each a small structure). An agent has a capability set (each capability has a proficiency score). A conversation has a message stream (each message has author + content + timestamp). The `Projects` index, the `Agents` index, the `Conversations` index — each is a container of compositional entities. G055 is the first Rosetta Stone project to model one.

## Insight: Scaling Is a Pure Linear Map Over the Structure

`scale(recipe, 0.5)` multiplies every line's quantity by 0.5 and returns a new recipe with the same shape. Pure function, no mutation, linear over the structure:

```
scale(r, f) = { ...r, lines: r.lines.map(l => { ...l, quantity: l.quantity * f }) }
```

This is the cleanest possible operation over a structured entity. In category theory terms, scaling is a morphism in the category of recipes-at-different-quantities. All recipes at all scales have the same line identities; only quantities change. The noosphere will have many such morphisms over compositional entities: "compress this project's timeline by 30%", "halve the agent's workload", "double this conversation's priority weights." G055 introduces the pattern; the rest of the Classes category will use it.

Exact arithmetic matters. Scaling by 1/3 on integer grams should give 100g → 33⅓g, not 33.333333g. The Rosetta Stone implementations use rationals (Fraction, Qty with num/den, Common Lisp's native rationals, Lean's own Qty struct) everywhere a quantity lives. This is the first project where floats would be wrong — and it's the same reason G052 used integer cents. When the math is about real-world amounts, the type must be exact.

## Insight: `can_make` Is an N-Ary Conjunction

G054 introduced binary conjunctive availability: doctor free AND room free. G055 generalizes to **N-ary**: `can_make(recipe)` is a conjunction over *every line*, and the width is dynamic — it depends on the recipe. Some recipes have 3 ingredients, some have 30. The conjunction adapts.

```
can_make(r)  ⇔  ∀ line ∈ r.lines: pantry_covers(line)
```

This is the shape of every "requirements met" check in the noosphere. All dependencies installed? Conjunction over a dynamic dependency list. All tests pass? Conjunction over a dynamic test list. All agents ready? Conjunction over the agent roster. Width-dynamic conjunction is how reality works; G055 is the first project where it appears cleanly.

When the conjunction fails, the report is **structured** — which lines failed, and why. Reasons are enumerated: `not_stocked`, `unit_mismatch`, `insufficient`. The failure mode isn't a single boolean or a single error string; it's a list of per-line diagnoses. Good `where` clauses in InnateScript should always return this shape when they fail. A single "where failed" with no detail is almost useless; a "where failed because A, B, and C" is actionable.

## Insight: Shopping List Is a Computed Delta, Not a Stored Record

```
shopping_list = required - available    (per line, clipped at zero)
```

The shopping list doesn't exist as a record anywhere. It is derived on demand from the gap between desire and reality. This is the first Rosetta Stone project where the main output of a query is a **gap report** — the difference between what's wanted and what's available.

Gap-based reporting is its own primitive. Every noosphere planning operation produces such a report: a project's roadmap gap is "required phases minus completed phases"; a context gap is "what the choreography needs to know minus what it has access to"; a capability gap is "what the task demands minus what the agent has." These are all computed deltas over structured fields. Recipes make the pattern concrete with the clearest possible example.

## Insight: `make` Is Atomic Multi-Entity Consumption

G052 introduced atomic multi-entity updates (double-entry transactions). G055 uses the pattern for a different purpose: **consumption** rather than transfer. `make(recipe)` subtracts quantities from N pantry items. Either all subtractions land or none do. The same all-or-nothing guarantee, different semantics.

The distinction from G052 matters:

- G052's transfer preserved total sum (conservation). What left one account arrived at another.
- G055's make does NOT preserve — ingredients are destroyed. The pantry's total diminishes.

Two faces of multi-entity atomic updates: conservation (money, labels) and consumption (materials, attention, compute). Both need atomicity; they differ in whether the total is invariant. InnateScript's choreography engine will need to distinguish these when scoring `where` clauses: "did the transfer balance?" is a conservation check; "did we have enough?" is a consumption check. Different arithmetic, same atomicity requirement.

## Choreographic Case: Meal Plan for the Week

```innate
(@weekly-meal-plan){
  @pantry  <- @recipe-book/pantry
  @planned <- [@breakfast-mon, @lunch-tue, @dinner-thu, ...]

  @cumulative-need <- @planned.fold(
    start: {},
    step: (acc, recipe) => merge_adding(acc, recipe.lines)
  )

  @shopping <- @recipe-book/gap{required: @cumulative-need, available: @pantry}

  concurrent {
    @sarah/order-groceries{list: @shopping}
    @korrallan/check-schedule{recipes: @planned}
  } join as @plan

  where {
    all_recipes_known:    @planned.every(.id ∈ @recipe-book/recipes)
    no_unit_conflicts:    @shopping.none(.reason == unit_mismatch)
    grocery_confirmed:    @plan.sarah == "ordered"
  }
}
```

Merge the week's recipes into a cumulative requirement. Subtract the pantry. What's left is the shopping list. One compositional operation (merge_adding) over the structured fields, followed by the same gap pattern G055 introduces at the single-recipe level.

## Structures

```innate
(defstruct ingredient
  id           : String
  name         : String
  default-unit : String)

(defstruct recipe-line
  ingredient-id : String
  quantity      : Qty                    ;; exact rational
  unit          : String)

(defstruct recipe
  id     : String
  name   : String
  lines  : [RecipeLine]
  serves : Nat
  steps  : [String])

(defstruct pantry-item
  quantity : Qty
  unit     : String)

(defstruct shortage
  ingredient-id : String
  required      : Qty
  available     : Qty
  unit          : String
  reason        : "insufficient" | "unit-mismatch" | "not-stocked")
```

## Resolver Natives

```innate
@recipe-book{}                                                -> RecipeBook
@recipe-book/add-ingredient{ingredient}                       -> RecipeBook
@recipe-book/add-recipe{recipe}                               -> RecipeBook
@recipe-book/stock{ingredient-id, quantity, unit}             -> RecipeBook
@recipe-book/scale{recipe-id, factor}                         -> Recipe
@recipe-book/can-make{recipe-id, servings?}                   -> (Bool, [Shortage])
@recipe-book/shopping-list{recipe-id, servings?}              -> [RecipeLine]
@recipe-book/make{recipe-id, servings?}                       -> RecipeBook
@recipe-book/recipes-using{ingredient-id}                     -> [Recipe]
```

## Demo

```innate
(@demo){
  @book <- @recipe-book{}
    .add-ingredient{id: "flour", default-unit: "g"}
    .add-ingredient{id: "sugar", default-unit: "g"}
    .add-recipe{
      id: "cookies", name: "Sugar Cookies", serves: 24,
      lines: [
        {ingredient-id: "flour", quantity: 300, unit: "g"},
        {ingredient-id: "sugar", quantity: 200, unit: "g"}
      ]
    }
    .stock{ingredient-id: "flour", quantity: 500, unit: "g"}
    .stock{ingredient-id: "sugar", quantity: 150, unit: "g"}

  @half <- @book/scale{recipe-id: "cookies", factor: 1/2}
  ;; @half.lines == [{flour, 150, g}, {sugar, 100, g}]

  @status <- @book/can-make{recipe-id: "cookies", servings: 12}
  ;; ok (half scale fits the pantry)

  @missing <- @book/shopping-list{recipe-id: "cookies"}
  ;; [{sugar, 50, g}]  -- only the gap
}
```

## Where

Scaling MUST be a pointwise linear map over `lines`. `can_make` MUST be the conjunction over every line (not the average, not the sum). `shopping_list` MUST be the delta, not the requirement. `make` MUST be atomic — either consume every line's amount or change nothing. Those four rules distinguish a recipe manager from a checklist that mentions food.
