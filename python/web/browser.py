"""Web Browser with Tabs — state model for tabbed navigation.

Each tab is an independent history timeline — its own back stack, forward
stack, and current URL. The browser holds tabs, an active-tab pointer, a
closed-tab stack (for Ctrl+Shift+T reopen), and browser-level state
(bookmarks) that transcends individual tabs.

Two orthogonal undo-like stacks: per-tab navigation history (back/forward)
and browser-level closed-tab recovery. Different scopes, different answers.
"""

from __future__ import annotations
from collections import deque
from dataclasses import dataclass, field


TabId = int


@dataclass
class Tab:
    id: TabId
    title: str
    url: str
    back: list[str] = field(default_factory=list)
    forward: list[str] = field(default_factory=list)


@dataclass
class _ClosedTab:
    url: str
    title: str
    back: list[str]
    forward: list[str]


class Browser:
    def __init__(self) -> None:
        self._tabs: list[Tab] = []
        self._active: TabId | None = None
        self._closed: deque[_ClosedTab] = deque()
        self._bookmarks: list[tuple[str, str]] = []
        self._next_id = 1

    def open_tab(self, url: str) -> TabId:
        tid = self._next_id
        self._next_id += 1
        self._tabs.append(Tab(id=tid, title=url, url=url))
        self._active = tid
        return tid

    def close_tab(self, tid: TabId) -> None:
        pos = next((i for i, t in enumerate(self._tabs) if t.id == tid), None)
        if pos is None:
            return
        t = self._tabs.pop(pos)
        self._closed.appendleft(_ClosedTab(
            url=t.url, title=t.title, back=t.back, forward=t.forward))
        if self._active == tid:
            if pos < len(self._tabs):
                self._active = self._tabs[pos].id
            elif self._tabs:
                self._active = self._tabs[-1].id
            else:
                self._active = None

    def switch_to(self, tid: TabId) -> bool:
        if any(t.id == tid for t in self._tabs):
            self._active = tid
            return True
        return False

    def active_tab(self) -> Tab | None:
        if self._active is None:
            return None
        return next((t for t in self._tabs if t.id == self._active), None)

    def navigate(self, url: str) -> bool:
        t = self.active_tab()
        if t is None:
            return False
        t.back.append(t.url)
        t.forward.clear()
        t.url = url
        t.title = url
        return True

    def back(self) -> str | None:
        t = self.active_tab()
        if t is None or not t.back:
            return None
        prev = t.back.pop()
        t.forward.append(t.url)
        t.url = prev
        return prev

    def forward(self) -> str | None:
        t = self.active_tab()
        if t is None or not t.forward:
            return None
        nxt = t.forward.pop()
        t.back.append(t.url)
        t.url = nxt
        return nxt

    def reopen_closed(self) -> TabId | None:
        if not self._closed:
            return None
        ct = self._closed.popleft()
        tid = self._next_id
        self._next_id += 1
        self._tabs.append(Tab(id=tid, title=ct.title, url=ct.url,
                              back=ct.back, forward=ct.forward))
        self._active = tid
        return tid

    def add_bookmark(self) -> bool:
        t = self.active_tab()
        if t is None:
            return False
        entry = (t.title, t.url)
        if entry not in self._bookmarks:
            self._bookmarks.append(entry)
        return True

    def bookmarks(self) -> list[tuple[str, str]]:
        return list(self._bookmarks)

    def tab_count(self) -> int: return len(self._tabs)
    def closed_count(self) -> int: return len(self._closed)
    def tabs(self) -> list[Tab]: return list(self._tabs)


if __name__ == "__main__":
    b = Browser()
    t1 = b.open_tab("https://example.org")
    b.navigate("https://example.org/about")
    b.navigate("https://example.org/contact")
    t2 = b.open_tab("https://news.example.com")
    b.navigate("https://news.example.com/story/1")
    b.add_bookmark()
    b.switch_to(t1)
    b.back()
    print(f"t1 back to {b.active_tab().url}")
    print(f"t1 forward stack: {b.active_tab().forward}")
    b.close_tab(t2)
    print(f"closed tabs: {b.closed_count()}, bookmarks survive: {b.bookmarks()}")
    reopened = b.reopen_closed()
    print(f"reopened tab {reopened}: {b.active_tab().url}")
