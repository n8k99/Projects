"""Product Inventory — entities with identity, movements as a ledger, current quantity as aggregate."""

from dataclasses import dataclass, field
from datetime import datetime, timezone
from typing import Optional


@dataclass
class Product:
    """A product entity with identity (sku) and mutable fields."""
    sku: str
    name: str
    price: float
    category: str = ""
    reorder_point: int = 0


@dataclass
class Movement:
    """A stock movement — receive or sell."""
    sku: str
    kind: str  # "receive" | "sell"
    quantity: int
    at: datetime = field(default_factory=lambda: datetime.now(timezone.utc))
    note: str = ""

    @property
    def signed(self) -> int:
        return self.quantity if self.kind == "receive" else -self.quantity


class InsufficientStock(Exception):
    """Raised when a sell exceeds on-hand quantity."""


class UnknownSku(KeyError):
    """Raised when an sku is not registered."""


class Inventory:
    """An inventory of products. Current quantity is aggregated from movements."""

    def __init__(self) -> None:
        self.products: dict[str, Product] = {}
        self.movements: list[Movement] = []

    # --- registration ---
    def add_product(self, product: Product) -> None:
        if product.sku in self.products:
            raise ValueError(f"sku already registered: {product.sku}")
        self.products[product.sku] = product

    def get(self, sku: str) -> Product:
        if sku not in self.products:
            raise UnknownSku(sku)
        return self.products[sku]

    # --- movements ---
    def receive(self, sku: str, quantity: int, note: str = "") -> Movement:
        self.get(sku)
        if quantity <= 0:
            raise ValueError("receive quantity must be positive")
        m = Movement(sku=sku, kind="receive", quantity=quantity, note=note)
        self.movements.append(m)
        return m

    def sell(self, sku: str, quantity: int, note: str = "") -> Movement:
        self.get(sku)
        if quantity <= 0:
            raise ValueError("sell quantity must be positive")
        if self.quantity(sku) < quantity:
            raise InsufficientStock(
                f"{sku}: on hand {self.quantity(sku)}, requested {quantity}"
            )
        m = Movement(sku=sku, kind="sell", quantity=quantity, note=note)
        self.movements.append(m)
        return m

    # --- aggregates ---
    def quantity(self, sku: str) -> int:
        self.get(sku)
        return sum(m.signed for m in self.movements if m.sku == sku)

    def history(self, sku: str) -> list[Movement]:
        self.get(sku)
        return [m for m in self.movements if m.sku == sku]

    def total_value(self) -> float:
        return sum(p.price * self.quantity(p.sku) for p in self.products.values())

    def restock_needed(self) -> list[Product]:
        """Products whose current quantity is at or below their reorder point."""
        return [p for p in self.products.values()
                if p.reorder_point > 0 and self.quantity(p.sku) <= p.reorder_point]

    def by_category(self, category: str) -> list[Product]:
        return [p for p in self.products.values() if p.category == category]

    def summary(self) -> dict:
        return {
            "products": len(self.products),
            "movements": len(self.movements),
            "total_value": round(self.total_value(), 2),
            "restock_count": len(self.restock_needed()),
        }


# --- demo ---
if __name__ == "__main__":
    inv = Inventory()
    inv.add_product(Product("SKU-001", "Widget", 9.99, "tools", reorder_point=5))
    inv.add_product(Product("SKU-002", "Gadget", 24.50, "tools", reorder_point=3))
    inv.add_product(Product("SKU-003", "Sprocket", 2.25, "parts", reorder_point=20))

    inv.receive("SKU-001", 50, note="initial stock")
    inv.receive("SKU-002", 10, note="initial stock")
    inv.receive("SKU-003", 15, note="initial stock")
    inv.sell("SKU-001", 3)
    inv.sell("SKU-002", 8)

    print(inv.summary())
    for p in inv.restock_needed():
        print(f"  restock: {p.sku} {p.name} (on hand {inv.quantity(p.sku)}, reorder at {p.reorder_point})")
