"""Online White Board — shared canvas of strokes with spatial queries.

First Rosetta Stone project with spatial/geometric data: coordinates,
bounding boxes, distance queries. Strokes are first-class entities with
identity, z-order (insertion order), and authorship. The data structure
IS the display; ``to_svg`` is a walk in z-order producing markup.
"""

from __future__ import annotations
import math
from dataclasses import dataclass, field


@dataclass(frozen=True)
class Point:
    x: float
    y: float

    def distance_to(self, other: Point) -> float:
        return math.hypot(self.x - other.x, self.y - other.y)


StrokeId = int


@dataclass
class Stroke:
    id: StrokeId
    author: str
    color: str
    thickness: float
    points: list[Point]

    def bbox(self) -> tuple[Point, Point] | None:
        if not self.points:
            return None
        xs = [p.x for p in self.points]
        ys = [p.y for p in self.points]
        return (Point(min(xs), min(ys)), Point(max(xs), max(ys)))

    def distance_to(self, point: Point) -> float:
        if not self.points:
            return math.inf
        return min(p.distance_to(point) for p in self.points)


class Whiteboard:
    def __init__(self) -> None:
        self._strokes: list[Stroke] = []
        self._next_id = 1

    def add_stroke(self, author: str, color: str, thickness: float,
                   points: list[Point]) -> StrokeId:
        sid = self._next_id
        self._next_id += 1
        self._strokes.append(Stroke(
            id=sid, author=author, color=color,
            thickness=thickness, points=list(points),
        ))
        return sid

    def remove_stroke(self, sid: StrokeId) -> bool:
        for i, s in enumerate(self._strokes):
            if s.id == sid:
                del self._strokes[i]
                return True
        return False

    def clear(self) -> None:
        self._strokes.clear()

    def stroke_count(self) -> int:
        return len(self._strokes)

    def strokes(self) -> list[Stroke]:
        return list(self._strokes)

    def strokes_by_author(self, author: str) -> list[Stroke]:
        return [s for s in self._strokes if s.author == author]

    def strokes_near(self, point: Point, radius: float) -> list[Stroke]:
        return [s for s in self._strokes if s.distance_to(point) <= radius]

    def bounding_box(self) -> tuple[Point, Point] | None:
        boxes = [b for b in (s.bbox() for s in self._strokes) if b is not None]
        if not boxes:
            return None
        lo_x = min(b[0].x for b in boxes)
        lo_y = min(b[0].y for b in boxes)
        hi_x = max(b[1].x for b in boxes)
        hi_y = max(b[1].y for b in boxes)
        return (Point(lo_x, lo_y), Point(hi_x, hi_y))

    def to_svg(self, width: float, height: float) -> str:
        out = [f'<svg xmlns="http://www.w3.org/2000/svg" '
               f'width="{width}" height="{height}">']
        for s in self._strokes:
            if not s.points:
                continue
            pts = " ".join(f"{p.x},{p.y}" for p in s.points)
            out.append(f'  <polyline fill="none" stroke="{s.color}" '
                       f'stroke-width="{s.thickness}" points="{pts}"/>')
        out.append("</svg>")
        return "\n".join(out)


if __name__ == "__main__":
    wb = Whiteboard()
    wb.add_stroke("alice", "#ff0000", 2.0,
                  [Point(10, 10), Point(50, 10), Point(50, 50), Point(10, 50), Point(10, 10)])
    wb.add_stroke("bob", "#0000ff", 3.0,
                  [Point(60, 20), Point(100, 60)])
    wb.add_stroke("alice", "#00aa00", 1.5,
                  [Point(30, 30), Point(35, 35)])

    print(f"stroke count: {wb.stroke_count()}")
    print(f"alice's strokes: {len(wb.strokes_by_author('alice'))}")
    print(f"bounding box: {wb.bounding_box()}")

    near = wb.strokes_near(Point(32, 32), 5)
    print(f"strokes near (32, 32) within r=5: {len(near)} "
          f"(expected alice's small green one)")

    print()
    print(wb.to_svg(200, 100))
