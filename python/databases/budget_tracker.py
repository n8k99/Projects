"""Budget Tracker — envelope allocation, variance analysis, alert thresholds.

Envelopes: per-category monthly allocation + alert threshold percent.
Transactions: negative = expense, positive = refund.
Variance: spent - budgeted; alerts fire at threshold_pct of budget.
"""

from __future__ import annotations
from dataclasses import dataclass, field
from enum import Enum


class RolloverPolicy(Enum):
    RESET = "reset"
    CARRY = "carry"


@dataclass
class Envelope:
    category: str
    budgeted_cents: int
    rollover: RolloverPolicy = RolloverPolicy.RESET
    alert_threshold_pct: int = 80


@dataclass
class Transaction:
    timestamp_ms: int
    category: str
    amount_cents: int  # negative = expense, positive = refund
    note: str = ""


@dataclass
class CategoryStatus:
    category: str
    budgeted_cents: int
    spent_cents: int
    remaining_cents: int
    variance_cents: int
    variance_pct: int  # -101 sentinel for no budget
    over_budget: bool
    alert: bool


@dataclass
class Budget:
    envelopes: dict[str, Envelope] = field(default_factory=dict)
    transactions: list[Transaction] = field(default_factory=list)

    def add_envelope(self, e: Envelope) -> None:
        self.envelopes[e.category] = e

    def record(self, t: Transaction) -> None:
        self.transactions.append(t)

    def total_budgeted(self) -> int:
        return sum(e.budgeted_cents for e in self.envelopes.values())

    def total_spent_between(self, from_ms: int, to_ms: int) -> int:
        total = 0
        for t in self.transactions:
            if t.timestamp_ms < from_ms or t.timestamp_ms >= to_ms:
                continue
            total += -t.amount_cents if t.amount_cents < 0 else -t.amount_cents
        return total

    def status_between(self, from_ms: int, to_ms: int) -> list[CategoryStatus]:
        spent: dict[str, int] = {}
        for t in self.transactions:
            if t.timestamp_ms < from_ms or t.timestamp_ms >= to_ms:
                continue
            delta = -t.amount_cents if t.amount_cents < 0 else -t.amount_cents
            spent[t.category] = spent.get(t.category, 0) + delta

        out: list[CategoryStatus] = []
        for env in self.envelopes.values():
            spent_cents = spent.get(env.category, 0)
            remaining = env.budgeted_cents - spent_cents
            variance = spent_cents - env.budgeted_cents
            if env.budgeted_cents == 0:
                variance_pct = -101
            else:
                variance_pct = variance * 100 // env.budgeted_cents
            over = spent_cents > env.budgeted_cents
            alert = False
            if env.budgeted_cents > 0:
                pct = max(0, spent_cents * 100 // env.budgeted_cents)
                alert = pct >= env.alert_threshold_pct
            out.append(CategoryStatus(
                category=env.category, budgeted_cents=env.budgeted_cents,
                spent_cents=spent_cents, remaining_cents=remaining,
                variance_cents=variance, variance_pct=variance_pct,
                over_budget=over, alert=alert,
            ))
        # Uncategorised spending → synthetic envelope.
        for cat, sp in spent.items():
            if cat not in self.envelopes:
                out.append(CategoryStatus(
                    category=cat, budgeted_cents=0, spent_cents=sp,
                    remaining_cents=-sp, variance_cents=sp,
                    variance_pct=-101, over_budget=sp > 0, alert=False,
                ))
        out.sort(key=lambda s: s.category)
        return out

    def status_all(self) -> list[CategoryStatus]:
        return self.status_between(-10**18, 10**18)

    def alerts(self) -> list[CategoryStatus]:
        return [s for s in self.status_all() if s.alert]


def demo() -> None:
    b = Budget()
    b.add_envelope(Envelope("groceries", 50000, alert_threshold_pct=80))
    b.add_envelope(Envelope("rent", 200000, alert_threshold_pct=90))
    b.add_envelope(Envelope("entertainment", 10000, alert_threshold_pct=50))
    b.record(Transaction(100, "groceries", -30000, "Store A"))
    b.record(Transaction(200, "groceries", -15000, "Store B"))
    b.record(Transaction(300, "rent", -200000, "April"))
    b.record(Transaction(400, "entertainment", -8000, "Concert"))
    b.record(Transaction(500, "vacation", -5000, "Weekend getaway"))  # uncategorised

    print("=== status ===")
    for s in b.status_all():
        bar = "OVER" if s.over_budget else "OK  "
        alert = " ⚠️" if s.alert else ""
        print(f"  {bar} {s.category:14} spent=${s.spent_cents/100:7.2f} / "
              f"budget=${s.budgeted_cents/100:7.2f}  variance={s.variance_pct:+4d}%{alert}")

    print("\n=== alerts ===")
    for s in b.alerts():
        print(f"  {s.category}: {s.spent_cents / s.budgeted_cents * 100:.1f}% spent")


if __name__ == "__main__":
    demo()
