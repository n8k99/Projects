"""Turtle Graphics — the Rosetta Stone finale.

Integer-degree heading (0..=359), integer-scaled sin/cos lookup (×10000),
vector-line canvas (no pixel rasterization until render), state stack for
nested procedures (PushState/PopState), Command ADT as program-as-data.
"""

from __future__ import annotations
from dataclasses import dataclass, field, replace
from enum import Enum
from typing import Union

from graphics.grayscale import Image, Rgb


_SIN_0_90 = [
    0, 175, 349, 523, 698, 872, 1045, 1219, 1392, 1564,
    1736, 1908, 2079, 2250, 2419, 2588, 2756, 2924, 3090, 3256,
    3420, 3584, 3746, 3907, 4067, 4226, 4384, 4540, 4695, 4848,
    5000, 5150, 5299, 5446, 5592, 5736, 5878, 6018, 6157, 6293,
    6428, 6561, 6691, 6820, 6947, 7071, 7193, 7314, 7431, 7547,
    7660, 7771, 7880, 7986, 8090, 8192, 8290, 8387, 8480, 8572,
    8660, 8746, 8829, 8910, 8988, 9063, 9135, 9205, 9272, 9336,
    9397, 9455, 9511, 9563, 9613, 9659, 9703, 9744, 9781, 9816,
    9848, 9877, 9903, 9925, 9945, 9962, 9976, 9986, 9994, 9998,
    10000,
]


def sin_table(angle: int) -> int:
    a = angle % 360
    if a <= 90: return _SIN_0_90[a]
    if a <= 180: return _SIN_0_90[180 - a]
    if a <= 270: return -_SIN_0_90[a - 180]
    return -_SIN_0_90[360 - a]


def cos_table(angle: int) -> int:
    return sin_table((angle + 90) % 360)


@dataclass
class Turtle:
    x: int = 0
    y: int = 0
    heading: int = 0  # 0..=359
    pen_down: bool = True
    pen_color: Rgb = field(default_factory=lambda: Rgb(0, 0, 0))
    pen_width: int = 1


@dataclass
class Line:
    x1: int
    y1: int
    x2: int
    y2: int
    color: Rgb
    width: int


class CommandKind(Enum):
    FORWARD = "forward"
    BACKWARD = "backward"
    TURN = "turn"
    SET_HEADING = "set_heading"
    PEN_UP = "pen_up"
    PEN_DOWN = "pen_down"
    SET_COLOR = "set_color"
    SET_PEN_WIDTH = "set_pen_width"
    PUSH_STATE = "push_state"
    POP_STATE = "pop_state"
    GOTO = "goto"


@dataclass(frozen=True)
class Command:
    kind: CommandKind
    n: int = 0
    deg: int = 0
    heading: int = 0
    color: Rgb = field(default_factory=lambda: Rgb(0, 0, 0))
    width: int = 1
    x: int = 0
    y: int = 0


def cmd_forward(n: int) -> Command: return Command(CommandKind.FORWARD, n=n)
def cmd_backward(n: int) -> Command: return Command(CommandKind.BACKWARD, n=n)
def cmd_turn(deg: int) -> Command: return Command(CommandKind.TURN, deg=deg)
def cmd_set_heading(h: int) -> Command: return Command(CommandKind.SET_HEADING, heading=h)
def cmd_pen_up() -> Command: return Command(CommandKind.PEN_UP)
def cmd_pen_down() -> Command: return Command(CommandKind.PEN_DOWN)
def cmd_set_color(c: Rgb) -> Command: return Command(CommandKind.SET_COLOR, color=c)
def cmd_set_pen_width(w: int) -> Command: return Command(CommandKind.SET_PEN_WIDTH, width=w)
def cmd_push_state() -> Command: return Command(CommandKind.PUSH_STATE)
def cmd_pop_state() -> Command: return Command(CommandKind.POP_STATE)
def cmd_goto(x: int, y: int) -> Command: return Command(CommandKind.GOTO, x=x, y=y)


@dataclass
class Canvas:
    lines: list[Line] = field(default_factory=list)
    turtle: Turtle = field(default_factory=Turtle)
    state_stack: list[Turtle] = field(default_factory=list)

    def step(self, cmd: Command) -> None:
        k = cmd.kind
        if k == CommandKind.FORWARD: self._move_by(cmd.n)
        elif k == CommandKind.BACKWARD: self._move_by(-cmd.n)
        elif k == CommandKind.TURN:
            self.turtle.heading = (self.turtle.heading + cmd.deg) % 360
        elif k == CommandKind.SET_HEADING:
            self.turtle.heading = cmd.heading % 360
        elif k == CommandKind.PEN_UP: self.turtle.pen_down = False
        elif k == CommandKind.PEN_DOWN: self.turtle.pen_down = True
        elif k == CommandKind.SET_COLOR: self.turtle.pen_color = cmd.color
        elif k == CommandKind.SET_PEN_WIDTH: self.turtle.pen_width = cmd.width
        elif k == CommandKind.PUSH_STATE:
            self.state_stack.append(replace(self.turtle))
        elif k == CommandKind.POP_STATE:
            if self.state_stack:
                self.turtle = self.state_stack.pop()
        elif k == CommandKind.GOTO:
            old_x, old_y = self.turtle.x, self.turtle.y
            self.turtle.x = cmd.x
            self.turtle.y = cmd.y
            if self.turtle.pen_down:
                self.lines.append(Line(old_x, old_y, cmd.x, cmd.y,
                                        self.turtle.pen_color,
                                        self.turtle.pen_width))

    def _move_by(self, distance: int) -> None:
        cos = cos_table(self.turtle.heading)
        sin = sin_table(self.turtle.heading)
        new_x = self.turtle.x + distance * cos // 10000
        new_y = self.turtle.y + distance * sin // 10000
        if self.turtle.pen_down:
            self.lines.append(Line(
                self.turtle.x, self.turtle.y, new_x, new_y,
                self.turtle.pen_color, self.turtle.pen_width))
        self.turtle.x = new_x
        self.turtle.y = new_y

    def execute(self, program: list[Command]) -> None:
        for cmd in program:
            self.step(cmd)


def demo() -> None:
    # Key-angle table
    print(f"sin(0)={sin_table(0)} cos(0)={cos_table(0)}")
    print(f"sin(90)={sin_table(90)} cos(90)={cos_table(90)}")
    print(f"sin(180)={sin_table(180)} cos(180)={cos_table(180)}")
    print(f"sin(270)={sin_table(270)} cos(270)={cos_table(270)}")
    print(f"sin(45)={sin_table(45)} cos(45)={cos_table(45)}")

    # Draw a square
    c = Canvas()
    program = [
        cmd_forward(100), cmd_turn(90),
        cmd_forward(100), cmd_turn(90),
        cmd_forward(100), cmd_turn(90),
        cmd_forward(100), cmd_turn(90),
    ]
    c.execute(program)
    print(f"square: turtle=({c.turtle.x}, {c.turtle.y}) heading={c.turtle.heading} lines={len(c.lines)}")

    # Nested procedure via state stack
    c2 = Canvas()
    c2.execute([
        cmd_forward(50),
        cmd_push_state(),
        cmd_turn(90), cmd_forward(30),
        cmd_pop_state(),
    ])
    print(f"after push/pop: turtle=({c2.turtle.x}, {c2.turtle.y}) heading={c2.turtle.heading}")
    print(f"lines drawn: {len(c2.lines)}")

    # 45-degree rotated square
    c3 = Canvas()
    c3.execute([cmd_turn(45)] + [cmd_forward(100), cmd_turn(90)] * 4)
    print(f"45-degree square: turtle=({c3.turtle.x}, {c3.turtle.y})")

    # Star via Logo-style loop
    c4 = Canvas()
    for _ in range(5):
        c4.execute([cmd_forward(100), cmd_turn(144)])
    print(f"five-pointed star: lines={len(c4.lines)} turtle=({c4.turtle.x}, {c4.turtle.y})")


if __name__ == "__main__":
    demo()
