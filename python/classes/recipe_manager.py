"""Recipe Creator and Manager — compositional entities, scaling, pantry checks, shopping lists.

A recipe is the first entity in the Rosetta Stone whose primary field is a
structured collection (an ordered list of line items), not atomic scalars.
Operations over recipes are therefore operations over that structure:

  * scale — a pure linear map over every line's quantity.
  * can_make — conjunction over N ingredients (width determined by recipe).
  * shopping_list — the delta between required and available.
  * make — atomic multi-entity consumption of pantry stock.

Units are matched strictly: if the pantry has 500 g of flour and the recipe
calls for 2 cups, the system reports incompatible units rather than guessing
a conversion. Unit algebra is an adjacent problem, not this problem.
"""

from dataclasses import dataclass, field
from typing import Optional
from fractions import Fraction


# Represent quantities as Fractions so scaling by 0.5 or 1/3 stays exact.
Qty = Fraction


def qty(value) -> Qty:
    """Construct a quantity from int/float/str/Fraction."""
    return value if isinstance(value, Fraction) else Fraction(str(value))


@dataclass
class Ingredient:
    id: str
    name: str
    default_unit: str = ""


@dataclass
class RecipeLine:
    """One ingredient requirement within a recipe."""
    ingredient_id: str
    quantity: Qty
    unit: str

    def scaled(self, factor: Qty) -> "RecipeLine":
        return RecipeLine(self.ingredient_id, self.quantity * factor, self.unit)


@dataclass
class Recipe:
    id: str
    name: str
    lines: list[RecipeLine] = field(default_factory=list)
    serves: int = 1
    steps: list[str] = field(default_factory=list)

    def scaled(self, factor: Qty) -> "Recipe":
        """A new recipe with every line's quantity multiplied by factor."""
        return Recipe(
            id=self.id, name=self.name,
            lines=[l.scaled(factor) for l in self.lines],
            serves=self.serves,  # serves is original; caller interprets
            steps=list(self.steps),
        )


@dataclass
class PantryItem:
    quantity: Qty
    unit: str


@dataclass
class Shortage:
    """One line's worth of missing stock, as a computed delta."""
    ingredient_id: str
    required: Qty
    available: Qty
    unit: str
    reason: str = "insufficient"  # or "unit_mismatch" or "not_stocked"

    @property
    def missing(self) -> Qty:
        return self.required - self.available


class RecipeError(Exception):
    pass


class UnknownEntity(RecipeError): ...
class UnitMismatch(RecipeError): ...
class InsufficientStock(RecipeError): ...


class RecipeBook:
    """Catalogue of ingredients and recipes, with a mutable pantry."""

    def __init__(self) -> None:
        self.ingredients: dict[str, Ingredient] = {}
        self.recipes: dict[str, Recipe] = {}
        self.pantry: dict[str, PantryItem] = {}

    # --- registration ---
    def add_ingredient(self, ingredient: Ingredient) -> None:
        if ingredient.id in self.ingredients:
            raise ValueError(f"ingredient already registered: {ingredient.id}")
        self.ingredients[ingredient.id] = ingredient

    def add_recipe(self, recipe: Recipe) -> None:
        if recipe.id in self.recipes:
            raise ValueError(f"recipe already registered: {recipe.id}")
        for line in recipe.lines:
            if line.ingredient_id not in self.ingredients:
                raise UnknownEntity(f"unknown ingredient: {line.ingredient_id}")
        self.recipes[recipe.id] = recipe

    def stock(self, ingredient_id: str, quantity, unit: str) -> None:
        """Set (not add) the pantry amount for an ingredient."""
        if ingredient_id not in self.ingredients:
            raise UnknownEntity(f"unknown ingredient: {ingredient_id}")
        self.pantry[ingredient_id] = PantryItem(qty(quantity), unit)

    def add_stock(self, ingredient_id: str, quantity, unit: str) -> None:
        """Increase pantry stock; fails on unit mismatch."""
        if ingredient_id not in self.ingredients:
            raise UnknownEntity(f"unknown ingredient: {ingredient_id}")
        q = qty(quantity)
        existing = self.pantry.get(ingredient_id)
        if existing is None:
            self.pantry[ingredient_id] = PantryItem(q, unit)
            return
        if existing.unit != unit:
            raise UnitMismatch(
                f"{ingredient_id}: pantry in {existing.unit}, adding {unit}")
        existing.quantity += q

    # --- scaling ---
    def scale(self, recipe_id: str, factor) -> Recipe:
        if recipe_id not in self.recipes:
            raise UnknownEntity(f"unknown recipe: {recipe_id}")
        return self.recipes[recipe_id].scaled(qty(factor))

    def _required_lines(self, recipe_id: str, servings: Optional[int]) -> list[RecipeLine]:
        if recipe_id not in self.recipes:
            raise UnknownEntity(f"unknown recipe: {recipe_id}")
        recipe = self.recipes[recipe_id]
        if servings is None or servings == recipe.serves:
            return list(recipe.lines)
        factor = Fraction(servings, recipe.serves)
        return [l.scaled(factor) for l in recipe.lines]

    # --- pantry evaluation ---
    def can_make(self, recipe_id: str, servings: Optional[int] = None
                 ) -> tuple[bool, list[Shortage]]:
        """Conjunction over recipe lines: all must be satisfiable from the pantry."""
        shortages: list[Shortage] = []
        for line in self._required_lines(recipe_id, servings):
            p = self.pantry.get(line.ingredient_id)
            if p is None:
                shortages.append(Shortage(
                    line.ingredient_id, line.quantity, Fraction(0),
                    line.unit, reason="not_stocked"))
                continue
            if p.unit != line.unit:
                shortages.append(Shortage(
                    line.ingredient_id, line.quantity, Fraction(0),
                    line.unit, reason="unit_mismatch"))
                continue
            if p.quantity < line.quantity:
                shortages.append(Shortage(
                    line.ingredient_id, line.quantity, p.quantity, line.unit))
        return (not shortages, shortages)

    def shopping_list(self, recipe_id: str, servings: Optional[int] = None
                      ) -> list[RecipeLine]:
        """Gap report: required minus available, per line. Only lines needing more appear."""
        out: list[RecipeLine] = []
        for line in self._required_lines(recipe_id, servings):
            p = self.pantry.get(line.ingredient_id)
            if p is None or p.unit != line.unit:
                out.append(RecipeLine(line.ingredient_id, line.quantity, line.unit))
                continue
            deficit = line.quantity - p.quantity
            if deficit > 0:
                out.append(RecipeLine(line.ingredient_id, deficit, line.unit))
        return out

    def make(self, recipe_id: str, servings: Optional[int] = None) -> Recipe:
        """Atomically consume all required ingredients. All-or-nothing."""
        ok, shortages = self.can_make(recipe_id, servings)
        if not ok:
            raise InsufficientStock(
                f"cannot make {recipe_id}: {len(shortages)} shortage(s): "
                f"{[(s.ingredient_id, s.reason) for s in shortages]}")
        for line in self._required_lines(recipe_id, servings):
            self.pantry[line.ingredient_id].quantity -= line.quantity
        return self.recipes[recipe_id]

    # --- queries over the recipe graph ---
    def recipes_using(self, ingredient_id: str) -> list[Recipe]:
        if ingredient_id not in self.ingredients:
            raise UnknownEntity(f"unknown ingredient: {ingredient_id}")
        return [r for r in self.recipes.values()
                if any(l.ingredient_id == ingredient_id for l in r.lines)]

    def ingredients_used(self) -> set[str]:
        """Every ingredient referenced by any recipe."""
        used: set[str] = set()
        for r in self.recipes.values():
            for l in r.lines:
                used.add(l.ingredient_id)
        return used


# --- demo ---
if __name__ == "__main__":
    book = RecipeBook()
    for i in [
        Ingredient("flour",  "all-purpose flour", "g"),
        Ingredient("sugar",  "white sugar",       "g"),
        Ingredient("butter", "unsalted butter",   "g"),
        Ingredient("egg",    "large egg",         "whole"),
        Ingredient("vanilla","vanilla extract",   "tsp"),
    ]:
        book.add_ingredient(i)

    cookies = Recipe(
        id="cookies", name="Sugar Cookies", serves=24,
        lines=[
            RecipeLine("flour",   qty(300), "g"),
            RecipeLine("sugar",   qty(200), "g"),
            RecipeLine("butter",  qty(225), "g"),
            RecipeLine("egg",     qty(2),   "whole"),
            RecipeLine("vanilla", qty(1),   "tsp"),
        ],
        steps=["cream butter and sugar", "add egg and vanilla",
               "fold in flour", "bake at 180C for 12 min"],
    )
    book.add_recipe(cookies)

    # Pantry has some, not all.
    book.stock("flour",  500, "g")
    book.stock("sugar",  150, "g")        # short on sugar
    book.stock("butter", 250, "g")
    book.stock("vanilla", 2,  "tsp")
    # egg is not stocked at all.

    ok, shortages = book.can_make("cookies")
    print(f"can make 24 cookies? {ok}")
    for s in shortages:
        print(f"  - {s.ingredient_id}: need {s.required}{s.unit}, "
              f"have {s.available}{s.unit} ({s.reason})")

    print("\nshopping list for 24 cookies:")
    for line in book.shopping_list("cookies"):
        print(f"  buy {line.quantity} {line.unit} of {line.ingredient_id}")

    # Scale down to 6 cookies and try again.
    ok6, _ = book.can_make("cookies", servings=6)
    print(f"\ncan make 6 cookies? {ok6}")
    if not ok6:
        for line in book.shopping_list("cookies", servings=6):
            print(f"  still need {line.quantity} {line.unit} of {line.ingredient_id}")

    # Top up the pantry, then actually make.
    book.add_stock("sugar", 100, "g")
    book.stock("egg", 6, "whole")
    book.make("cookies", servings=12)
    print("\nafter making 12 cookies, pantry:")
    for iid, item in sorted(book.pantry.items()):
        print(f"  {iid}: {item.quantity} {item.unit}")
