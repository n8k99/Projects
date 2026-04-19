"""Vending Machine — a finite state machine with coin-constrained change-making.

First project in the Rosetta Stone where operations are gated by the class's
STATE, not (only) by its DATA. `insert_coin`, `select`, and `refund` are legal
in different states and illegal in others. Purchase is an atomic transition
that touches three pieces of state simultaneously: slot inventory, machine
coin inventory (consuming change coins, adding inserted coins), and the
state itself.

Change-making reuses G007's greedy algorithm but is constrained by the
machine's current coin stock: if it can't assemble the owed amount from
what it has, the purchase fails cleanly (coins refunded, state resets).
"""

from __future__ import annotations
from dataclasses import dataclass, field
from enum import Enum


class MachineState(Enum):
    IDLE = "idle"              # no coins inserted
    ACCEPTING = "accepting"    # coins inserted, waiting for selection


# Standard US coin denominations (cents). Ordered high → low for greedy change.
COINS_CENTS = (100, 25, 10, 5, 1)


@dataclass
class Slot:
    slot_id: str
    product: str
    price_cents: int
    quantity: int


class VendingError(Exception): ...
class UnknownSlot(VendingError): ...
class SoldOut(VendingError): ...
class InsufficientFunds(VendingError): ...
class CannotMakeChange(VendingError): ...
class BadState(VendingError): ...
class InvalidCoin(VendingError): ...


@dataclass
class VendingMachine:
    slots: dict[str, Slot] = field(default_factory=dict)
    coin_inventory: dict[int, int] = field(
        default_factory=lambda: {d: 0 for d in COINS_CENTS})
    inserted_cents: int = 0
    inserted_coins: dict[int, int] = field(
        default_factory=lambda: {d: 0 for d in COINS_CENTS})
    state: MachineState = MachineState.IDLE

    # --- setup ---
    def add_slot(self, slot: Slot) -> None:
        if slot.slot_id in self.slots:
            raise ValueError(f"slot already exists: {slot.slot_id}")
        if slot.price_cents <= 0:
            raise ValueError("price must be positive")
        self.slots[slot.slot_id] = slot

    def load_change(self, denom: int, count: int) -> None:
        if denom not in COINS_CENTS:
            raise InvalidCoin(f"not a recognized denomination: {denom}")
        if count < 0:
            raise ValueError("count must be non-negative")
        self.coin_inventory[denom] = self.coin_inventory.get(denom, 0) + count

    # --- interactive operations ---
    def insert_coin(self, denom: int) -> int:
        """Accept a coin. Transitions IDLE→ACCEPTING on the first coin.
        Returns new inserted total."""
        if self.state not in (MachineState.IDLE, MachineState.ACCEPTING):
            raise BadState(f"cannot insert coin while {self.state.value}")
        if denom not in COINS_CENTS:
            raise InvalidCoin(f"not a recognized denomination: {denom}")
        self.inserted_cents += denom
        self.inserted_coins[denom] = self.inserted_coins.get(denom, 0) + 1
        self.state = MachineState.ACCEPTING
        return self.inserted_cents

    def refund(self) -> dict[int, int]:
        """Return all inserted coins. Transitions back to IDLE. Legal in any state."""
        refunded = dict(self.inserted_coins)
        self.inserted_cents = 0
        self.inserted_coins = {d: 0 for d in COINS_CENTS}
        self.state = MachineState.IDLE
        return {d: c for d, c in refunded.items() if c > 0}

    def select(self, slot_id: str) -> tuple[str, dict[int, int]]:
        """Atomically dispense product + change. Returns (product, change_coins).

        Transitions: ACCEPTING → IDLE on success. On failure, state is unchanged
        and coins remain inserted (caller may refund or insert more).
        """
        if self.state != MachineState.ACCEPTING:
            raise BadState(f"cannot select while {self.state.value}")
        if slot_id not in self.slots:
            raise UnknownSlot(f"unknown slot: {slot_id}")
        slot = self.slots[slot_id]
        if slot.quantity <= 0:
            raise SoldOut(f"{slot_id} ({slot.product}) is sold out")
        if self.inserted_cents < slot.price_cents:
            raise InsufficientFunds(
                f"{slot.product} is {slot.price_cents}¢, "
                f"{self.inserted_cents}¢ inserted")

        change_owed = self.inserted_cents - slot.price_cents
        # Merge inserted coins into the machine's available change pool
        # SO the machine can potentially use them to make change.
        pool = dict(self.coin_inventory)
        for d, c in self.inserted_coins.items():
            pool[d] = pool.get(d, 0) + c
        change = _greedy_change(change_owed, pool)
        if change is None:
            raise CannotMakeChange(
                f"cannot make {change_owed}¢ from available coins; refunding")

        # Atomic commit: subtract change from pool, update inventory accordingly.
        for d, c in change.items():
            pool[d] -= c
        self.coin_inventory = pool
        slot.quantity -= 1
        self.inserted_cents = 0
        self.inserted_coins = {d: 0 for d in COINS_CENTS}
        self.state = MachineState.IDLE
        return (slot.product, {d: c for d, c in change.items() if c > 0})

    # --- queries ---
    def available(self, slot_id: str) -> bool:
        return slot_id in self.slots and self.slots[slot_id].quantity > 0

    def total_coins_cents(self) -> int:
        return sum(d * c for d, c in self.coin_inventory.items())


def _greedy_change(amount: int, pool: dict[int, int]) -> dict[int, int] | None:
    """Try to make `amount` cents from `pool` using high-denom-first greedy.
    Returns the coin counts used, or None if impossible with these coins."""
    result: dict[int, int] = {}
    remaining = amount
    for d in sorted(pool.keys(), reverse=True):
        if d > remaining:
            continue
        use = min(pool[d], remaining // d)
        if use > 0:
            result[d] = use
            remaining -= d * use
        if remaining == 0:
            break
    return result if remaining == 0 else None


# --- demo ---
if __name__ == "__main__":
    m = VendingMachine()
    m.add_slot(Slot("A1", "Cola",    price_cents=125, quantity=5))
    m.add_slot(Slot("B2", "Chips",   price_cents=100, quantity=3))
    m.add_slot(Slot("C3", "Gum",     price_cents=50,  quantity=10))
    m.load_change(25, 20)
    m.load_change(10, 20)
    m.load_change(5, 20)
    m.load_change(1, 50)

    # Try to select without coins — state gate refuses.
    try:
        m.select("A1")
    except BadState as e:
        print(f"refused: {e}")

    # Insert $1.50, buy Cola ($1.25), get 25¢ change.
    m.insert_coin(100)
    m.insert_coin(25)
    m.insert_coin(25)
    print(f"inserted: {m.inserted_cents}¢  state: {m.state.value}")
    product, change = m.select("A1")
    print(f"dispensed: {product}; change: {change}  state: {m.state.value}")

    # Deliberately run down change and try a purchase the machine can't fulfill.
    m2 = VendingMachine()
    m2.add_slot(Slot("G1", "Gum", price_cents=30, quantity=10))
    m2.load_change(25, 0)
    m2.load_change(10, 0)
    m2.load_change(5, 0)          # no small change at all
    m2.load_change(1, 0)
    m2.insert_coin(100)
    try:
        m2.select("G1")
    except CannotMakeChange as e:
        print(f"refused: {e}")
        refunded = m2.refund()
        print(f"refunded: {refunded}")
