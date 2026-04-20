"""Web Board / Forum — threaded posts with vote tallies + hot-score ranking.

Parent-pointer storage (Post.parent_id), tree derived on read via DFS.
Hot score: sign(s) * log10(max(1, |s|)) - age_days * 0.2.
Sort threads by NEW, TOP, HOT; pinned always first.
"""

from __future__ import annotations
from dataclasses import dataclass, field
from enum import Enum
import math


class ThreadSort(Enum):
    NEW = "new"
    TOP = "top"
    HOT = "hot"


@dataclass
class Thread:
    id: int
    title: str
    author: str
    created_ms: int
    pinned: bool = False
    locked: bool = False


@dataclass
class Post:
    id: int
    thread_id: int
    parent_id: int | None
    author: str
    body: str
    created_ms: int
    upvotes: int = 0
    downvotes: int = 0

    def score(self) -> int:
        return self.upvotes - self.downvotes


def hot_score(post: Post, now_ms: int) -> float:
    s = post.score()
    sign = 1.0 if s > 0 else (-1.0 if s < 0 else 0.0)
    magnitude = math.log10(max(1, abs(s)))
    age_days = max(0, now_ms - post.created_ms) / 86_400_000.0
    return sign * magnitude - age_days * 0.2


@dataclass
class PostWithDepth:
    post: Post
    depth: int


@dataclass
class Board:
    threads: dict[int, Thread] = field(default_factory=dict)
    posts: dict[int, Post] = field(default_factory=dict)

    def add_thread(self, t: Thread) -> None:
        self.threads[t.id] = t

    def add_post(self, p: Post) -> None:
        if p.thread_id not in self.threads:
            raise ValueError(f"unknown thread {p.thread_id}")
        if p.parent_id is not None:
            parent = self.posts.get(p.parent_id)
            if parent is None:
                raise ValueError(f"unknown parent {p.parent_id}")
            if parent.thread_id != p.thread_id:
                raise ValueError(f"parent {p.parent_id} is in a different thread")
        self.posts[p.id] = p

    def top_level_posts(self, thread_id: int) -> list[Post]:
        return sorted(
            [p for p in self.posts.values()
             if p.thread_id == thread_id and p.parent_id is None],
            key=lambda p: p.created_ms,
        )

    def children_of(self, post_id: int) -> list[Post]:
        return sorted(
            [p for p in self.posts.values() if p.parent_id == post_id],
            key=lambda p: p.created_ms,
        )

    def thread_tree(self, thread_id: int) -> list[PostWithDepth]:
        out: list[PostWithDepth] = []
        for top in self.top_level_posts(thread_id):
            self._walk(top, 0, out)
        return out

    def _walk(self, post: Post, depth: int, out: list[PostWithDepth]) -> None:
        out.append(PostWithDepth(post, depth))
        for child in self.children_of(post.id):
            self._walk(child, depth + 1, out)

    def thread_score(self, thread_id: int) -> int:
        return sum(p.score() for p in self.posts.values()
                   if p.thread_id == thread_id)

    def thread_hot_score(self, thread_id: int, now_ms: int) -> float:
        return sum(hot_score(p, now_ms) for p in self.posts.values()
                   if p.thread_id == thread_id)

    def list_threads(self, sort: ThreadSort, now_ms: int) -> list[Thread]:
        def sort_key(t: Thread):
            pin = 0 if t.pinned else 1  # pinned first
            if sort == ThreadSort.NEW:
                return (pin, -t.created_ms)
            if sort == ThreadSort.TOP:
                return (pin, -self.thread_score(t.id))
            # HOT
            return (pin, -self.thread_hot_score(t.id, now_ms))
        return sorted(self.threads.values(), key=sort_key)

    def post_count(self, thread_id: int) -> int:
        return sum(1 for p in self.posts.values() if p.thread_id == thread_id)


def demo() -> None:
    board = Board()
    board.add_thread(Thread(1, "Welcome", "alice", 1_000_000, pinned=True))
    board.add_thread(Thread(2, "General discussion", "bob", 2_000_000))

    board.add_post(Post(10, 1, None, "alice", "hi everyone", 1_000_000, 10, 1))
    board.add_post(Post(11, 1, 10, "bob", "hi alice", 1_100_000, 5, 0))
    board.add_post(Post(12, 1, 10, "carol", "hello", 1_200_000, 3, 0))
    board.add_post(Post(13, 1, 11, "alice", "hey bob", 1_300_000, 2, 0))

    board.add_post(Post(20, 2, None, "bob", "first!", 2_000_000, 100, 5))

    print("=== thread 1 tree ===")
    for item in board.thread_tree(1):
        indent = "  " * item.depth
        print(f"  {indent}#{item.post.id} by {item.post.author} (score={item.post.score()})")

    print("\n=== threads sorted TOP ===")
    for t in board.list_threads(ThreadSort.TOP, 3_000_000):
        score = board.thread_score(t.id)
        pin = " [pinned]" if t.pinned else ""
        print(f"  {t.id}: {t.title}{pin} (score={score})")


if __name__ == "__main__":
    demo()
