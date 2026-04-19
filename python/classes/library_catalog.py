"""Library Catalog — three-layer identity, hierarchical subjects, FIFO holds, renewal.

Three layers of identity:
  Title  — the abstract work ("1984" by Orwell).
  Copy   — a physical item with its own location and status.
  Loan   — the temporal claim on one copy by one patron.

Subjects are stored as dot-delimited paths ("500.512.5") and queried by
prefix, making the Dewey-style taxonomy a tree. Holds form a FIFO queue per
title: when a copy is returned and holds exist, the next hold is fulfilled.
"""

from dataclasses import dataclass, field
from datetime import datetime, timedelta, timezone
from enum import Enum
from typing import Optional
import itertools


class CopyStatus(Enum):
    AVAILABLE = "available"
    CHECKED_OUT = "checked_out"
    LOST = "lost"
    WITHDRAWN = "withdrawn"


class HoldStatus(Enum):
    WAITING = "waiting"
    FULFILLED = "fulfilled"
    CANCELLED = "cancelled"


@dataclass
class Title:
    id: str
    name: str
    authors: list[str] = field(default_factory=list)
    subject_path: str = ""     # dot-delimited, e.g. "500.512.5"
    isbn: str = ""


@dataclass
class Copy:
    id: str
    title_id: str
    location: str = ""
    status: CopyStatus = CopyStatus.AVAILABLE


@dataclass
class Patron:
    id: str
    name: str


@dataclass
class Loan:
    id: int
    copy_id: str
    patron_id: str
    checked_out_at: datetime
    due_date: datetime
    returned_at: Optional[datetime] = None
    renewal_count: int = 0

    @property
    def is_active(self) -> bool:
        return self.returned_at is None

    def is_overdue(self, now: datetime) -> bool:
        return self.is_active and now > self.due_date


@dataclass
class Hold:
    """A patron's FIFO place in line for a title when no copies are available."""
    id: int
    title_id: str
    patron_id: str
    placed_at: datetime
    status: HoldStatus = HoldStatus.WAITING
    fulfilled_copy_id: Optional[str] = None


class CatalogError(Exception):
    pass


class UnknownEntity(CatalogError): ...
class NotAvailable(CatalogError): ...
class AlreadyOut(CatalogError): ...
class RenewalBlocked(CatalogError): ...


class LibraryCatalog:
    DEFAULT_LOAN_DAYS = 21
    MAX_RENEWALS = 2

    def __init__(self) -> None:
        self.titles: dict[str, Title] = {}
        self.copies: dict[str, Copy] = {}
        self.patrons: dict[str, Patron] = {}
        self.loans: dict[int, Loan] = {}
        self.holds: dict[int, Hold] = {}
        self._loan_ids = itertools.count(1)
        self._hold_ids = itertools.count(1)

    # --- registration ---
    def add_title(self, title: Title) -> None:
        if title.id in self.titles:
            raise ValueError(f"title already registered: {title.id}")
        self.titles[title.id] = title

    def add_copy(self, copy: Copy) -> None:
        if copy.title_id not in self.titles:
            raise UnknownEntity(f"unknown title: {copy.title_id}")
        if copy.id in self.copies:
            raise ValueError(f"copy already registered: {copy.id}")
        self.copies[copy.id] = copy

    def add_patron(self, patron: Patron) -> None:
        if patron.id in self.patrons:
            raise ValueError(f"patron already registered: {patron.id}")
        self.patrons[patron.id] = patron

    # --- checkout / return ---
    def check_out(self, copy_id: str, patron_id: str,
                  duration_days: int = DEFAULT_LOAN_DAYS,
                  now: Optional[datetime] = None) -> Loan:
        if copy_id not in self.copies:
            raise UnknownEntity(f"unknown copy: {copy_id}")
        if patron_id not in self.patrons:
            raise UnknownEntity(f"unknown patron: {patron_id}")
        copy = self.copies[copy_id]
        if copy.status != CopyStatus.AVAILABLE:
            raise AlreadyOut(f"copy {copy_id} not available (status: {copy.status.value})")
        at = now or datetime.now(timezone.utc)
        loan = Loan(
            id=next(self._loan_ids),
            copy_id=copy_id,
            patron_id=patron_id,
            checked_out_at=at,
            due_date=at + timedelta(days=duration_days),
        )
        self.loans[loan.id] = loan
        copy.status = CopyStatus.CHECKED_OUT
        return loan

    def return_copy(self, loan_id: int,
                    now: Optional[datetime] = None) -> tuple[Loan, Optional[Hold]]:
        """Return a copy. If holds exist for its title, fulfill the next one."""
        if loan_id not in self.loans:
            raise UnknownEntity(f"unknown loan: {loan_id}")
        loan = self.loans[loan_id]
        if loan.returned_at is not None:
            raise CatalogError(f"loan {loan_id} already returned")
        at = now or datetime.now(timezone.utc)
        loan.returned_at = at
        copy = self.copies[loan.copy_id]
        copy.status = CopyStatus.AVAILABLE

        fulfilled = self._fulfill_next_hold(copy.title_id, copy.id, at)
        return (loan, fulfilled)

    def _fulfill_next_hold(self, title_id: str, copy_id: str,
                            now: datetime) -> Optional[Hold]:
        waiting = sorted(
            (h for h in self.holds.values()
             if h.title_id == title_id and h.status == HoldStatus.WAITING),
            key=lambda h: h.placed_at,
        )
        if not waiting:
            return None
        hold = waiting[0]
        hold.status = HoldStatus.FULFILLED
        hold.fulfilled_copy_id = copy_id
        return hold

    # --- renewal ---
    def renew(self, loan_id: int, additional_days: int = DEFAULT_LOAN_DAYS) -> Loan:
        if loan_id not in self.loans:
            raise UnknownEntity(f"unknown loan: {loan_id}")
        loan = self.loans[loan_id]
        if not loan.is_active:
            raise CatalogError(f"loan {loan_id} is not active")
        if loan.renewal_count >= self.MAX_RENEWALS:
            raise RenewalBlocked(
                f"loan {loan_id} reached max renewals ({self.MAX_RENEWALS})"
            )
        title_id = self.copies[loan.copy_id].title_id
        if any(h.title_id == title_id and h.status == HoldStatus.WAITING
               for h in self.holds.values()):
            raise RenewalBlocked(
                f"cannot renew {loan_id}: other patrons have holds on this title"
            )
        loan.due_date += timedelta(days=additional_days)
        loan.renewal_count += 1
        return loan

    # --- holds ---
    def place_hold(self, title_id: str, patron_id: str,
                   now: Optional[datetime] = None) -> Hold:
        if title_id not in self.titles:
            raise UnknownEntity(f"unknown title: {title_id}")
        if patron_id not in self.patrons:
            raise UnknownEntity(f"unknown patron: {patron_id}")
        hold = Hold(
            id=next(self._hold_ids),
            title_id=title_id,
            patron_id=patron_id,
            placed_at=now or datetime.now(timezone.utc),
        )
        self.holds[hold.id] = hold
        return hold

    def cancel_hold(self, hold_id: int) -> Hold:
        if hold_id not in self.holds:
            raise UnknownEntity(f"unknown hold: {hold_id}")
        hold = self.holds[hold_id]
        if hold.status != HoldStatus.WAITING:
            raise CatalogError(f"hold {hold_id} is not waiting")
        hold.status = HoldStatus.CANCELLED
        return hold

    def hold_queue(self, title_id: str) -> list[Hold]:
        """Active holds for a title, in FIFO order by placed_at."""
        if title_id not in self.titles:
            raise UnknownEntity(f"unknown title: {title_id}")
        return sorted(
            (h for h in self.holds.values()
             if h.title_id == title_id and h.status == HoldStatus.WAITING),
            key=lambda h: h.placed_at,
        )

    # --- queries ---
    def copies_of(self, title_id: str) -> list[Copy]:
        if title_id not in self.titles:
            raise UnknownEntity(f"unknown title: {title_id}")
        return [c for c in self.copies.values() if c.title_id == title_id]

    def available_copies(self, title_id: str) -> list[Copy]:
        return [c for c in self.copies_of(title_id) if c.status == CopyStatus.AVAILABLE]

    def is_available(self, title_id: str) -> bool:
        return len(self.available_copies(title_id)) > 0

    def search_by_subject(self, prefix: str) -> list[Title]:
        """Tree-prefix query: all titles whose subject_path starts with prefix."""
        return [t for t in self.titles.values()
                if t.subject_path == prefix or t.subject_path.startswith(prefix + ".")]

    def search_by_author(self, author: str) -> list[Title]:
        return [t for t in self.titles.values() if author in t.authors]

    def search_by_name(self, substring: str) -> list[Title]:
        s = substring.lower()
        return [t for t in self.titles.values() if s in t.name.lower()]

    def loans_for_patron(self, patron_id: str, active_only: bool = False) -> list[Loan]:
        if patron_id not in self.patrons:
            raise UnknownEntity(f"unknown patron: {patron_id}")
        return [l for l in self.loans.values()
                if l.patron_id == patron_id and (not active_only or l.is_active)]

    def overdue(self, as_of: Optional[datetime] = None) -> list[Loan]:
        now = as_of or datetime.now(timezone.utc)
        return [l for l in self.loans.values() if l.is_overdue(now)]


# --- demo ---
if __name__ == "__main__":
    cat = LibraryCatalog()
    cat.add_title(Title("T-1984", "1984", ["Orwell"], subject_path="800.813", isbn="0451524934"))
    cat.add_title(Title("T-ELEMENTS", "Elements", ["Euclid"], subject_path="500.510.1"))
    cat.add_title(Title("T-ALGEBRA", "A Book of Abstract Algebra", ["Pinter"], subject_path="500.512.02"))

    cat.add_copy(Copy("C-1984-A", "T-1984", location="Main"))
    cat.add_copy(Copy("C-1984-B", "T-1984", location="Branch"))
    cat.add_copy(Copy("C-ELEMENTS-A", "T-ELEMENTS", location="Main"))
    cat.add_copy(Copy("C-ALGEBRA-A", "T-ALGEBRA", location="Main"))

    cat.add_patron(Patron("P-001", "Nathan"))
    cat.add_patron(Patron("P-002", "Korrallan"))

    print("math titles (subject 500):", [t.id for t in cat.search_by_subject("500")])
    print("pure math (500.512):    ", [t.id for t in cat.search_by_subject("500.512")])

    l1 = cat.check_out("C-1984-A", "P-001")
    l2 = cat.check_out("C-1984-B", "P-002")
    print(f"both copies out; available? {cat.is_available('T-1984')}")

    hold = cat.place_hold("T-1984", "P-001")
    print(f"queue length: {len(cat.hold_queue('T-1984'))}")

    try:
        cat.renew(l1.id)
    except RenewalBlocked as e:
        print(f"renewal blocked: {e}")

    _, fulfilled = cat.return_copy(l2.id)
    if fulfilled:
        print(f"hold {fulfilled.id} fulfilled by {fulfilled.fulfilled_copy_id}")
