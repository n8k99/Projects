"""RPG Character Stat Creator — class templates + derived stats + level progression.

Base stats (Strength, Dexterity, Intellect, Stamina, Luck). Class multipliers
scale base → adjusted. Derived stats (HP, MP, armour, crit) compute every read.
Levels grant 5 points to spend; class caps enforce maxima.
"""

from __future__ import annotations
from dataclasses import dataclass, field
from enum import Enum


class Stat(Enum):
    STRENGTH = "strength"
    DEXTERITY = "dexterity"
    INTELLECT = "intellect"
    STAMINA = "stamina"
    LUCK = "luck"


ALL_STATS = [Stat.STRENGTH, Stat.DEXTERITY, Stat.INTELLECT, Stat.STAMINA, Stat.LUCK]


class Class(Enum):
    WARRIOR = "warrior"
    MAGE = "mage"
    ROGUE = "rogue"


def multipliers(cls: Class) -> dict[Stat, float]:
    m = {s: 1.0 for s in ALL_STATS}
    if cls == Class.WARRIOR:
        m[Stat.STRENGTH] = 1.5
        m[Stat.STAMINA] = 1.3
    elif cls == Class.MAGE:
        m[Stat.INTELLECT] = 1.5
        m[Stat.STAMINA] = 0.8
    elif cls == Class.ROGUE:
        m[Stat.DEXTERITY] = 1.4
        m[Stat.LUCK] = 1.3
    return m


def max_per_stat(cls: Class) -> int:
    return 99


@dataclass
class Derived:
    max_hp: int
    max_mp: int
    armour: int
    carry_capacity: int
    crit_chance: float


class LcgRng:
    def __init__(self, seed: int) -> None:
        self.state = (seed + 1) & 0xFFFFFFFFFFFFFFFF

    def next_u32(self) -> int:
        self.state = (self.state * 6364136223846793005 + 1442695040888963407) & 0xFFFFFFFFFFFFFFFF
        return (self.state >> 33) & 0xFFFFFFFF

    def next_in(self, lo: int, range: int) -> int:
        return lo + (self.next_u32() % range)


@dataclass
class Character:
    name: str
    class_: Class
    level: int = 1
    base: dict[Stat, int] = field(default_factory=dict)
    unspent_points: int = 0

    @classmethod
    def roll(cls, name: str, class_: Class, seed: int) -> Character:
        rng = LcgRng(seed)
        base = {}
        for stat in ALL_STATS:
            base[stat] = 3 + rng.next_in(0, 16)
        return cls(name=name, class_=class_, level=1, base=base, unspent_points=0)

    def adjusted(self, stat: Stat) -> int:
        b = self.base.get(stat, 0)
        m = multipliers(self.class_).get(stat, 1.0)
        return int(b * m)

    def derived(self) -> Derived:
        str_ = self.adjusted(Stat.STRENGTH)
        int_ = self.adjusted(Stat.INTELLECT)
        sta = self.adjusted(Stat.STAMINA)
        dex = self.adjusted(Stat.DEXTERITY)
        luk = self.adjusted(Stat.LUCK)
        return Derived(
            max_hp=10 + sta * 5 + self.level * 10,
            max_mp=int_ * 3 + self.level * 5,
            armour=str_ // 3 + sta // 5,
            carry_capacity=10 + str_ * 2,
            crit_chance=(luk * 0.5 + dex * 0.2) / 100.0,
        )

    def level_up(self) -> None:
        self.level += 1
        self.unspent_points += 5

    def spend_point(self, stat: Stat, points: int) -> None:
        if points > self.unspent_points:
            raise ValueError(f"not enough points: have {self.unspent_points}, need {points}")
        current = self.base.get(stat, 0)
        new_val = current + points
        cap = max_per_stat(self.class_)
        if new_val > cap:
            raise ValueError(f"stat cap exceeded: {new_val} > {cap}")
        self.base[stat] = new_val
        self.unspent_points -= points

    def to_text(self) -> str:
        lines = [
            f"name: {self.name}",
            f"class: {self.class_.value}",
            f"level: {self.level}",
            f"unspent: {self.unspent_points}",
        ]
        for stat in ALL_STATS:
            if stat in self.base:
                lines.append(f"stat:{stat.value}={self.base[stat]}")
        return "\n".join(lines) + "\n"


def demo() -> None:
    c = Character.roll("Rook", Class.WARRIOR, 42)
    print(f"=== {c.name} the {c.class_.value} ===")
    for s in ALL_STATS:
        print(f"  {s.value:10} base={c.base[s]:2}  adj={c.adjusted(s):2}")
    d = c.derived()
    print(f"\n  HP: {d.max_hp}  MP: {d.max_mp}  Armour: {d.armour}")
    print(f"  Carry: {d.carry_capacity}  Crit: {d.crit_chance:.4f}")

    c.level_up()
    c.level_up()
    c.spend_point(Stat.STRENGTH, 5)
    c.spend_point(Stat.STAMINA, 3)
    d = c.derived()
    print(f"\n=== After 2 levels + point spend ===")
    print(f"  level={c.level} unspent={c.unspent_points}")
    print(f"  STR={c.base[Stat.STRENGTH]} STA={c.base[Stat.STAMINA]}")
    print(f"  HP: {d.max_hp}  Armour: {d.armour}")


if __name__ == "__main__":
    demo()
