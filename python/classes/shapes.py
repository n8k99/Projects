"""Shape Area and Perimeter — one interface, many implementations.

Python's polymorphism is duck-typed with an ABC to make the contract explicit.
Any object with `area()`, `perimeter()`, and `name()` methods is usable as a
Shape; subclassing ABC enforces the three-method contract at construction.

Collection operations (`total_area`, `largest_by_area`, `sort_by_perimeter`)
take `list[Shape]` and work uniformly — they never inspect the concrete type.
That's the win: heterogeneous data, homogeneous algorithms.
"""

from __future__ import annotations
from abc import ABC, abstractmethod
from dataclasses import dataclass
from math import pi, sqrt, tan


class Shape(ABC):
    @abstractmethod
    def area(self) -> float: ...
    @abstractmethod
    def perimeter(self) -> float: ...
    @abstractmethod
    def name(self) -> str: ...


@dataclass(frozen=True)
class Circle(Shape):
    radius: float

    def __post_init__(self) -> None:
        if self.radius < 0:
            raise ValueError(f"negative radius: {self.radius}")

    def area(self) -> float: return pi * self.radius * self.radius
    def perimeter(self) -> float: return 2 * pi * self.radius
    def name(self) -> str: return "Circle"


@dataclass(frozen=True)
class Rectangle(Shape):
    width: float
    height: float

    def __post_init__(self) -> None:
        if self.width < 0 or self.height < 0:
            raise ValueError(f"negative dimension: {self.width}, {self.height}")

    def area(self) -> float: return self.width * self.height
    def perimeter(self) -> float: return 2 * (self.width + self.height)
    def name(self) -> str:
        return "Square" if self.width == self.height else "Rectangle"


@dataclass(frozen=True)
class Triangle(Shape):
    a: float
    b: float
    c: float

    def __post_init__(self) -> None:
        sides = (self.a, self.b, self.c)
        if any(s <= 0 for s in sides):
            raise ValueError(f"non-positive side: {sides}")
        # Triangle inequality: each side must be less than the sum of the others.
        if not (self.a + self.b > self.c
                and self.a + self.c > self.b
                and self.b + self.c > self.a):
            raise ValueError(f"violates triangle inequality: {sides}")

    def area(self) -> float:
        # Heron's formula.
        s = (self.a + self.b + self.c) / 2
        return sqrt(s * (s - self.a) * (s - self.b) * (s - self.c))

    def perimeter(self) -> float: return self.a + self.b + self.c
    def name(self) -> str: return "Triangle"


@dataclass(frozen=True)
class RegularPolygon(Shape):
    sides: int
    side_length: float

    def __post_init__(self) -> None:
        if self.sides < 3:
            raise ValueError(f"polygon needs >= 3 sides: {self.sides}")
        if self.side_length < 0:
            raise ValueError(f"negative side length: {self.side_length}")

    def area(self) -> float:
        n, s = self.sides, self.side_length
        return n * s * s / (4 * tan(pi / n))

    def perimeter(self) -> float: return self.sides * self.side_length
    def name(self) -> str: return f"Regular-{self.sides}-gon"


# --- heterogeneous collection operations (interface-only) ---

def total_area(shapes: list[Shape]) -> float:
    return sum(s.area() for s in shapes)


def total_perimeter(shapes: list[Shape]) -> float:
    return sum(s.perimeter() for s in shapes)


def largest_by_area(shapes: list[Shape]) -> Shape:
    if not shapes:
        raise ValueError("empty list has no largest")
    return max(shapes, key=lambda s: s.area())


def sort_by_perimeter(shapes: list[Shape]) -> list[Shape]:
    return sorted(shapes, key=lambda s: s.perimeter())


def summary(shapes: list[Shape]) -> dict:
    by_kind: dict[str, int] = {}
    for s in shapes:
        by_kind[s.name()] = by_kind.get(s.name(), 0) + 1
    return {
        "count": len(shapes),
        "total_area": total_area(shapes),
        "total_perimeter": total_perimeter(shapes),
        "by_kind": by_kind,
    }


# --- demo ---
if __name__ == "__main__":
    shapes: list[Shape] = [
        Circle(5),
        Rectangle(4, 6),
        Rectangle(3, 3),                      # square
        Triangle(3, 4, 5),                    # right triangle
        Triangle(5, 5, 5),                    # equilateral
        RegularPolygon(sides=6, side_length=2),  # hexagon
    ]
    for s in shapes:
        print(f"  {s.name():14s}  area={s.area():7.3f}  perimeter={s.perimeter():7.3f}")

    print()
    print(f"total area        = {total_area(shapes):.3f}")
    print(f"total perimeter   = {total_perimeter(shapes):.3f}")

    biggest = largest_by_area(shapes)
    print(f"largest by area   = {biggest.name()} ({biggest.area():.3f})")

    ordered = sort_by_perimeter(shapes)
    print("\nsorted by perimeter (ascending):")
    for s in ordered:
        print(f"  {s.name():14s}  {s.perimeter():7.3f}")

    print(f"\nsummary: {summary(shapes)}")

    # Validation in action.
    try:
        Triangle(1, 2, 10)   # violates inequality
    except ValueError as e:
        print(f"\ncaught: {e}")
