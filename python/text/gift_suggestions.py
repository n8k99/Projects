"""Random Gift Suggestions — attribute-based gift recommendation."""

from dataclasses import dataclass, field
import random


@dataclass
class Gift:
    """A gift with category, price range, and compatible traits."""
    name: str
    category: str
    price_range: str  # "budget", "mid", "premium"
    suitable_for: list[str] = field(default_factory=list)

    def __str__(self) -> str:
        traits = ", ".join(self.suitable_for)
        return f"{self.name} [{self.category}] ({self.price_range}) — suits: {traits}"


class GiftSuggester:
    """Attribute-based gift recommendation engine.

    Suggests gifts by matching recipient traits against each gift's
    suitable_for list. Relevance = number of matching traits.
    """

    def __init__(self):
        self._gifts: list[Gift] = []
        self._load_defaults()

    def _load_defaults(self):
        """Pre-load the catalog with 24 gifts across categories."""
        defaults = [
            # Tech
            Gift("Mechanical Keyboard", "tech", "mid", ["techie", "creative"]),
            Gift("Raspberry Pi Kit", "tech", "budget", ["techie", "creative"]),
            Gift("Noise-Cancelling Headphones", "tech", "premium", ["techie", "music-lover"]),
            Gift("Smart Home Starter Kit", "tech", "mid", ["techie"]),
            # Books
            Gift("Leather-Bound Journal", "books", "mid", ["bookworm", "creative"]),
            Gift("Complete Tolkien Collection", "books", "premium", ["bookworm", "adventurer"]),
            Gift("Pocket Poetry Anthology", "books", "budget", ["bookworm", "creative"]),
            Gift("Cookbook: World Cuisines", "books", "mid", ["bookworm", "foodie"]),
            # Outdoor
            Gift("Hammock", "outdoor", "budget", ["adventurer"]),
            Gift("Hiking Backpack", "outdoor", "mid", ["adventurer"]),
            Gift("Camping Cookset", "outdoor", "mid", ["adventurer", "foodie"]),
            Gift("Trail Running Shoes", "outdoor", "premium", ["adventurer"]),
            # Cooking
            Gift("Cast Iron Skillet", "cooking", "budget", ["foodie"]),
            Gift("Spice Collection Box", "cooking", "mid", ["foodie", "adventurer"]),
            Gift("Chef's Knife Set", "cooking", "premium", ["foodie"]),
            Gift("Pasta Maker", "cooking", "mid", ["foodie", "creative"]),
            # Music
            Gift("Vinyl Record Starter Pack", "music", "budget", ["music-lover"]),
            Gift("Concert Tickets", "music", "mid", ["music-lover", "adventurer"]),
            Gift("MIDI Controller", "music", "mid", ["music-lover", "techie", "creative"]),
            Gift("Turntable", "music", "premium", ["music-lover"]),
            # Art
            Gift("Watercolor Set", "art", "budget", ["creative"]),
            Gift("Drawing Tablet", "art", "mid", ["creative", "techie"]),
            Gift("Museum Membership", "art", "mid", ["creative", "bookworm"]),
            Gift("Oil Paint Master Set", "art", "premium", ["creative"]),
        ]
        self._gifts.extend(defaults)

    def add_gift(self, name: str, category: str, price_range: str,
                 suitable_for: list[str] | None = None) -> Gift:
        """Add a gift to the catalog."""
        gift = Gift(name=name, category=category, price_range=price_range,
                    suitable_for=suitable_for or [])
        self._gifts.append(gift)
        return gift

    def suggest(self, traits: list[str], budget: str | None = None) -> list[Gift]:
        """Suggest gifts matching recipient traits, sorted by relevance.

        Relevance = number of matching traits between recipient and gift.
        If budget is given, only gifts in that price range are included.
        """
        trait_set = set(t.lower() for t in traits)
        scored: list[tuple[int, Gift]] = []
        for gift in self._gifts:
            if budget and gift.price_range != budget:
                continue
            matches = len(trait_set & set(s.lower() for s in gift.suitable_for))
            if matches > 0:
                scored.append((matches, gift))
        scored.sort(key=lambda pair: pair[0], reverse=True)
        return [gift for _, gift in scored]

    def random_suggestion(self) -> Gift | None:
        """Return a random gift from the catalog."""
        if not self._gifts:
            return None
        return random.choice(self._gifts)

    def suggest_by_category(self, category: str) -> list[Gift]:
        """Return all gifts in a given category."""
        cat = category.lower()
        return [g for g in self._gifts if g.category.lower() == cat]


if __name__ == "__main__":
    gs = GiftSuggester()

    print("=== Gifts for a techie bookworm (mid budget) ===")
    for gift in gs.suggest(["techie", "bookworm"], budget="mid"):
        print(f"  {gift}")

    print()
    print("=== Gifts for a creative adventurer (premium) ===")
    for gift in gs.suggest(["creative", "adventurer"], budget="premium"):
        print(f"  {gift}")

    print()
    print("=== Random suggestion ===")
    print(f"  {gs.random_suggestion()}")

    print()
    print("=== All cooking gifts ===")
    for gift in gs.suggest_by_category("cooking"):
        print(f"  {gift}")
