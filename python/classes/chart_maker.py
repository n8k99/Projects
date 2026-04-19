"""Chart Making — a visualization pipeline as composed functions.

The chart is not a container of drawing commands. It is a small pipeline:

  data → scale → encode → layout → render

Each stage is a pure function. `LinearScale(d_min, d_max, r_min, r_max)` is
literally a function from a data domain to a visual range. `Encoding` is the
binding between data fields and visual channels (position, size, label).
`render` is the pluggable backend — here ASCII, but the same spec could emit
SVG or canvas with a different renderer.

This is the grammar-of-graphics shape: declarative spec + rendering backend,
separating WHAT TO SHOW from HOW TO DRAW IT.
"""

from __future__ import annotations
from dataclasses import dataclass, field
from typing import Callable


@dataclass
class DataPoint:
    label: str
    value: float
    series: str = ""


@dataclass
class Series:
    name: str
    points: list[DataPoint]


@dataclass
class LinearScale:
    """A function from a numeric domain to a numeric range. Callable."""
    domain_min: float
    domain_max: float
    range_min: float
    range_max: float

    def __call__(self, x: float) -> float:
        if self.domain_max == self.domain_min:
            return (self.range_min + self.range_max) / 2
        t = (x - self.domain_min) / (self.domain_max - self.domain_min)
        return self.range_min + t * (self.range_max - self.range_min)

    @staticmethod
    def fit(values: list[float], range_min: float, range_max: float,
            include_zero: bool = True) -> LinearScale:
        """Build a scale that comfortably spans the data."""
        if not values:
            return LinearScale(0, 1, range_min, range_max)
        lo = min(values)
        hi = max(values)
        if include_zero:
            lo = min(lo, 0)
            hi = max(hi, 0)
        if lo == hi:
            lo -= 1
            hi += 1
        return LinearScale(lo, hi, range_min, range_max)


@dataclass
class OrdinalScale:
    """Maps a discrete set of categories to consecutive positions."""
    categories: list[str]
    range_min: int
    range_max: int

    def __call__(self, category: str) -> int:
        if category not in self.categories:
            raise ValueError(f"unknown category: {category}")
        n = len(self.categories)
        idx = self.categories.index(category)
        if n <= 1:
            return (self.range_min + self.range_max) // 2
        step = (self.range_max - self.range_min) / (n - 1)
        return int(self.range_min + idx * step)


# --- chart specifications ---

@dataclass
class BarChart:
    """Horizontal ASCII bar chart."""
    title: str
    points: list[DataPoint]
    width: int = 40

    def render(self) -> str:
        if not self.points:
            return f"{self.title}\n(no data)"
        values = [p.value for p in self.points]
        # Bars grow from 0 to max; negatives aren't modeled here.
        scale = LinearScale.fit(values, 0, self.width, include_zero=True)
        label_w = max(len(p.label) for p in self.points)
        lines = [self.title, "=" * (label_w + 4 + self.width + 10)]
        for p in self.points:
            bar_len = max(0, int(round(scale(p.value))))
            bar = "█" * bar_len
            lines.append(f"  {p.label.ljust(label_w)}  {bar.ljust(self.width)}  {p.value:.2f}")
        return "\n".join(lines)


@dataclass
class LineChart:
    """Vertical ASCII line chart. Points plotted on a 2D grid of characters."""
    title: str
    points: list[DataPoint]
    width: int = 40
    height: int = 12

    def render(self) -> str:
        if not self.points:
            return f"{self.title}\n(no data)"
        n = len(self.points)
        values = [p.value for p in self.points]
        x_scale = LinearScale(0, max(1, n - 1), 0, self.width - 1)
        y_scale = LinearScale.fit(values, 0, self.height - 1, include_zero=False)
        grid = [[" "] * self.width for _ in range(self.height)]

        def plot(col: int, row: int, ch: str) -> None:
            if 0 <= col < self.width and 0 <= row < self.height:
                grid[row][col] = ch

        # Plot each point and a crude line segment to the previous one.
        prev: tuple[int, int] | None = None
        for i, p in enumerate(self.points):
            col = int(round(x_scale(i)))
            # y axis grows down; invert row index.
            row = self.height - 1 - int(round(y_scale(p.value)))
            plot(col, row, "●")
            if prev is not None:
                pc, pr = prev
                # Simple line: step along the longer axis.
                dx = col - pc
                dy = row - pr
                steps = max(abs(dx), abs(dy))
                if steps > 1:
                    for s in range(1, steps):
                        t = s / steps
                        ic = int(round(pc + dx * t))
                        ir = int(round(pr + dy * t))
                        if grid[ir][ic] == " ":
                            plot(ic, ir, "·")
            prev = (col, row)

        # Left axis with max/min labels.
        hi = y_scale.domain_max
        lo = y_scale.domain_min
        out = [self.title]
        for r in range(self.height):
            left = ""
            if r == 0:
                left = f"{hi:>7.2f} ┤"
            elif r == self.height - 1:
                left = f"{lo:>7.2f} ┤"
            else:
                left = "        │"
            out.append(left + "".join(grid[r]))
        out.append("        └" + "─" * self.width)
        xticks_line = ["        "]
        # Show first and last labels (others would crowd).
        first_label = self.points[0].label
        last_label = self.points[-1].label
        padding = " " * max(0, self.width - len(first_label) - len(last_label) - 1)
        out.append("         " + first_label + padding + last_label)
        return "\n".join(out)


# --- demo ---
if __name__ == "__main__":
    # Monthly revenue → bar chart
    monthly = [
        DataPoint("Jan", 12000), DataPoint("Feb", 15500),
        DataPoint("Mar", 18700), DataPoint("Apr", 14200),
        DataPoint("May", 21300), DataPoint("Jun", 19800),
    ]
    print(BarChart(title="Monthly Revenue ($)", points=monthly, width=30).render())
    print()

    # Daily temperatures → line chart
    temps = [
        DataPoint(f"D{i}", v) for i, v in
        enumerate([15.2, 16.1, 18.5, 22.0, 24.3, 23.7, 20.1,
                   17.8, 19.4, 21.9, 23.2, 25.1, 22.6, 18.3])
    ]
    print(LineChart(title="Two-Week Temperature (°C)", points=temps,
                    width=40, height=10).render())

    # Compose: the same data can feed either chart type.
    # The spec (which chart) is declarative; the renderer is pluggable.
    print()
    print("(same 14-day data as bars, truncated to first 7):")
    print(BarChart(title="First Week Temp (°C)", points=temps[:7], width=25).render())
