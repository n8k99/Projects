"""Transaction Averages — grouped aggregation over financial records.

Integer cents throughout; floats only for mean. Group-by: hash accumulate,
sort keys before emit. Determinism via sorted output keys.
"""

from __future__ import annotations
from dataclasses import dataclass
from typing import Callable


@dataclass
class Transaction:
    date: str        # YYYY-MM-DD
    category: str
    amount_cents: int

    def month(self) -> str:
        return self.date[:7] if len(self.date) >= 7 else self.date


@dataclass
class Stats:
    count: int = 0
    sum_cents: int = 0
    mean_cents: float = 0.0
    min_cents: int = 0
    max_cents: int = 0

    @classmethod
    def new(cls) -> Stats:
        return cls(min_cents=10**18, max_cents=-10**18)

    def observe(self, amount: int) -> None:
        self.count += 1
        self.sum_cents += amount
        if amount < self.min_cents:
            self.min_cents = amount
        if amount > self.max_cents:
            self.max_cents = amount

    def finalise(self) -> None:
        self.mean_cents = 0.0 if self.count == 0 else self.sum_cents / self.count


def parse_transaction(line: str) -> Transaction | None:
    parts = line.split(",", 2)
    if len(parts) != 3:
        return None
    date, category, amount = parts[0].strip(), parts[1].strip(), parts[2].strip()
    if not date or not category:
        return None
    try:
        amt = int(amount)
    except ValueError:
        return None
    return Transaction(date, category, amt)


def parse_transactions(text: str) -> list[Transaction]:
    out = []
    for line in text.split("\n"):
        if line == "":
            continue
        tx = parse_transaction(line)
        if tx is not None:
            out.append(tx)
    return out


def group_by(txns: list[Transaction],
             key_fn: Callable[[Transaction], str]) -> list[tuple[str, Stats]]:
    groups: dict[str, Stats] = {}
    for t in txns:
        k = key_fn(t)
        if k not in groups:
            groups[k] = Stats.new()
        groups[k].observe(t.amount_cents)
    out = []
    for k in sorted(groups.keys()):
        s = groups[k]
        s.finalise()
        out.append((k, s))
    return out


def by_category(txns: list[Transaction]) -> list[tuple[str, Stats]]:
    return group_by(txns, lambda t: t.category)


def by_month(txns: list[Transaction]) -> list[tuple[str, Stats]]:
    return group_by(txns, lambda t: t.month())


def total(txns: list[Transaction]) -> Stats:
    s = Stats.new()
    for t in txns:
        s.observe(t.amount_cents)
    s.finalise()
    return s


def top_n_by_mean(groups: list[tuple[str, Stats]], n: int) -> list[tuple[str, Stats]]:
    return sorted(groups, key=lambda kv: -kv[1].mean_cents)[:n]


def demo() -> None:
    sample = (
        "2026-01-03,groceries,4500\n"
        "2026-01-10,groceries,3200\n"
        "2026-01-12,rent,120000\n"
        "2026-02-01,rent,120000\n"
        "2026-02-14,groceries,5500\n"
        "2026-02-22,books,2200\n"
    )
    txns = parse_transactions(sample)
    print(f"=== {len(txns)} transactions ===")
    print("\nby category:")
    for k, s in by_category(txns):
        print(f"  {k:10} count={s.count} sum={s.sum_cents} mean={s.mean_cents:.2f} "
              f"min={s.min_cents} max={s.max_cents}")
    print("\nby month:")
    for k, s in by_month(txns):
        print(f"  {k}  count={s.count}  sum={s.sum_cents}  mean={s.mean_cents:.2f}")
    print("\ntop 2 by mean:")
    for k, s in top_n_by_mean(by_category(txns), 2):
        print(f"  {k}  mean={s.mean_cents:.2f}")


if __name__ == "__main__":
    demo()
