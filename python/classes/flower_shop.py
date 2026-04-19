"""Flower Shop Ordering — build-up/commit transaction with draft state and reversal.

The cart is the Rosetta Stone's first INTENTIONALLY MUTABLE primary entity.
Prior projects had immutable value classes or entities mutated only through
atomic commits. A cart is *designed* to evolve: add flowers, remove, change
quantities, apply discounts. It commits once — at checkout — and no sooner.

Prices are frozen at add-time: the line carries the price it had when added,
not whatever the flower costs at checkout. This invariant is what makes
carts predictable across time.

Money is integer cents throughout (same as G052).
"""

from __future__ import annotations
from dataclasses import dataclass, field
from datetime import datetime, timezone
from enum import Enum
from typing import Optional
import itertools


class OrderStatus(Enum):
    DRAFT = "draft"        # cart
    PLACED = "placed"      # committed
    FULFILLED = "fulfilled"
    CANCELLED = "cancelled"


@dataclass
class Flower:
    sku: str
    name: str
    unit_price_cents: int
    stock: int = 0


@dataclass
class CartLine:
    """One line in a cart. Price is FROZEN at add time."""
    sku: str
    quantity: int
    unit_price_cents: int   # snapshot

    @property
    def line_total_cents(self) -> int:
        return self.quantity * self.unit_price_cents


# --- discount rules (closed polymorphism: enum of kinds) ---

class DiscountKind(Enum):
    PERCENTAGE_OFF = "percentage_off"
    FIXED_OFF      = "fixed_off"
    FREE_SHIPPING  = "free_shipping"


@dataclass
class DiscountRule:
    kind: DiscountKind
    # PERCENTAGE_OFF: value is percent (0-100); FIXED_OFF: value is cents;
    # FREE_SHIPPING: value is the threshold cents above which shipping is free.
    value: int
    # Optional minimum subtotal for the rule to apply.
    min_subtotal_cents: int = 0
    note: str = ""

    def applies(self, subtotal_cents: int) -> bool:
        return subtotal_cents >= self.min_subtotal_cents

    def discount_cents(self, subtotal_cents: int, shipping_cents: int) -> int:
        if not self.applies(subtotal_cents):
            return 0
        if self.kind == DiscountKind.PERCENTAGE_OFF:
            return subtotal_cents * self.value // 100
        if self.kind == DiscountKind.FIXED_OFF:
            return min(self.value, subtotal_cents)
        if self.kind == DiscountKind.FREE_SHIPPING:
            return shipping_cents if subtotal_cents >= self.value else 0
        return 0


@dataclass
class Cart:
    """A draft order — mutable until checkout."""
    id: str
    owner_id: str
    lines: list[CartLine] = field(default_factory=list)
    discount_rules: list[DiscountRule] = field(default_factory=list)
    shipping_cents: int = 0
    tax_rate_bps: int = 0   # basis points (1/100 of a percent); 825 = 8.25%
    status: OrderStatus = OrderStatus.DRAFT


@dataclass
class Order:
    """A committed cart. Carries enough state to be cancelled and reversed."""
    id: int
    owner_id: str
    lines: list[CartLine]
    discount_rules: list[DiscountRule]
    shipping_cents: int
    tax_rate_bps: int
    placed_at: datetime
    status: OrderStatus = OrderStatus.PLACED


# --- errors ---

class ShopError(Exception): ...
class UnknownEntity(ShopError): ...
class OutOfStock(ShopError): ...
class BadTransition(ShopError): ...


class Shop:
    """Flowers + carts + orders. Coordinates the build-up/commit lifecycle."""

    def __init__(self) -> None:
        self.flowers: dict[str, Flower] = {}
        self.carts: dict[str, Cart] = {}
        self.orders: dict[int, Order] = {}
        self._order_ids = itertools.count(1)

    # --- catalog ---
    def add_flower(self, flower: Flower) -> None:
        if flower.sku in self.flowers:
            raise ValueError(f"flower already registered: {flower.sku}")
        self.flowers[flower.sku] = flower

    def stock(self, sku: str, quantity: int) -> None:
        if sku not in self.flowers:
            raise UnknownEntity(f"unknown flower: {sku}")
        if quantity < 0:
            raise ValueError("stock quantity must be non-negative")
        self.flowers[sku].stock += quantity

    # --- carts (mutable) ---
    def new_cart(self, cart_id: str, owner_id: str,
                 shipping_cents: int = 0, tax_rate_bps: int = 0) -> Cart:
        if cart_id in self.carts:
            raise ValueError(f"cart already exists: {cart_id}")
        cart = Cart(id=cart_id, owner_id=owner_id,
                    shipping_cents=shipping_cents, tax_rate_bps=tax_rate_bps)
        self.carts[cart_id] = cart
        return cart

    def _require_cart(self, cart_id: str) -> Cart:
        if cart_id not in self.carts:
            raise UnknownEntity(f"unknown cart: {cart_id}")
        cart = self.carts[cart_id]
        if cart.status != OrderStatus.DRAFT:
            raise BadTransition(f"cart {cart_id} is {cart.status.value}, not draft")
        return cart

    def add_to_cart(self, cart_id: str, sku: str, quantity: int) -> Cart:
        cart = self._require_cart(cart_id)
        if sku not in self.flowers:
            raise UnknownEntity(f"unknown flower: {sku}")
        if quantity <= 0:
            raise ValueError("quantity must be positive")
        flower = self.flowers[sku]
        already_in_cart = sum(l.quantity for l in cart.lines if l.sku == sku)
        if flower.stock < already_in_cart + quantity:
            raise OutOfStock(
                f"{sku}: stock {flower.stock}, "
                f"already in cart {already_in_cart}, requested {quantity}")
        # Merge with an existing line for the same SKU (preserving the earlier price).
        for line in cart.lines:
            if line.sku == sku:
                line.quantity += quantity
                return cart
        cart.lines.append(CartLine(
            sku=sku, quantity=quantity,
            unit_price_cents=flower.unit_price_cents,   # FROZEN
        ))
        return cart

    def update_quantity(self, cart_id: str, sku: str, new_quantity: int) -> Cart:
        cart = self._require_cart(cart_id)
        if new_quantity < 0:
            raise ValueError("quantity must be non-negative")
        if sku not in self.flowers:
            raise UnknownEntity(f"unknown flower: {sku}")
        for i, line in enumerate(cart.lines):
            if line.sku == sku:
                if new_quantity == 0:
                    cart.lines.pop(i)
                else:
                    if self.flowers[sku].stock < new_quantity:
                        raise OutOfStock(
                            f"{sku}: stock {self.flowers[sku].stock}, "
                            f"requested {new_quantity}")
                    line.quantity = new_quantity
                return cart
        raise UnknownEntity(f"{sku} is not in cart {cart_id}")

    def remove_from_cart(self, cart_id: str, sku: str) -> Cart:
        return self.update_quantity(cart_id, sku, 0)

    def apply_discount(self, cart_id: str, rule: DiscountRule) -> Cart:
        cart = self._require_cart(cart_id)
        cart.discount_rules.append(rule)
        return cart

    # --- totals ---
    def subtotal_cents(self, cart_or_order: Cart | Order) -> int:
        return sum(l.line_total_cents for l in cart_or_order.lines)

    def total_discount_cents(self, cart_or_order: Cart | Order) -> int:
        sub = self.subtotal_cents(cart_or_order)
        return sum(r.discount_cents(sub, cart_or_order.shipping_cents)
                   for r in cart_or_order.discount_rules)

    def tax_cents(self, cart_or_order: Cart | Order) -> int:
        taxable = max(0, self.subtotal_cents(cart_or_order)
                     - self.total_discount_cents(cart_or_order))
        return taxable * cart_or_order.tax_rate_bps // 10_000

    def total_cents(self, cart_or_order: Cart | Order) -> int:
        return max(0, self.subtotal_cents(cart_or_order)
                   - self.total_discount_cents(cart_or_order)
                   + self.tax_cents(cart_or_order)
                   + cart_or_order.shipping_cents)

    # --- checkout (atomic commit) ---
    def checkout(self, cart_id: str, now: Optional[datetime] = None) -> Order:
        cart = self._require_cart(cart_id)
        if not cart.lines:
            raise BadTransition(f"cart {cart_id} is empty")
        # Pre-flight: verify every line against current stock atomically.
        shortages = []
        for line in cart.lines:
            if line.sku not in self.flowers:
                shortages.append((line.sku, "not_found"))
                continue
            if self.flowers[line.sku].stock < line.quantity:
                shortages.append((line.sku, "insufficient"))
        if shortages:
            raise OutOfStock(f"cannot checkout {cart_id}: shortages {shortages}")
        # Commit: decrement all stocks, create Order.
        for line in cart.lines:
            self.flowers[line.sku].stock -= line.quantity
        order = Order(
            id=next(self._order_ids),
            owner_id=cart.owner_id,
            lines=[CartLine(l.sku, l.quantity, l.unit_price_cents) for l in cart.lines],
            discount_rules=list(cart.discount_rules),
            shipping_cents=cart.shipping_cents,
            tax_rate_bps=cart.tax_rate_bps,
            placed_at=now or datetime.now(timezone.utc),
        )
        self.orders[order.id] = order
        cart.status = OrderStatus.PLACED
        return order

    def cancel_order(self, order_id: int) -> Order:
        """Reverse a placed order: restore inventory, mark cancelled."""
        if order_id not in self.orders:
            raise UnknownEntity(f"unknown order: {order_id}")
        order = self.orders[order_id]
        if order.status != OrderStatus.PLACED:
            raise BadTransition(
                f"order {order_id} is {order.status.value}, only PLACED can cancel")
        # Reverse the atomic decrement.
        for line in order.lines:
            if line.sku in self.flowers:
                self.flowers[line.sku].stock += line.quantity
        order.status = OrderStatus.CANCELLED
        return order

    def fulfill_order(self, order_id: int) -> Order:
        if order_id not in self.orders:
            raise UnknownEntity(f"unknown order: {order_id}")
        order = self.orders[order_id]
        if order.status != OrderStatus.PLACED:
            raise BadTransition(
                f"order {order_id} is {order.status.value}, only PLACED can fulfill")
        order.status = OrderStatus.FULFILLED
        return order

    # --- receipt ---
    def receipt(self, order: Order) -> str:
        lines = [f"ORDER #{order.id}  ({order.status.value})",
                 f"placed: {order.placed_at.strftime('%Y-%m-%d %H:%M')}  owner: {order.owner_id}"]
        lines.append("-" * 52)
        for l in order.lines:
            name = self.flowers.get(l.sku, Flower(l.sku, l.sku, 0)).name
            lines.append(f"  {l.quantity:3d} × {name:<20s}  "
                         f"@ ${l.unit_price_cents/100:>6.2f} = "
                         f"${l.line_total_cents/100:>8.2f}")
        sub  = self.subtotal_cents(order)
        disc = self.total_discount_cents(order)
        tax  = self.tax_cents(order)
        ship = order.shipping_cents
        tot  = self.total_cents(order)
        lines.append("-" * 52)
        lines.append(f"  {'Subtotal':<38s}  ${sub/100:>8.2f}")
        if disc > 0:
            lines.append(f"  {'Discounts':<38s} -${disc/100:>8.2f}")
        if tax > 0:
            lines.append(f"  {'Tax':<38s}  ${tax/100:>8.2f}")
        if ship > 0:
            lines.append(f"  {'Shipping':<38s}  ${ship/100:>8.2f}")
        lines.append(f"  {'TOTAL':<38s}  ${tot/100:>8.2f}")
        return "\n".join(lines)


# --- demo ---
if __name__ == "__main__":
    shop = Shop()
    shop.add_flower(Flower("ROSE",  "Red Rose",      unit_price_cents=450))
    shop.add_flower(Flower("TULIP", "Yellow Tulip",  unit_price_cents=300))
    shop.add_flower(Flower("LILY",  "White Lily",    unit_price_cents=625))
    shop.stock("ROSE", 50)
    shop.stock("TULIP", 30)
    shop.stock("LILY", 12)

    cart = shop.new_cart("CART-001", owner_id="P-001",
                         shipping_cents=500, tax_rate_bps=825)
    shop.add_to_cart(cart.id, "ROSE", 6)
    shop.add_to_cart(cart.id, "TULIP", 4)
    shop.add_to_cart(cart.id, "LILY", 2)

    # Price changes between add-to-cart and checkout don't affect the cart.
    shop.flowers["ROSE"].unit_price_cents = 600
    print("Price of ROSE changed to $6.00 after cart, but cart's line is unchanged:")
    print(f"  cart line price: ${[l for l in cart.lines if l.sku=='ROSE'][0].unit_price_cents/100:.2f}")

    shop.apply_discount(cart.id, DiscountRule(
        kind=DiscountKind.PERCENTAGE_OFF, value=10,
        min_subtotal_cents=3000, note="10% off orders over $30"))
    shop.apply_discount(cart.id, DiscountRule(
        kind=DiscountKind.FREE_SHIPPING, value=4000,
        note="free shipping on orders over $40"))

    order = shop.checkout(cart.id)
    print("\n" + shop.receipt(order))

    # Inventory reflects the atomic decrement.
    print(f"\nPost-checkout stock:")
    for sku in ("ROSE", "TULIP", "LILY"):
        print(f"  {sku}: {shop.flowers[sku].stock}")

    # Cancel the order; inventory restores.
    shop.cancel_order(order.id)
    print(f"\nAfter cancellation:")
    for sku in ("ROSE", "TULIP", "LILY"):
        print(f"  {sku}: {shop.flowers[sku].stock}")
