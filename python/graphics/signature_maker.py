"""Signature Maker — stroke model + Catmull-Rom + variable-width + normalize.

Stroke model: Point {x, y, pressure (0-1024), t_ms}; Stroke = list of points;
Signature = list of strokes.
Catmull-Rom spline smoothing with integer-scaled math for cross-lang parity.
Variable-width raster: pressure maps to disk-stamp radius along Bresenham line.
Bounding-box normalization: translate + scale to target box.
Similarity: sum of squared distances from each point to nearest point in other.
"""

from __future__ import annotations
from dataclasses import dataclass, field
from graphics.grayscale import Image, Rgb
from typing import Optional


@dataclass(frozen=True)
class Point:
    x: int
    y: int
    pressure: int  # 0..1024
    t_ms: int


@dataclass
class Stroke:
    points: list[Point] = field(default_factory=list)


@dataclass
class Signature:
    strokes: list[Stroke] = field(default_factory=list)

    def add_stroke(self, stroke: Stroke) -> None:
        self.strokes.append(stroke)

    def point_count(self) -> int:
        return sum(len(s.points) for s in self.strokes)

    def bounding_box(self) -> Optional[tuple[int, int, int, int]]:
        min_x, min_y = 10**18, 10**18
        max_x, max_y = -10**18, -10**18
        any_point = False
        for stroke in self.strokes:
            for p in stroke.points:
                if p.x < min_x: min_x = p.x
                if p.y < min_y: min_y = p.y
                if p.x > max_x: max_x = p.x
                if p.y > max_y: max_y = p.y
                any_point = True
        if not any_point:
            return None
        return (min_x, min_y, max_x, max_y)


def catmull_rom(p0: tuple[int, int], p1: tuple[int, int],
                 p2: tuple[int, int], p3: tuple[int, int],
                 t: int) -> tuple[int, int]:
    """t in 0..=1000 (integer scale). Returns integer (x, y)."""
    t2 = t * t
    t3 = t2 * t // 1000

    def compute(a: int, b: int, c: int, d: int) -> int:
        term_a = 2 * b
        term_b = (-a + c) * t
        term_c = (2 * a - 5 * b + 4 * c - d) * t2
        term_d = (-a + 3 * b - 3 * c + d) * t3
        sum_ = (term_a * 1_000_000
                + term_b * 1_000
                + term_c
                + term_d // 1_000)
        return sum_ // 2_000_000

    x = compute(p0[0], p1[0], p2[0], p3[0])
    y = compute(p0[1], p1[1], p2[1], p3[1])
    return (x, y)


def smooth_stroke(stroke: Stroke, steps: int) -> Stroke:
    pts = stroke.points
    if len(pts) < 4 or steps == 0:
        return Stroke(points=list(pts))
    out = [pts[0]]
    for i in range(len(pts) - 3):
        p0 = (pts[i].x, pts[i].y)
        p1 = (pts[i + 1].x, pts[i + 1].y)
        p2 = (pts[i + 2].x, pts[i + 2].y)
        p3 = (pts[i + 3].x, pts[i + 3].y)
        for s in range(1, steps + 1):
            t = s * 1000 // steps
            x, y = catmull_rom(p0, p1, p2, p3, t)
            pressure = pts[i + 1].pressure + (pts[i + 2].pressure - pts[i + 1].pressure) * t // 1000
            pressure = max(0, min(1024, pressure))
            t_ms = pts[i + 1].t_ms + (pts[i + 2].t_ms - pts[i + 1].t_ms) * t // 1000
            out.append(Point(x=x, y=y, pressure=pressure, t_ms=max(0, t_ms)))
    out.append(pts[-1])
    return Stroke(points=out)


def normalize(sig: Signature, target_w: int, target_h: int) -> Signature:
    bbox = sig.bounding_box()
    if bbox is None:
        return Signature(strokes=[Stroke(list(s.points)) for s in sig.strokes])
    min_x, min_y, max_x, max_y = bbox
    w = max(1, max_x - min_x)
    h = max(1, max_y - min_y)
    scale_x = target_w * 1000 // w
    scale_y = target_h * 1000 // h
    new_strokes = []
    for s in sig.strokes:
        new_points = []
        for p in s.points:
            nx = (p.x - min_x) * scale_x // 1000
            ny = (p.y - min_y) * scale_y // 1000
            new_points.append(Point(x=nx, y=ny, pressure=p.pressure, t_ms=p.t_ms))
        new_strokes.append(Stroke(points=new_points))
    return Signature(strokes=new_strokes)


def similarity_distance(a: Signature, b: Signature) -> int:
    total = 0
    for stroke_a in a.strokes:
        for p in stroke_a.points:
            min_d2 = 10**18
            for stroke_b in b.strokes:
                for q in stroke_b.points:
                    dx = p.x - q.x
                    dy = p.y - q.y
                    d2 = dx * dx + dy * dy
                    if d2 < min_d2:
                        min_d2 = d2
            total += min_d2
    return total


def _set_pixel(img: Image, x: int, y: int, color: Rgb) -> None:
    if 0 <= x < img.width and 0 <= y < img.height:
        img.pixels[y * img.width + x] = color


def _stamp_disk(img: Image, cx: int, cy: int, radius: int, color: Rgb) -> None:
    r = radius
    r2 = r * r
    for dy in range(-r, r + 1):
        for dx in range(-r, r + 1):
            if dx * dx + dy * dy > r2:
                continue
            _set_pixel(img, cx + dx, cy + dy, color)


def _stamp_line(img: Image, x1: int, y1: int, x2: int, y2: int,
                 width: int, color: Rgb) -> None:
    radius = max(1, width // 2)
    dx = abs(x2 - x1)
    dy = -abs(y2 - y1)
    sx = 1 if x1 < x2 else -1
    sy = 1 if y1 < y2 else -1
    err = dx + dy
    x, y = x1, y1
    while True:
        _stamp_disk(img, x, y, radius, color)
        if x == x2 and y == y2:
            break
        e2 = 2 * err
        if e2 >= dy:
            err += dy
            x += sx
        if e2 <= dx:
            err += dx
            y += sy


def render_to_image(sig: Signature, width: int, height: int,
                     max_width: int, bg: Rgb, ink: Rgb) -> Image:
    img = Image(width, height, [bg for _ in range(width * height)])
    for stroke in sig.strokes:
        if len(stroke.points) < 2:
            continue
        for i in range(len(stroke.points) - 1):
            a = stroke.points[i]
            b = stroke.points[i + 1]
            w_a = a.pressure * max_width // 1024
            w_b = b.pressure * max_width // 1024
            w_seg = max(1, (w_a + w_b) // 2)
            _stamp_line(img, a.x, a.y, b.x, b.y, w_seg, ink)
    return img


def demo() -> None:
    sig = Signature()
    sig.add_stroke(Stroke(points=[
        Point(x=10, y=20, pressure=512, t_ms=0),
        Point(x=30, y=40, pressure=768, t_ms=100),
        Point(x=50, y=30, pressure=1024, t_ms=200),
        Point(x=70, y=50, pressure=512, t_ms=300),
    ]))
    sig.add_stroke(Stroke(points=[
        Point(x=80, y=30, pressure=600, t_ms=400),
        Point(x=100, y=50, pressure=700, t_ms=500),
    ]))

    print(f"point count: {sig.point_count()}")
    print(f"bbox: {sig.bounding_box()}")

    smoothed = smooth_stroke(sig.strokes[0], 5)
    print(f"smoothed points: {len(smoothed.points)}")

    norm = normalize(sig, 1000, 1000)
    print(f"normalized bbox: {norm.bounding_box()}")

    # Catmull-Rom endpoints
    x0, y0 = catmull_rom((0, 0), (10, 10), (20, 10), (30, 0), 0)
    x1, y1 = catmull_rom((0, 0), (10, 10), (20, 10), (30, 0), 1000)
    print(f"cr t=0: ({x0}, {y0})")
    print(f"cr t=1000: ({x1}, {y1})")

    # Similarity: same sig to itself = 0
    print(f"self-similarity: {similarity_distance(sig, sig)}")

    # Same shape different size → 0 after normalize
    small = Signature()
    small.add_stroke(Stroke(points=[
        Point(0, 0, 500, 0), Point(10, 0, 500, 100),
        Point(10, 10, 500, 200), Point(0, 10, 500, 300),
    ]))
    big = Signature()
    big.add_stroke(Stroke(points=[
        Point(0, 0, 500, 0), Point(100, 0, 500, 100),
        Point(100, 100, 500, 200), Point(0, 100, 500, 300),
    ]))
    n_s = normalize(small, 1000, 1000)
    n_b = normalize(big, 1000, 1000)
    print(f"normalized same-shape similarity: {similarity_distance(n_s, n_b)}")


if __name__ == "__main__":
    demo()
