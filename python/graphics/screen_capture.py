"""Screen Capture Program — region + delay FSM + annotations + ring-buffer.

Region ADT: FullScreen | Rect(x,y,w,h) | Window(id).
Delay FSM: Idle → Armed → Counting → Capturing → Captured.
Annotations: Rectangle / Highlight(alpha-blend) / Text / Arrow (Bresenham).
History: fixed-size ring buffer; oldest evicted when full.
"""

from __future__ import annotations
from dataclasses import dataclass, field
from enum import Enum
from typing import Callable, Optional, Union

from graphics.grayscale import Image, Rgb


class RegionKind(Enum):
    FULL_SCREEN = "full_screen"
    RECT = "rect"
    WINDOW = "window"


@dataclass(frozen=True)
class CaptureRegion:
    kind: RegionKind
    x: int = 0
    y: int = 0
    width: int = 0
    height: int = 0
    window_id: int = 0

    @classmethod
    def full_screen(cls) -> CaptureRegion:
        return cls(RegionKind.FULL_SCREEN)

    @classmethod
    def rect(cls, x: int, y: int, w: int, h: int) -> CaptureRegion:
        return cls(RegionKind.RECT, x=x, y=y, width=w, height=h)

    @classmethod
    def window(cls, window_id: int) -> CaptureRegion:
        return cls(RegionKind.WINDOW, window_id=window_id)


class AnnotationKind(Enum):
    RECTANGLE = "rectangle"
    HIGHLIGHT = "highlight"
    TEXT = "text"
    ARROW = "arrow"


@dataclass(frozen=True)
class Annotation:
    kind: AnnotationKind
    x: int = 0
    y: int = 0
    width: int = 0
    height: int = 0
    x1: int = 0
    y1: int = 0
    x2: int = 0
    y2: int = 0
    color: Rgb = field(default_factory=lambda: Rgb(0, 0, 0))
    stroke: int = 1
    opacity: int = 255
    text: str = ""


def ann_rect(x: int, y: int, w: int, h: int, color: Rgb, stroke: int = 1) -> Annotation:
    return Annotation(AnnotationKind.RECTANGLE, x=x, y=y, width=w, height=h,
                       color=color, stroke=stroke)

def ann_highlight(x: int, y: int, w: int, h: int, color: Rgb, opacity: int) -> Annotation:
    return Annotation(AnnotationKind.HIGHLIGHT, x=x, y=y, width=w, height=h,
                       color=color, opacity=opacity)

def ann_text(x: int, y: int, text: str, color: Rgb) -> Annotation:
    return Annotation(AnnotationKind.TEXT, x=x, y=y, text=text, color=color)

def ann_arrow(x1: int, y1: int, x2: int, y2: int, color: Rgb) -> Annotation:
    return Annotation(AnnotationKind.ARROW, x1=x1, y1=y1, x2=x2, y2=y2, color=color)


@dataclass
class CaptureSpec:
    region: CaptureRegion
    delay_ms: int = 0
    include_cursor: bool = False
    annotations: list[Annotation] = field(default_factory=list)


class RecorderState(Enum):
    IDLE = "idle"
    ARMED = "armed"
    COUNTING = "counting"
    CAPTURING = "capturing"
    CAPTURED = "captured"


@dataclass
class Capture:
    id: int
    spec: CaptureSpec
    image: Image
    captured_at_ms: int


class RecorderEventKind(Enum):
    ARMED = "armed"
    COUNTDOWN_TICK = "countdown_tick"
    CAPTURED = "captured"
    EVICTED = "evicted"
    CAPTURE_FAILED = "capture_failed"


@dataclass
class RecorderEvent:
    kind: RecorderEventKind
    id: int
    detail: str = ""


@dataclass
class Recorder:
    history_max: int
    state: RecorderState = RecorderState.IDLE
    pending_spec: Optional[CaptureSpec] = None
    pending_id: int = 0
    countdown_remaining_ms: int = 0
    history: list[Capture] = field(default_factory=list)
    elapsed_ms: int = 0
    events: list[RecorderEvent] = field(default_factory=list)
    _next_id: int = 1

    def arm(self, spec: CaptureSpec) -> int:
        id = self._next_id
        self._next_id += 1
        self.pending_id = id
        self.countdown_remaining_ms = spec.delay_ms
        delay = spec.delay_ms
        self.pending_spec = spec
        self.events.append(RecorderEvent(RecorderEventKind.ARMED, id,
                                          f"delay={delay}ms"))
        self.state = (RecorderState.CAPTURING if delay == 0
                       else RecorderState.COUNTING)
        return id

    def tick(self, ms: int) -> None:
        self.elapsed_ms += ms
        if self.state == RecorderState.COUNTING:
            if self.countdown_remaining_ms > ms:
                self.countdown_remaining_ms -= ms
            else:
                self.countdown_remaining_ms = 0
                self.state = RecorderState.CAPTURING
            self.events.append(RecorderEvent(
                RecorderEventKind.COUNTDOWN_TICK, self.pending_id,
                f"remaining={self.countdown_remaining_ms}ms"))

    def capture(self, screen: Image,
                 window_lookup: Callable[[int], Optional[Image]]) -> Optional[Capture]:
        if self.state != RecorderState.CAPTURING:
            return None
        spec = self.pending_spec
        if spec is None:
            return None
        self.pending_spec = None
        id = self.pending_id

        region = spec.region
        if region.kind == RegionKind.FULL_SCREEN:
            base = Image(screen.width, screen.height, list(screen.pixels))
        elif region.kind == RegionKind.RECT:
            if region.x + region.width > screen.width or region.y + region.height > screen.height:
                self.events.append(RecorderEvent(
                    RecorderEventKind.CAPTURE_FAILED, id, "rect out of bounds"))
                self.state = RecorderState.IDLE
                return None
            base = _crop(screen, region.x, region.y, region.width, region.height)
        elif region.kind == RegionKind.WINDOW:
            win = window_lookup(region.window_id)
            if win is None:
                self.events.append(RecorderEvent(
                    RecorderEventKind.CAPTURE_FAILED, id,
                    f"window {region.window_id} not found"))
                self.state = RecorderState.IDLE
                return None
            base = win
        else:
            raise AssertionError("unreachable")

        annotated = apply_annotations(base, spec.annotations)
        capture = Capture(id=id, spec=spec, image=annotated,
                           captured_at_ms=self.elapsed_ms)
        self.history.append(capture)
        if len(self.history) > self.history_max:
            evicted = self.history.pop(0)
            self.events.append(RecorderEvent(RecorderEventKind.EVICTED, evicted.id))
        self.events.append(RecorderEvent(
            RecorderEventKind.CAPTURED, id, f"at={self.elapsed_ms}ms"))
        self.state = RecorderState.CAPTURED
        return capture

    def reset(self) -> None:
        self.state = RecorderState.IDLE
        self.pending_spec = None
        self.pending_id = 0
        self.countdown_remaining_ms = 0


def apply_annotations(base: Image, annotations: list[Annotation]) -> Image:
    img = Image(base.width, base.height, list(base.pixels))
    for ann in annotations:
        _apply_one(img, ann)
    return img


def _apply_one(img: Image, ann: Annotation) -> None:
    k = ann.kind
    if k == AnnotationKind.RECTANGLE:
        _draw_rect_outline(img, ann.x, ann.y, ann.width, ann.height,
                            ann.color, max(1, ann.stroke))
    elif k == AnnotationKind.HIGHLIGHT:
        _blend_rect(img, ann.x, ann.y, ann.width, ann.height,
                     ann.color, ann.opacity)
    elif k == AnnotationKind.TEXT:
        if 0 <= ann.x < img.width and 0 <= ann.y < img.height:
            img.pixels[ann.y * img.width + ann.x] = ann.color
    elif k == AnnotationKind.ARROW:
        _draw_line(img, ann.x1, ann.y1, ann.x2, ann.y2, ann.color)


def _set_clamped(img: Image, x: int, y: int, color: Rgb) -> None:
    if 0 <= x < img.width and 0 <= y < img.height:
        img.pixels[y * img.width + x] = color


def _blend(a: int, b: int, alpha: int) -> int:
    return (a * (255 - alpha) + b * alpha) // 255


def _blend_rect(img: Image, x: int, y: int, w: int, h: int,
                 color: Rgb, opacity: int) -> None:
    for dy in range(h):
        for dx in range(w):
            px = x + dx
            py = y + dy
            if px >= img.width or py >= img.height:
                continue
            cur = img.pixels[py * img.width + px]
            blended = Rgb(
                _blend(cur.r, color.r, opacity),
                _blend(cur.g, color.g, opacity),
                _blend(cur.b, color.b, opacity),
            )
            img.pixels[py * img.width + px] = blended


def _draw_rect_outline(img: Image, x: int, y: int, w: int, h: int,
                        color: Rgb, stroke: int) -> None:
    for s in range(stroke):
        for dx in range(w):
            _set_clamped(img, x + dx, y + s, color)
            if h > s:
                _set_clamped(img, x + dx, y + h - 1 - s, color)
        for dy in range(h):
            _set_clamped(img, x + s, y + dy, color)
            if w > s:
                _set_clamped(img, x + w - 1 - s, y + dy, color)


def _draw_line(img: Image, x1: int, y1: int, x2: int, y2: int, color: Rgb) -> None:
    x = x1
    y = y1
    dx = abs(x2 - x)
    dy = -abs(y2 - y)
    sx = 1 if x < x2 else -1
    sy = 1 if y < y2 else -1
    err = dx + dy
    while True:
        if x >= 0 and y >= 0:
            _set_clamped(img, x, y, color)
        if x == x2 and y == y2:
            break
        e2 = 2 * err
        if e2 >= dy:
            err += dy
            x += sx
        if e2 <= dx:
            err += dx
            y += sy


def _crop(img: Image, x0: int, y0: int, w: int, h: int) -> Image:
    pixels = []
    for y in range(h):
        for x in range(w):
            pixels.append(img.pixels[(y0 + y) * img.width + (x0 + x)])
    return Image(w, h, pixels)


def demo() -> None:
    rec = Recorder(history_max=3)
    # delayed fullscreen capture
    rec.arm(CaptureSpec(region=CaptureRegion.full_screen(), delay_ms=1000))
    print(f"state: {rec.state.value} remaining: {rec.countdown_remaining_ms}ms")
    rec.tick(400)
    print(f"after 400ms tick: state: {rec.state.value} remaining: {rec.countdown_remaining_ms}ms")
    rec.tick(700)
    print(f"after 700ms tick: state: {rec.state.value} remaining: {rec.countdown_remaining_ms}ms")

    screen = Image(10, 8, [Rgb(100, 100, 100) for _ in range(80)])
    cap = rec.capture(screen, lambda _: None)
    print(f"captured: id={cap.id} image={cap.image.width}x{cap.image.height}")

    # immediate rect capture with annotations
    rec.reset()
    spec = CaptureSpec(
        region=CaptureRegion.rect(2, 1, 6, 5),
        delay_ms=0,
        annotations=[
            ann_highlight(0, 0, 6, 5, Rgb(255, 255, 0), opacity=128),
            ann_rect(1, 1, 4, 3, Rgb(255, 0, 0), stroke=1),
            ann_arrow(0, 0, 5, 4, Rgb(0, 255, 0)),
        ],
    )
    rec.arm(spec)
    cap2 = rec.capture(screen, lambda _: None)
    print(f"annotated: {cap2.image.width}x{cap2.image.height}")
    p = cap2.image.pixels[0]
    print(f"pixels[0]=({p.r},{p.g},{p.b})")  # green arrow start

    # ring buffer
    for _ in range(5):
        rec.reset()
        rec.arm(CaptureSpec(region=CaptureRegion.full_screen(), delay_ms=0))
        rec.capture(screen, lambda _: None)
    print(f"history size: {len(rec.history)}")
    print(f"history ids: {[c.id for c in rec.history]}")


if __name__ == "__main__":
    demo()
