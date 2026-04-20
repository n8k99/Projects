"""Image Map Generator — clickable regions on an image with hit testing.

Shape polymorphism (rect/circle/polygon); point-in-polygon via ray casting;
z-order topmost-first for overlap resolution. Export as HTML <map>.
"""

from __future__ import annotations
from dataclasses import dataclass, field
from enum import Enum


class ShapeKind(Enum):
    RECT = "rect"
    CIRCLE = "circle"
    POLYGON = "polygon"


@dataclass
class Shape:
    kind: ShapeKind
    # RECT
    x: int = 0
    y: int = 0
    w: int = 0
    h: int = 0
    # CIRCLE
    cx: int = 0
    cy: int = 0
    r: int = 0
    # POLYGON
    points: list[tuple[int, int]] = field(default_factory=list)

    @classmethod
    def rect(cls, x: int, y: int, w: int, h: int) -> Shape:
        return cls(ShapeKind.RECT, x=x, y=y, w=w, h=h)

    @classmethod
    def circle(cls, cx: int, cy: int, r: int) -> Shape:
        return cls(ShapeKind.CIRCLE, cx=cx, cy=cy, r=r)

    @classmethod
    def polygon(cls, points: list[tuple[int, int]]) -> Shape:
        return cls(ShapeKind.POLYGON, points=points)

    def contains(self, x: int, y: int) -> bool:
        if self.kind == ShapeKind.RECT:
            return self.x <= x < self.x + self.w and self.y <= y < self.y + self.h
        if self.kind == ShapeKind.CIRCLE:
            dx = x - self.cx
            dy = y - self.cy
            return dx * dx + dy * dy <= self.r * self.r
        if self.kind == ShapeKind.POLYGON:
            return _point_in_polygon(x, y, self.points)
        return False

    def bbox(self) -> tuple[int, int, int, int]:
        if self.kind == ShapeKind.RECT:
            return (self.x, self.y, self.w, self.h)
        if self.kind == ShapeKind.CIRCLE:
            return (self.cx - self.r, self.cy - self.r, 2 * self.r, 2 * self.r)
        if self.kind == ShapeKind.POLYGON:
            xs = [p[0] for p in self.points]
            ys = [p[1] for p in self.points]
            return (min(xs), min(ys), max(xs) - min(xs), max(ys) - min(ys))
        return (0, 0, 0, 0)


@dataclass
class Region:
    id: int
    shape: Shape
    action: str
    alt: str
    z: int = 0


class ImageMap:
    def __init__(self) -> None:
        self.regions: list[Region] = []

    def add(self, region: Region) -> None:
        self.regions.append(region)

    def hit_test(self, x: int, y: int) -> Region | None:
        topmost = sorted(self.regions, key=lambda r: -r.z)
        for r in topmost:
            if r.shape.contains(x, y):
                return r
        return None

    def hit_test_all(self, x: int, y: int) -> list[Region]:
        topmost = sorted(self.regions, key=lambda r: -r.z)
        return [r for r in topmost if r.shape.contains(x, y)]

    def to_html(self, map_name: str) -> str:
        parts = [f'<map name="{map_name}">']
        for r in self.regions:
            if r.shape.kind == ShapeKind.RECT:
                s = r.shape
                shape_name = "rect"
                coords = f"{s.x},{s.y},{s.x + s.w},{s.y + s.h}"
            elif r.shape.kind == ShapeKind.CIRCLE:
                s = r.shape
                shape_name = "circle"
                coords = f"{s.cx},{s.cy},{s.r}"
            else:
                shape_name = "poly"
                coords = ",".join(f"{x},{y}" for x, y in r.shape.points)
            parts.append(
                f'  <area shape="{shape_name}" coords="{coords}" '
                f'href="{r.action}" alt="{r.alt}">'
            )
        parts.append("</map>")
        return "\n".join(parts) + "\n"


def _point_in_polygon(x: int, y: int, points: list[tuple[int, int]]) -> bool:
    if len(points) < 3:
        return False
    inside = False
    n = len(points)
    j = n - 1
    for i in range(n):
        xi, yi = points[i]
        xj, yj = points[j]
        if (yi > y) != (yj > y):
            x_intersect = (xj - xi) * (y - yi) // (yj - yi) + xi
            if x < x_intersect:
                inside = not inside
        j = i
    return inside


def demo() -> None:
    m = ImageMap()
    m.add(Region(1, Shape.rect(0, 0, 100, 100), "/home", "Home", 0))
    m.add(Region(2, Shape.circle(50, 50, 20), "/center", "Centre", 1))
    m.add(Region(3, Shape.polygon([(200, 0), (250, 0), (225, 50)]), "/tri", "Tri", 0))

    print("=== hit tests ===")
    for (x, y) in [(10, 10), (50, 50), (225, 25), (500, 500)]:
        hit = m.hit_test(x, y)
        print(f"  ({x:3},{y:3}) -> {hit.action if hit else 'miss'}")

    print("\n=== HTML ===")
    print(m.to_html("nav"))


if __name__ == "__main__":
    demo()
