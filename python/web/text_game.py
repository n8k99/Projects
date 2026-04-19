"""Text Based Game — room graph + player state + command dispatch.

Combines earlier primitives: G064 graph (rooms), G073 command dispatch,
G077/G078 state FSMs. Each command is a turn; the world responds (enemies
strike) and produces a log.

The text-adventure shape: describe → prompt → parse → dispatch → mutate
→ describe. Every ancient Infocom game, every MUD, every modern
interactive fiction tool is built on this loop.
"""

from __future__ import annotations
from dataclasses import dataclass, field
from enum import Enum


class ItemKind(Enum):
    WEAPON = "weapon"
    POTION = "potion"
    TREASURE = "treasure"


@dataclass
class Item:
    id: int
    name: str
    kind: ItemKind
    damage: int = 0
    heal: int = 0
    worth_gold: int = 0


@dataclass
class Enemy:
    id: int
    name: str
    health: int
    damage: int


@dataclass
class Room:
    id: int
    name: str
    description: str
    exits: dict[str, int] = field(default_factory=dict)
    items: list[Item] = field(default_factory=list)
    enemies: list[Enemy] = field(default_factory=list)


@dataclass
class Player:
    location: int = 1
    health: int = 20
    max_health: int = 20
    gold: int = 0
    inventory: list[Item] = field(default_factory=list)
    equipped_weapon_id: int | None = None
    equipped_damage: int = 1   # bare hands


class GameState(Enum):
    PLAYING = "playing"
    DEAD = "dead"
    WON = "won"


_DIR_ALIASES = {"n": "north", "s": "south", "e": "east", "w": "west"}
_PROVOKES = {"go", "move", "north", "south", "east", "west",
             "n", "s", "e", "w", "take", "get", "use", "fight", "attack"}


class World:
    def __init__(self) -> None:
        self.rooms: dict[int, Room] = {}
        self.player = Player()
        self.state = GameState.PLAYING
        self.turn = 0
        self.win_condition_gold = 50
        self.log: list[str] = []

    @classmethod
    def demo(cls) -> "World":
        w = cls()
        sword = Item(100, "rusty sword",    ItemKind.WEAPON,   damage=5)
        potion = Item(101, "healing potion", ItemKind.POTION,   heal=15)
        treasure = Item(102, "gold chalice",  ItemKind.TREASURE, worth_gold=50)
        goblin = Enemy(200, "goblin", health=10, damage=2)

        w.rooms[1] = Room(1, "Entrance Hall",
                          "A dusty hall. Torchlight flickers on the walls. "
                          "Doorways lead north and east.",
                          exits={"north": 2, "east": 3},
                          items=[sword, potion])
        w.rooms[2] = Room(2, "Damp Cavern",
                          "A cold cavern smelling of mildew. Something shifts "
                          "in the shadows. A passage leads south.",
                          exits={"south": 1}, enemies=[goblin])
        w.rooms[3] = Room(3, "Treasure Room",
                          "A small room gleaming with gold. A lone chalice "
                          "rests on a pedestal. Exit west.",
                          exits={"west": 1}, items=[treasure])

        w.log.append("You stand at the entrance of an old dungeon.")
        return w

    def current_room(self) -> Room:
        return self.rooms[self.player.location]

    def execute(self, cmd: str) -> list[str]:
        before = len(self.log)
        if self.state is not GameState.PLAYING:
            self.log.append("The game is over. Reset to play again.")
            return self.log[before:]

        parts = cmd.strip().lower().split()
        if not parts:
            return []
        verb = parts[0]
        arg = " ".join(parts[1:])

        if verb in _DIR_ALIASES:
            verb, arg = "go", _DIR_ALIASES[verb]
        elif verb in ("north", "south", "east", "west"):
            arg = verb
            verb = "go"

        dispatch = {
            "look": self._look, "l": self._look,
            "go": self._go, "move": self._go,
            "take": self._take, "get": self._take,
            "use": self._use,
            "equip": self._equip, "wield": self._equip,
            "fight": self._fight, "attack": self._fight,
            "inventory": self._inventory, "inv": self._inventory, "i": self._inventory,
            "stats": self._stats,
        }
        handler = dispatch.get(verb)
        if handler is None:
            self.log.append(f"I don't understand '{verb}'.")
        else:
            handler(arg)

        raw = cmd.strip().lower().split()[0] if cmd.strip() else ""
        if raw in _PROVOKES:
            self.turn += 1
            self._enemies_strike()
            self._check_end_conditions()
        return self.log[before:]

    def _look(self, _arg: str) -> None:
        r = self.current_room()
        self.log.append(f"{r.name}: {r.description}")
        if r.items:
            self.log.append("You see here: " + ", ".join(i.name for i in r.items))
        if r.enemies:
            self.log.append("Enemies: " +
                            ", ".join(f"{e.name} ({e.health}hp)" for e in r.enemies))
        self.log.append("Exits: " + ", ".join(sorted(r.exits)))

    def _go(self, direction: str) -> None:
        if not direction:
            self.log.append("Go where?")
            return
        r = self.current_room()
        dest = r.exits.get(direction)
        if dest is None:
            self.log.append(f"You can't go {direction} from here.")
            return
        self.player.location = dest
        self.log.append(f"You travel {direction}. You are now in {self.current_room().name}.")

    def _take(self, name: str) -> None:
        if not name:
            self.log.append("Take what?")
            return
        r = self.current_room()
        for i, it in enumerate(r.items):
            if name in it.name.lower():
                taken = r.items.pop(i)
                self.player.inventory.append(taken)
                self.log.append(f"You take the {taken.name}.")
                return
        self.log.append(f"There is no '{name}' here.")

    def _use(self, name: str) -> None:
        if not name:
            self.log.append("Use what?")
            return
        for i, it in enumerate(self.player.inventory):
            if name in it.name.lower():
                item = self.player.inventory.pop(i)
                if item.kind is ItemKind.POTION:
                    before = self.player.health
                    self.player.health = min(self.player.max_health,
                                              self.player.health + item.heal)
                    self.log.append(f"You drink the {item.name}. "
                                     f"Health {before} → {self.player.health}.")
                elif item.kind is ItemKind.TREASURE:
                    self.player.gold += item.worth_gold
                    self.log.append(f"You pocket the {item.name} "
                                     f"(+{item.worth_gold} gold, total {self.player.gold}).")
                elif item.kind is ItemKind.WEAPON:
                    self.player.equipped_weapon_id = item.id
                    self.player.equipped_damage = item.damage
                    self.log.append(f"You wield the {item.name} ({item.damage} damage).")
                    self.player.inventory.append(item)
                return
        self.log.append(f"You don't have a '{name}'.")

    def _equip(self, name: str) -> None:
        if not name:
            self.log.append("Equip what?")
            return
        for it in self.player.inventory:
            if name in it.name.lower():
                if it.kind is not ItemKind.WEAPON:
                    self.log.append(f"The {it.name} is not a weapon.")
                    return
                self.player.equipped_weapon_id = it.id
                self.player.equipped_damage = it.damage
                self.log.append(f"You wield the {it.name} ({it.damage} damage).")
                return
        self.log.append(f"You don't have a '{name}'.")

    def _fight(self, name: str) -> None:
        r = self.current_room()
        target_idx = None
        if not name and r.enemies:
            target_idx = 0
        elif name:
            for i, e in enumerate(r.enemies):
                if name in e.name.lower():
                    target_idx = i
                    break
        if target_idx is None:
            self.log.append(f"There's no '{name}' to fight here.")
            return
        enemy = r.enemies[target_idx]
        dmg = self.player.equipped_damage
        enemy.health -= dmg
        self.log.append(f"You hit the {enemy.name} for {dmg} damage "
                         f"({max(0, enemy.health)}hp remaining).")
        if enemy.health <= 0:
            defeated = r.enemies.pop(target_idx)
            self.log.append(f"The {defeated.name} falls defeated.")

    def _inventory(self, _arg: str) -> None:
        if not self.player.inventory:
            self.log.append("Your pack is empty.")
            return
        self.log.append("You carry: " +
                         ", ".join(i.name for i in self.player.inventory) + ".")

    def _stats(self, _arg: str) -> None:
        wpn = next((i for i in self.player.inventory
                    if i.id == self.player.equipped_weapon_id), None)
        wpn_name = wpn.name if wpn else "bare hands"
        self.log.append(f"Turn {self.turn}. Health {self.player.health}/"
                         f"{self.player.max_health}. Gold {self.player.gold}. "
                         f"Damage {self.player.equipped_damage} (equipped: {wpn_name}).")

    def _enemies_strike(self) -> None:
        r = self.current_room()
        total = sum(e.damage for e in r.enemies)
        if total > 0:
            self.player.health -= total
            self.log.append(f"Enemies strike you for {total} damage "
                             f"({max(0, self.player.health)} hp)!")

    def _check_end_conditions(self) -> None:
        if self.player.health <= 0:
            self.state = GameState.DEAD
            self.log.append("You fall, lifeless. The dungeon is silent.")
            return
        if self.player.gold >= self.win_condition_gold:
            self.state = GameState.WON
            self.log.append(f"With {self.player.gold} gold in hand, "
                             "you've proven your fortune. You win!")


if __name__ == "__main__":
    w = World.demo()
    for cmd in [
        "look",
        "take sword",
        "equip sword",
        "take potion",
        "go east",
        "take chalice",
        "use chalice",
    ]:
        print(f"> {cmd}")
        for line in w.execute(cmd):
            print(f"  {line}")
        print(f"  [state: {w.state.value}, hp: {w.player.health}, gold: {w.player.gold}]")
        print()
        if w.state is not GameState.PLAYING:
            break
