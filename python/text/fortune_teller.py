"""Fortune Teller — randomized wisdom from categorized pools."""

import random


class FortuneTeller:
    """A fortune/prediction generator with categorized pools of wisdom."""

    CATEGORIES = ("wisdom", "humor", "warning", "motivation", "mystery")

    def __init__(self):
        self._pools: dict[str, list[str]] = {cat: [] for cat in self.CATEGORIES}
        self._load_defaults()

    def _load_defaults(self):
        """Pre-load fortunes into each category."""
        defaults = {
            "wisdom": [
                "The obstacle is the path.",
                "What you resist persists.",
                "Still water runs deep.",
                "The map is not the territory.",
                "Every expert was once a beginner.",
            ],
            "humor": [
                "You will step on a Lego in the dark tonight.",
                "A closed mouth gathers no foot.",
                "Today is a good day to avoid making eye contact.",
                "Your socks will never match again.",
            ],
            "warning": [
                "Beware the shortcut that saves no time.",
                "Not every open door is an invitation.",
                "The comfortable path leads to the boring destination.",
                "Trust your instincts — they remember what you forgot.",
            ],
            "motivation": [
                "You are closer than you think.",
                "The best time to start was yesterday. The second best is now.",
                "Doubt kills more dreams than failure ever will.",
                "Small steps still move you forward.",
                "Discipline is choosing what you want most over what you want now.",
            ],
            "mystery": [
                "Something lost will find you when you stop looking.",
                "The answer you seek is in a room you haven't entered yet.",
                "A stranger already knows your name.",
                "The next full moon brings clarity.",
            ],
        }
        for cat, fortunes in defaults.items():
            self._pools[cat] = list(fortunes)

    def tell_fortune(self) -> str:
        """Return a random fortune from all categories."""
        all_fortunes = [f for pool in self._pools.values() for f in pool]
        if not all_fortunes:
            return "The spirits are silent."
        return random.choice(all_fortunes)

    def tell_fortune_by_category(self, category: str) -> str:
        """Return a random fortune from the specified category."""
        if category not in self._pools:
            return f"Unknown category: {category}"
        pool = self._pools[category]
        if not pool:
            return f"No fortunes in category: {category}"
        return random.choice(pool)

    def add_fortune(self, text: str, category: str) -> None:
        """Add a fortune to a category pool."""
        if category not in self._pools:
            self._pools[category] = []
        self._pools[category].append(text)

    def ask_question(self, question: str) -> str:
        """Acknowledge the question, then return a random fortune regardless."""
        _ = question  # acknowledged but not used
        return self.tell_fortune()

    def fortune_count(self) -> int:
        """Return the total number of fortunes across all categories."""
        return sum(len(pool) for pool in self._pools.values())


if __name__ == "__main__":
    ft = FortuneTeller()
    print(f"Fortune pool: {ft.fortune_count()} fortunes\n")

    print("--- Random fortunes ---")
    for _ in range(3):
        print(f"  {ft.tell_fortune()}")

    print("\n--- By category (motivation) ---")
    print(f"  {ft.tell_fortune_by_category('motivation')}")

    print("\n--- Ask a question ---")
    q = "Will I finish this project on time?"
    print(f"  Q: {q}")
    print(f"  A: {ft.ask_question(q)}")

    print("\n--- Add a custom fortune ---")
    ft.add_fortune("Code compiles on the first try today.", "humor")
    print(f"  Pool now: {ft.fortune_count()} fortunes")
    print(f"  Humor: {ft.tell_fortune_by_category('humor')}")
