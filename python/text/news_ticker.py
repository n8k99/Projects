"""News Ticker and Game Scores — real-time prioritized message stream."""

from dataclasses import dataclass, field
from datetime import datetime, timedelta
from typing import Optional


@dataclass
class TickerItem:
    text: str
    category: str  # news, sports, finance, alert
    priority: int  # 1-5, 5 = highest
    timestamp: datetime = field(default_factory=datetime.now)
    expires_at: Optional[datetime] = None

    def is_expired(self) -> bool:
        if self.expires_at is None:
            return False
        return datetime.now() >= self.expires_at


class NewsTicker:
    CATEGORIES = {"news", "sports", "finance", "alert"}

    def __init__(self):
        self.items: list[TickerItem] = []

    def post(
        self,
        text: str,
        category: str,
        priority: int,
        ttl_seconds: Optional[int] = None,
    ) -> TickerItem:
        if category not in self.CATEGORIES:
            raise ValueError(f"Invalid category '{category}'. Must be one of {self.CATEGORIES}")
        if not 1 <= priority <= 5:
            raise ValueError(f"Priority must be 1-5, got {priority}")

        expires_at = None
        if ttl_seconds is not None:
            expires_at = datetime.now() + timedelta(seconds=ttl_seconds)

        item = TickerItem(
            text=text,
            category=category,
            priority=priority,
            expires_at=expires_at,
        )
        self.items.append(item)
        return item

    def breaking(self, text: str) -> TickerItem:
        return self.post(text, "alert", 5)

    def get_current(self) -> list[TickerItem]:
        active = [item for item in self.items if not item.is_expired()]
        return sorted(active, key=lambda i: (-i.priority, i.timestamp))

    def get_by_category(self, category: str) -> list[TickerItem]:
        return [
            item
            for item in self.get_current()
            if item.category == category
        ]

    def expire_old(self) -> int:
        before = len(self.items)
        self.items = [item for item in self.items if not item.is_expired()]
        return before - len(self.items)

    def display(self) -> str:
        current = self.get_current()
        if not current:
            return ">>> No items >>>"
        texts = [item.text for item in current]
        return ">>> " + " | ".join(texts) + " >>>"


if __name__ == "__main__":
    ticker = NewsTicker()

    # Post headlines
    ticker.post("Markets rally on strong earnings", "finance", 3)
    ticker.post("Lakers defeat Celtics 112-108", "sports", 2)
    ticker.post("New climate accord signed by 40 nations", "news", 4)
    ticker.post("Flash sale: tech stocks surge 5%", "finance", 2, ttl_seconds=1)

    print("=== Full Ticker ===")
    print(ticker.display())

    print("\n=== Sports Only ===")
    for item in ticker.get_by_category("sports"):
        print(f"  [{item.priority}] {item.text}")

    print("\n=== Finance Only ===")
    for item in ticker.get_by_category("finance"):
        print(f"  [{item.priority}] {item.text}")

    # Wait for expiry
    import time
    time.sleep(1.1)

    removed = ticker.expire_old()
    print(f"\n=== After expiry ({removed} removed) ===")
    print(ticker.display())

    # Breaking news
    ticker.breaking("EARTHQUAKE: 7.2 magnitude off Pacific coast")
    print("\n=== BREAKING NEWS ===")
    print(ticker.display())
