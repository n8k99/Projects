"""Chat Application (Networking) — multi-room chat with users and history."""

from dataclasses import dataclass, field
from datetime import datetime, timezone
from typing import Optional


@dataclass
class Message:
    """A single chat message with sender, room, text, and timestamp."""
    sender: str
    room: str
    text: str
    timestamp: datetime = field(default_factory=lambda: datetime.now(timezone.utc))

    def __repr__(self) -> str:
        ts = self.timestamp.strftime("%H:%M:%S")
        return f"[{ts}] {self.sender}@{self.room}: {self.text}"


class ChatRoom:
    """A chat room that holds messages and tracks members."""

    def __init__(self, name: str) -> None:
        self.name = name
        self.members: set[str] = set()
        self.messages: list[Message] = []

    def add_member(self, username: str) -> None:
        self.members.add(username)

    def remove_member(self, username: str) -> None:
        self.members.discard(username)

    def post_message(self, msg: Message) -> None:
        self.messages.append(msg)

    def get_messages(self, since: Optional[datetime] = None) -> list[Message]:
        if since is None:
            return list(self.messages)
        return [m for m in self.messages if m.timestamp >= since]

    def __repr__(self) -> str:
        return f"ChatRoom({self.name!r}, members={len(self.members)}, messages={len(self.messages)})"


class ChatUser:
    """A chat user who can join rooms, send messages, and receive messages."""

    def __init__(self, name: str) -> None:
        self.name = name
        self.rooms: set[str] = set()
        self.inbox: list[Message] = []

    def receive(self, msg: Message) -> None:
        """Receive a message (called by the server on broadcast)."""
        self.inbox.append(msg)

    def __repr__(self) -> str:
        return f"ChatUser({self.name!r}, rooms={self.rooms})"


class ChatServer:
    """Manages rooms, users, and message routing."""

    def __init__(self) -> None:
        self.rooms: dict[str, ChatRoom] = {}
        self.users: dict[str, ChatUser] = {}

    def register_user(self, name: str) -> ChatUser:
        """Register a new user on the server."""
        if name in self.users:
            raise ValueError(f"User {name!r} already exists")
        user = ChatUser(name)
        self.users[name] = user
        return user

    def create_room(self, name: str) -> ChatRoom:
        """Create a new chat room."""
        if name in self.rooms:
            raise ValueError(f"Room {name!r} already exists")
        room = ChatRoom(name)
        self.rooms[name] = room
        return room

    def join_room(self, user: ChatUser, room_name: str) -> None:
        """Add a user to a room."""
        if room_name not in self.rooms:
            raise ValueError(f"Room {room_name!r} does not exist")
        self.rooms[room_name].add_member(user.name)
        user.rooms.add(room_name)

    def leave_room(self, user: ChatUser, room_name: str) -> None:
        """Remove a user from a room."""
        if room_name not in self.rooms:
            raise ValueError(f"Room {room_name!r} does not exist")
        self.rooms[room_name].remove_member(user.name)
        user.rooms.discard(room_name)

    def send_message(self, user: ChatUser, room_name: str, text: str) -> Message:
        """Send a message from a user to a room, broadcasting to all members."""
        if room_name not in self.rooms:
            raise ValueError(f"Room {room_name!r} does not exist")
        room = self.rooms[room_name]
        if user.name not in room.members:
            raise ValueError(f"User {user.name!r} is not in room {room_name!r}")
        msg = Message(sender=user.name, room=room_name, text=text)
        room.post_message(msg)
        # Broadcast to all members except the sender
        for member_name in room.members:
            if member_name != user.name:
                if member_name in self.users:
                    self.users[member_name].receive(msg)
        return msg

    def get_messages(self, room_name: str, since: Optional[datetime] = None) -> list[Message]:
        """Get message history for a room, optionally since a timestamp."""
        if room_name not in self.rooms:
            raise ValueError(f"Room {room_name!r} does not exist")
        return self.rooms[room_name].get_messages(since)

    def list_rooms(self) -> list[str]:
        """List all room names."""
        return list(self.rooms.keys())

    def list_users(self, room_name: str) -> list[str]:
        """List all users in a room."""
        if room_name not in self.rooms:
            raise ValueError(f"Room {room_name!r} does not exist")
        return sorted(self.rooms[room_name].members)


if __name__ == "__main__":
    server = ChatServer()

    # Register users
    alice = server.register_user("alice")
    bob = server.register_user("bob")
    carol = server.register_user("carol")

    # Create rooms
    server.create_room("general")
    server.create_room("engineering")

    # Users join rooms
    server.join_room(alice, "general")
    server.join_room(bob, "general")
    server.join_room(carol, "general")
    server.join_room(alice, "engineering")
    server.join_room(bob, "engineering")

    # Exchange messages
    server.send_message(alice, "general", "Hello everyone!")
    server.send_message(bob, "general", "Hey Alice, welcome!")
    server.send_message(carol, "general", "Good to see you both.")

    server.send_message(alice, "engineering", "Let's discuss the new API.")
    server.send_message(bob, "engineering", "I've been looking at the schema.")

    # Show history
    print("=== Rooms ===")
    for room_name in server.list_rooms():
        members = server.list_users(room_name)
        print(f"  #{room_name}: {', '.join(members)}")

    print("\n=== #general history ===")
    for msg in server.get_messages("general"):
        print(f"  {msg}")

    print("\n=== #engineering history ===")
    for msg in server.get_messages("engineering"):
        print(f"  {msg}")

    print("\n=== Bob's inbox ===")
    for msg in bob.inbox:
        print(f"  {msg}")

    # Carol leaves general
    server.leave_room(carol, "general")
    print(f"\n=== After Carol leaves #general ===")
    print(f"  #general members: {server.list_users('general')}")
