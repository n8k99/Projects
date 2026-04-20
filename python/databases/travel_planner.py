"""Travel Planner System — leg-chained itineraries with integrity validation.

Trip = ordered list of Legs. Validation checks leg chain: location
continuity, time non-overlap, tight layovers below minimum. Layovers
enumerated between consecutive legs. Metrics: span, in-transit, cost.
"""

from __future__ import annotations
from dataclasses import dataclass, field
from enum import Enum


class TransportMode(Enum):
    FLIGHT = "flight"
    TRAIN = "train"
    BUS = "bus"
    CAR = "car"
    WALK = "walk"


@dataclass
class Leg:
    id: int
    mode: TransportMode
    origin: str
    destination: str
    depart_ms: int
    arrive_ms: int
    cost_cents: int
    carrier: str = ""

    def duration_ms(self) -> int:
        return self.arrive_ms - self.depart_ms


@dataclass
class Trip:
    id: int
    title: str
    legs: list[Leg] = field(default_factory=list)
    min_layover_minutes: int = 30

    def add_leg(self, leg: Leg) -> None:
        self.legs.append(leg)

    def total_span_ms(self) -> int:
        if not self.legs:
            return 0
        return self.legs[-1].arrive_ms - self.legs[0].depart_ms

    def in_transit_ms(self) -> int:
        return sum(l.duration_ms() for l in self.legs)

    def total_cost_cents(self) -> int:
        return sum(l.cost_cents for l in self.legs)

    def total_layover_ms(self) -> int:
        return self.total_span_ms() - self.in_transit_ms()


@dataclass
class Layover:
    location: str
    arrive_ms: int
    depart_ms: int
    duration_ms: int


def layovers(trip: Trip) -> list[Layover]:
    out = []
    for i in range(len(trip.legs) - 1):
        prev = trip.legs[i]
        next_leg = trip.legs[i + 1]
        out.append(Layover(
            location=prev.destination,
            arrive_ms=prev.arrive_ms,
            depart_ms=next_leg.depart_ms,
            duration_ms=next_leg.depart_ms - prev.arrive_ms,
        ))
    return out


class IssueSeverity(Enum):
    INFO = "info"
    WARNING = "warning"
    ERROR = "error"


class IssueKind(Enum):
    LOCATION_MISMATCH = "location_mismatch"
    TIME_CONFLICT = "time_conflict"
    TIGHT_LAYOVER = "tight_layover"
    ZERO_DURATION_LEG = "zero_duration_leg"
    NEGATIVE_DURATION_LEG = "negative_duration_leg"


@dataclass
class Issue:
    severity: IssueSeverity
    kind: IssueKind
    leg_idx: int
    detail: str = ""


def validate_trip(trip: Trip) -> list[Issue]:
    issues: list[Issue] = []

    for i, leg in enumerate(trip.legs):
        d = leg.duration_ms()
        if d == 0:
            issues.append(Issue(IssueSeverity.WARNING, IssueKind.ZERO_DURATION_LEG, i))
        elif d < 0:
            issues.append(Issue(IssueSeverity.ERROR, IssueKind.NEGATIVE_DURATION_LEG, i))

    for i in range(len(trip.legs) - 1):
        prev = trip.legs[i]
        next_leg = trip.legs[i + 1]
        if prev.destination != next_leg.origin:
            issues.append(Issue(
                IssueSeverity.ERROR, IssueKind.LOCATION_MISMATCH, i + 1,
                detail=f"{prev.destination} -> {next_leg.origin}",
            ))
        if next_leg.depart_ms < prev.arrive_ms:
            issues.append(Issue(
                IssueSeverity.ERROR, IssueKind.TIME_CONFLICT, i + 1,
                detail=f"arrive={prev.arrive_ms}, depart={next_leg.depart_ms}",
            ))
        else:
            layover_ms = next_leg.depart_ms - prev.arrive_ms
            layover_min = layover_ms // 60_000
            if layover_min < trip.min_layover_minutes and layover_ms > 0:
                issues.append(Issue(
                    IssueSeverity.WARNING, IssueKind.TIGHT_LAYOVER, i + 1,
                    detail=f"{layover_min} min",
                ))

    return issues


def demo() -> None:
    trip = Trip(1, "NYC -> SFO -> LAX")
    trip.legs.append(Leg(1, TransportMode.FLIGHT, "JFK", "SFO",
                         100_000_000, 120_000_000, 30000, "ACME Air"))
    trip.legs.append(Leg(2, TransportMode.FLIGHT, "SFO", "LAX",
                         130_000_000, 135_000_000, 15000, "ACME Air"))

    print(f"=== {trip.title} ===")
    print(f"  total span:    {trip.total_span_ms() / 60000:.1f} min")
    print(f"  in transit:    {trip.in_transit_ms() / 60000:.1f} min")
    print(f"  layovers:      {trip.total_layover_ms() / 60000:.1f} min")
    print(f"  total cost:    ${trip.total_cost_cents() / 100:.2f}")

    print("\n=== layovers ===")
    for lo in layovers(trip):
        print(f"  {lo.location}: {lo.duration_ms / 60000:.1f} min")

    print("\n=== validation ===")
    for issue in validate_trip(trip):
        print(f"  [{issue.severity.value}] {issue.kind.value} on leg {issue.leg_idx}: {issue.detail}")


if __name__ == "__main__":
    demo()
