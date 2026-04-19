"""Chat Application — bidirectional broadcast across N concurrent actors.

Every actor is both a producer (sending messages) and a consumer (reading
its inbox). One send from Alice fans out to every joined user's inbox —
broadcast, not point-to-point.

The room holds the user list and the full chat log under a single lock.
Send assigns a monotonically-increasing sequence number under that lock
before pushing into every inbox, so all recipients see messages in the
same total order. This is the standard trick for getting total ordering
out of an otherwise-interleaved concurrent system: a single global
counter incremented under a single lock.

This is the first Rosetta Stone project where every actor is both
producer and consumer; previous threading projects had a strict
producer-consumer or one-producer / many-workers split.
"""

from __future__ import annotations
import threading
from collections import deque
from dataclasses import dataclass


UserId = int


@dataclass(frozen=True)
class Message:
    seq: int
    from_id: UserId
    from_name: str
    text: str


class ChatRoom:
    def __init__(self) -> None:
        self._lock = threading.Lock()
        self._users: dict[UserId, _Inbox] = {}
        self._log: list[Message] = []
        self._next_id = 1
        self._next_seq = 1

    def join(self, name: str) -> UserId:
        with self._lock:
            uid = self._next_id
            self._next_id += 1
            self._users[uid] = _Inbox(name=name)
            return uid

    def leave(self, uid: UserId) -> None:
        with self._lock:
            self._users.pop(uid, None)

    def send(self, sender: UserId, text: str) -> bool:
        """Broadcast `text` from `sender` to every joined inbox (including sender's).
        Returns False if sender is not a member."""
        with self._lock:
            inbox = self._users.get(sender)
            if inbox is None:
                return False
            seq = self._next_seq
            self._next_seq += 1
            msg = Message(seq=seq, from_id=sender, from_name=inbox.name, text=text)
            self._log.append(msg)
            for target in self._users.values():
                target.messages.append(msg)
            return True

    def read(self, uid: UserId) -> list[Message]:
        """Drain and return this user's current inbox."""
        with self._lock:
            inbox = self._users.get(uid)
            if inbox is None:
                return []
            out = list(inbox.messages)
            inbox.messages.clear()
            return out

    def history(self) -> list[Message]:
        with self._lock:
            return list(self._log)

    def user_count(self) -> int:
        with self._lock:
            return len(self._users)


@dataclass
class _Inbox:
    name: str
    messages: deque[Message] = None  # type: ignore[assignment]

    def __post_init__(self) -> None:
        if self.messages is None:
            self.messages = deque()


if __name__ == "__main__":
    import random
    import time

    room = ChatRoom()
    alice = room.join("Alice")
    bob = room.join("Bob")
    carol = room.join("Carol")
    greetings = ["hello", "hi there", "what's up", "hey", "good morning"]

    def chatter(uid: UserId, name: str, count: int) -> None:
        for i in range(count):
            room.send(uid, f"{random.choice(greetings)} ({i})")
            time.sleep(random.uniform(0.001, 0.005))

    threads = [
        threading.Thread(target=chatter, args=(alice, "Alice", 20)),
        threading.Thread(target=chatter, args=(bob, "Bob", 20)),
        threading.Thread(target=chatter, args=(carol, "Carol", 20)),
    ]
    for t in threads: t.start()
    for t in threads: t.join()

    log = room.history()
    print(f"{len(log)} messages exchanged (expected 60)")
    print(f"final user count: {room.user_count()}")
    # Verify total ordering.
    seqs = [m.seq for m in log]
    print(f"sequence order strictly increasing: {seqs == sorted(seqs)}")
    # Verify every receiver saw exactly 60 messages (each user has unread = 60
    # because nobody read during the conversation).
    for (uid, name) in [(alice, "Alice"), (bob, "Bob"), (carol, "Carol")]:
        msgs = room.read(uid)
        print(f"{name}'s inbox: {len(msgs)} messages")
