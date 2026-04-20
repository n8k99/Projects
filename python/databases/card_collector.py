"""Card Collector — multiset inventory with valuation and trade balancing.

Collection is a multiset keyed by (card_id, condition). Valuation combines
base_price * rarity_mult * condition_mult. Trade evaluator compares two
bundles by value with a configurable fairness tolerance.
"""

from __future__ import annotations
from dataclasses import dataclass, field
from enum import Enum


class Rarity(Enum):
    COMMON = "common"
    UNCOMMON = "uncommon"
    RARE = "rare"
    MYTHIC = "mythic"


_RARITY_MULT = {
    Rarity.COMMON: 1.0, Rarity.UNCOMMON: 2.0,
    Rarity.RARE: 8.0, Rarity.MYTHIC: 25.0,
}


class Condition(Enum):
    MINT = "mint"
    NEAR_MINT = "near_mint"
    PLAYED = "played"
    DAMAGED = "damaged"


_COND_MULT = {
    Condition.MINT: 1.0, Condition.NEAR_MINT: 0.8,
    Condition.PLAYED: 0.4, Condition.DAMAGED: 0.15,
}


@dataclass
class Card:
    set_code: str
    number: int
    name: str
    rarity: Rarity
    base_price_cents: int

    def id(self) -> str:
        return f"{self.set_code}-{self.number:03d}"

    def value_cents(self, condition: Condition) -> int:
        v = self.base_price_cents * _RARITY_MULT[self.rarity] * _COND_MULT[condition]
        return round(v)


@dataclass
class Catalogue:
    cards: dict[str, Card] = field(default_factory=dict)

    def add(self, card: Card) -> None:
        self.cards[card.id()] = card

    def get(self, card_id: str) -> Card | None:
        return self.cards.get(card_id)

    def all_ids(self) -> list[str]:
        return sorted(self.cards.keys())


@dataclass(frozen=True)
class InventoryKey:
    card_id: str
    condition: Condition


@dataclass
class Collection:
    counts: dict[InventoryKey, int] = field(default_factory=dict)

    def add(self, card_id: str, condition: Condition, qty: int) -> None:
        k = InventoryKey(card_id, condition)
        self.counts[k] = self.counts.get(k, 0) + qty

    def remove(self, card_id: str, condition: Condition, qty: int) -> None:
        k = InventoryKey(card_id, condition)
        current = self.counts.get(k, 0)
        if current < qty:
            raise ValueError(f"not enough {card_id} — have {current}, need {qty}")
        if current == qty:
            del self.counts[k]
        else:
            self.counts[k] = current - qty

    def total_cards(self) -> int:
        return sum(self.counts.values())

    def unique_cards(self) -> int:
        return len({k.card_id for k in self.counts})

    def total_value_cents(self, cat: Catalogue) -> int:
        total = 0
        for k, n in self.counts.items():
            c = cat.get(k.card_id)
            if c is not None:
                total += c.value_cents(k.condition) * n
        return total

    def missing(self, cat: Catalogue) -> list[str]:
        owned = {k.card_id for k in self.counts}
        return [i for i in cat.all_ids() if i not in owned]


@dataclass
class TradeBundle:
    cards: list[tuple[str, Condition, int]] = field(default_factory=list)

    def add(self, card_id: str, condition: Condition, qty: int) -> None:
        self.cards.append((card_id, condition, qty))

    def value_cents(self, cat: Catalogue) -> int:
        total = 0
        for cid, cond, qty in self.cards:
            c = cat.get(cid)
            if c is not None:
                total += c.value_cents(cond) * qty
        return total


class TradeVerdict(Enum):
    FAIR = "fair"
    OFFERED_MORE = "offered_more"
    ASKED_MORE = "asked_more"


@dataclass
class TradeEvaluation:
    verdict: TradeVerdict
    diff_cents: int


def evaluate_trade(offered: TradeBundle, asked: TradeBundle,
                    cat: Catalogue, tolerance_cents: int) -> TradeEvaluation:
    o = offered.value_cents(cat)
    a = asked.value_cents(cat)
    diff = abs(o - a)
    if diff <= tolerance_cents:
        return TradeEvaluation(TradeVerdict.FAIR, 0)
    if o > a:
        return TradeEvaluation(TradeVerdict.OFFERED_MORE, o - a)
    return TradeEvaluation(TradeVerdict.ASKED_MORE, a - o)


def demo() -> None:
    cat = Catalogue()
    cat.add(Card("DPN", 1, "Common One", Rarity.COMMON, 25))
    cat.add(Card("DPN", 2, "Uncommon Two", Rarity.UNCOMMON, 25))
    cat.add(Card("DPN", 3, "Rare Three", Rarity.RARE, 25))
    cat.add(Card("DPN", 4, "Mythic Four", Rarity.MYTHIC, 100))

    c = Collection()
    c.add("DPN-001", Condition.MINT, 4)
    c.add("DPN-003", Condition.NEAR_MINT, 1)
    c.add("DPN-004", Condition.MINT, 1)
    print(f"total cards:   {c.total_cards()}")
    print(f"unique cards:  {c.unique_cards()}")
    print(f"total value:   {c.total_value_cents(cat)} cents")
    print(f"missing cards: {c.missing(cat)}")

    print("\n=== trade evaluation ===")
    offered = TradeBundle()
    offered.add("DPN-004", Condition.MINT, 1)  # 2500
    asked = TradeBundle()
    asked.add("DPN-003", Condition.MINT, 12)   # 2400
    asked.add("DPN-001", Condition.MINT, 4)    # 100
    verdict = evaluate_trade(offered, asked, cat, tolerance_cents=100)
    print(f"  offered value: {offered.value_cents(cat)}")
    print(f"  asked value:   {asked.value_cents(cat)}")
    print(f"  verdict:       {verdict.verdict.value} (diff={verdict.diff_cents})")


if __name__ == "__main__":
    demo()
