"""Mail Checker — monitor an inbox for new messages with filtering and notification."""

from dataclasses import dataclass, field
from datetime import datetime, timezone
from typing import Optional, Callable
import hashlib


@dataclass
class EmailAddress:
    """A parsed email address."""
    name: str = ""
    address: str = ""

    def __str__(self) -> str:
        if self.name:
            return f"{self.name} <{self.address}>"
        return self.address

    @staticmethod
    def parse(raw: str) -> "EmailAddress":
        raw = raw.strip()
        if "<" in raw and ">" in raw:
            name = raw[:raw.index("<")].strip().strip('"')
            addr = raw[raw.index("<") + 1:raw.index(">")].strip()
            return EmailAddress(name=name, address=addr)
        return EmailAddress(address=raw)


@dataclass
class MailMessage:
    """A single email message."""
    uid: str
    sender: EmailAddress
    recipients: list[EmailAddress]
    subject: str
    body: str
    date: datetime
    is_read: bool = False
    is_flagged: bool = False
    headers: dict[str, str] = field(default_factory=dict)

    @property
    def preview(self) -> str:
        body_preview = self.body[:80].replace("\n", " ")
        if len(self.body) > 80:
            body_preview += "..."
        return body_preview

    def matches_filter(self, f: "MailFilter") -> bool:
        if f.sender_contains and f.sender_contains.lower() not in self.sender.address.lower():
            return False
        if f.subject_contains and f.subject_contains.lower() not in self.subject.lower():
            return False
        if f.body_contains and f.body_contains.lower() not in self.body.lower():
            return False
        if f.is_unread_only and self.is_read:
            return False
        if f.since and self.date < f.since:
            return False
        return True

    def __repr__(self) -> str:
        flag = "★" if self.is_flagged else " "
        read = " " if self.is_read else "●"
        date_str = self.date.strftime("%Y-%m-%d %H:%M")
        return f"{read}{flag} {date_str} {self.sender.address:30s} {self.subject}"


@dataclass
class MailFilter:
    """Filter criteria for mail searches."""
    sender_contains: str = ""
    subject_contains: str = ""
    body_contains: str = ""
    is_unread_only: bool = False
    since: Optional[datetime] = None


@dataclass
class CheckResult:
    """Result of a mail check operation."""
    total_messages: int
    new_messages: int
    matched_messages: list[MailMessage]
    checked_at: datetime = field(default_factory=lambda: datetime.now(timezone.utc))

    def summary(self) -> str:
        lines = [
            f"Mail check at {self.checked_at.strftime('%Y-%m-%d %H:%M:%S UTC')}",
            f"  Total: {self.total_messages}, New: {self.new_messages}, Matched: {len(self.matched_messages)}",
        ]
        for msg in self.matched_messages[:10]:
            lines.append(f"  {msg}")
        if len(self.matched_messages) > 10:
            lines.append(f"  ... and {len(self.matched_messages) - 10} more")
        return "\n".join(lines)


class Mailbox:
    """An in-process mailbox that stores messages and supports check/filter operations.

    Models the data layer of a mail client — the same operations that IMAP/POP3
    provide over the network. Actual protocol integration is transport-level;
    this module is the callable library underneath.
    """

    def __init__(self, owner: str) -> None:
        self.owner = owner
        self.messages: dict[str, MailMessage] = {}
        self.last_check: Optional[datetime] = None
        self._seen_uids: set[str] = set()
        self._notifications: list[Callable[[MailMessage], None]] = []

    def deliver(self, msg: MailMessage) -> None:
        """Deliver a message to the mailbox (called by the mail server)."""
        self.messages[msg.uid] = msg
        if msg.uid not in self._seen_uids:
            for callback in self._notifications:
                callback(msg)

    def check(self, mail_filter: Optional[MailFilter] = None) -> CheckResult:
        """Check for messages, optionally filtered. Returns a CheckResult."""
        new_uids = set(self.messages.keys()) - self._seen_uids
        self._seen_uids = set(self.messages.keys())
        now = datetime.now(timezone.utc)

        all_msgs = sorted(self.messages.values(), key=lambda m: m.date, reverse=True)

        if mail_filter:
            matched = [m for m in all_msgs if m.matches_filter(mail_filter)]
        else:
            matched = all_msgs

        result = CheckResult(
            total_messages=len(self.messages),
            new_messages=len(new_uids),
            matched_messages=matched,
            checked_at=now,
        )
        self.last_check = now
        return result

    def mark_read(self, uid: str) -> bool:
        """Mark a message as read."""
        if uid in self.messages:
            self.messages[uid].is_read = True
            return True
        return False

    def mark_flagged(self, uid: str, flagged: bool = True) -> bool:
        """Flag or unflag a message."""
        if uid in self.messages:
            self.messages[uid].is_flagged = flagged
            return True
        return False

    def delete(self, uid: str) -> bool:
        """Delete a message from the mailbox."""
        if uid in self.messages:
            del self.messages[uid]
            self._seen_uids.discard(uid)
            return True
        return False

    def on_new_mail(self, callback: Callable[[MailMessage], None]) -> None:
        """Register a callback for new mail notifications."""
        self._notifications.append(callback)

    def unread_count(self) -> int:
        """Return the number of unread messages."""
        return sum(1 for m in self.messages.values() if not m.is_read)

    def stats(self) -> dict:
        """Return mailbox statistics."""
        msgs = list(self.messages.values())
        return {
            "owner": self.owner,
            "total": len(msgs),
            "unread": sum(1 for m in msgs if not m.is_read),
            "flagged": sum(1 for m in msgs if m.is_flagged),
            "latest": max((m.date for m in msgs), default=None),
        }


class MailServer:
    """A simple mail server that routes messages between mailboxes."""

    def __init__(self) -> None:
        self.mailboxes: dict[str, Mailbox] = {}
        self._next_uid = 0

    def create_mailbox(self, owner: str) -> Mailbox:
        """Create a mailbox for a user."""
        mb = Mailbox(owner)
        self.mailboxes[owner] = mb
        return mb

    def send(self, sender: EmailAddress, recipients: list[EmailAddress],
             subject: str, body: str, headers: Optional[dict[str, str]] = None) -> MailMessage:
        """Send a message. Delivers to all recipients with local mailboxes."""
        self._next_uid += 1
        uid = hashlib.sha256(f"{self._next_uid}-{subject}".encode()).hexdigest()[:12]
        msg = MailMessage(
            uid=uid,
            sender=sender,
            recipients=recipients,
            subject=subject,
            body=body,
            date=datetime.now(timezone.utc),
            headers=headers or {},
        )
        for recip in recipients:
            if recip.address in self.mailboxes:
                self.mailboxes[recip.address].deliver(msg)
        return msg

    def get_mailbox(self, owner: str) -> Optional[Mailbox]:
        return self.mailboxes.get(owner)


def poll_check(mailbox: Mailbox, mail_filter: Optional[MailFilter] = None,
               interval_label: str = "") -> CheckResult:
    """Perform a single poll check — the operation that a scheduler would invoke."""
    result = mailbox.check(mail_filter)
    return result


# --- demo ---
if __name__ == "__main__":
    server = MailServer()
    nathan = server.create_mailbox("nathan@em.com")
    kathryn = server.create_mailbox("kathryn@em.com")

    # Kathryn sends Nathan a finance report
    server.send(
        sender=EmailAddress("Kathryn Lyonne", "kathryn@em.com"),
        recipients=[EmailAddress("Nathan", "nathan@em.com")],
        subject="Q2 Finance Positions",
        body="Morning briefing: forex positions are tracking ahead of pace. "
             "Covered Captivate and Anthropic bills. DigitalOcean due in 3 days.",
    )
    server.send(
        sender=EmailAddress("Jay Harper", "jay@em.com"),
        recipients=[EmailAddress("Nathan", "nathan@em.com")],
        subject="Git commit summary",
        body="12 commits across 3 repos. Rosetta Stone: G036-G038 complete.",
    )
    server.send(
        sender=EmailAddress("Spam Bot", "spam@junk.com"),
        recipients=[EmailAddress("Nathan", "nathan@em.com")],
        subject="You won a prize!",
        body="Click here to claim your free vacation.",
    )

    # Check with filter
    result = nathan.check(MailFilter(sender_contains="em.com"))
    print(result.summary())
    print(f"\nUnread: {nathan.unread_count()}")
    print(f"Stats: {nathan.stats()}")
