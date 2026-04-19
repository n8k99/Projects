"""Reservation System — bookable resources with time intervals and conflict detection.

Models airline seats and hotel rooms uniformly: a `Resource` is any bookable
slot, and a `Reservation` is an interval claim on one resource by one customer.
Overlap is the core primitive — two reservations on the same resource collide
iff their half-open intervals [start, end) overlap.
"""

from dataclasses import dataclass, field
from datetime import datetime, timezone
from enum import Enum
from typing import Optional
import itertools


class ReservationStatus(Enum):
    BOOKED = "booked"
    CANCELLED = "cancelled"
    COMPLETED = "completed"


@dataclass
class Resource:
    """A specific bookable thing. `12A` or `Room 304` — not fungible."""
    id: str
    kind: str          # "seat", "room", "slot", ...
    label: str = ""
    attributes: dict[str, str] = field(default_factory=dict)


@dataclass
class Customer:
    id: str
    name: str


@dataclass
class Reservation:
    id: int
    resource_id: str
    customer_id: str
    start: datetime
    end: datetime
    status: ReservationStatus = ReservationStatus.BOOKED
    created_at: datetime = field(default_factory=lambda: datetime.now(timezone.utc))

    def overlaps(self, start: datetime, end: datetime) -> bool:
        """Half-open [start, end) overlap — touching boundaries do not conflict."""
        return self.start < end and start < self.end

    @property
    def is_active(self) -> bool:
        return self.status == ReservationStatus.BOOKED


class ReservationError(Exception):
    """Base class for reservation errors."""


class UnknownEntity(ReservationError): ...
class Conflict(ReservationError): ...
class InvalidInterval(ReservationError): ...
class AlreadyCancelled(ReservationError): ...


class ReservationSystem:
    """Scheduled resource allocation. Availability is a negative search over conflicts."""

    def __init__(self) -> None:
        self.resources: dict[str, Resource] = {}
        self.customers: dict[str, Customer] = {}
        self.reservations: dict[int, Reservation] = {}
        self._ids = itertools.count(1)

    # --- registration ---
    def add_resource(self, resource: Resource) -> None:
        if resource.id in self.resources:
            raise ValueError(f"resource already registered: {resource.id}")
        self.resources[resource.id] = resource

    def add_customer(self, customer: Customer) -> None:
        if customer.id in self.customers:
            raise ValueError(f"customer already registered: {customer.id}")
        self.customers[customer.id] = customer

    # --- core: overlap and conflicts ---
    def conflicts(self, resource_id: str, start: datetime, end: datetime) -> list[Reservation]:
        """Active reservations on this resource whose interval overlaps [start, end)."""
        if resource_id not in self.resources:
            raise UnknownEntity(f"unknown resource: {resource_id}")
        if start >= end:
            raise InvalidInterval("start must be before end")
        return [r for r in self.reservations.values()
                if r.resource_id == resource_id and r.is_active and r.overlaps(start, end)]

    def book(self, resource_id: str, customer_id: str,
             start: datetime, end: datetime) -> Reservation:
        if resource_id not in self.resources:
            raise UnknownEntity(f"unknown resource: {resource_id}")
        if customer_id not in self.customers:
            raise UnknownEntity(f"unknown customer: {customer_id}")
        if start >= end:
            raise InvalidInterval("start must be before end")
        colliding = self.conflicts(resource_id, start, end)
        if colliding:
            raise Conflict(
                f"{resource_id} has {len(colliding)} overlapping reservation(s): "
                f"{[r.id for r in colliding]}")
        reservation = Reservation(
            id=next(self._ids),
            resource_id=resource_id,
            customer_id=customer_id,
            start=start,
            end=end,
        )
        self.reservations[reservation.id] = reservation
        return reservation

    def cancel(self, reservation_id: int) -> Reservation:
        if reservation_id not in self.reservations:
            raise UnknownEntity(f"unknown reservation: {reservation_id}")
        r = self.reservations[reservation_id]
        if r.status == ReservationStatus.CANCELLED:
            raise AlreadyCancelled(f"reservation {reservation_id} already cancelled")
        r.status = ReservationStatus.CANCELLED
        return r

    # --- queries ---
    def available(self, start: datetime, end: datetime,
                  kind: Optional[str] = None) -> list[Resource]:
        """Resources with no overlapping active reservation across [start, end)."""
        if start >= end:
            raise InvalidInterval("start must be before end")
        out: list[Resource] = []
        for res in self.resources.values():
            if kind is not None and res.kind != kind:
                continue
            if not self.conflicts(res.id, start, end):
                out.append(res)
        return out

    def occupancy(self, resource_id: str) -> list[Reservation]:
        """All active reservations on a resource, sorted by start."""
        if resource_id not in self.resources:
            raise UnknownEntity(f"unknown resource: {resource_id}")
        active = [r for r in self.reservations.values()
                  if r.resource_id == resource_id and r.is_active]
        return sorted(active, key=lambda r: r.start)

    def customer_bookings(self, customer_id: str) -> list[Reservation]:
        if customer_id not in self.customers:
            raise UnknownEntity(f"unknown customer: {customer_id}")
        return [r for r in self.reservations.values() if r.customer_id == customer_id]

    def summary(self) -> dict:
        active = [r for r in self.reservations.values() if r.is_active]
        return {
            "resources": len(self.resources),
            "customers": len(self.customers),
            "reservations_total": len(self.reservations),
            "reservations_active": len(active),
            "resources_booked": len({r.resource_id for r in active}),
        }


# --- demo ---
if __name__ == "__main__":
    from datetime import timedelta

    sys = ReservationSystem()
    sys.add_resource(Resource("12A", kind="seat", label="Window, row 12, first class"))
    sys.add_resource(Resource("12B", kind="seat", label="Aisle, row 12, first class"))
    sys.add_resource(Resource("304", kind="room", label="Hotel room 304, king bed"))
    sys.add_customer(Customer("C001", "Nathan"))
    sys.add_customer(Customer("C002", "Korrallan"))

    t = datetime(2026, 5, 1, 14, 0, tzinfo=timezone.utc)
    r1 = sys.book("12A", "C001", t, t + timedelta(hours=3))
    r2 = sys.book("304", "C001", t + timedelta(days=1), t + timedelta(days=4))

    try:
        sys.book("12A", "C002", t + timedelta(hours=1), t + timedelta(hours=4))
    except Conflict as e:
        print(f"conflict detected: {e}")

    r3 = sys.book("12B", "C002", t + timedelta(hours=1), t + timedelta(hours=4))
    free = sys.available(t, t + timedelta(hours=3), kind="seat")
    print(f"free seats during flight: {[r.id for r in free]}")
    print(sys.summary())
