"""Slide Show — presentation navigation with dual-track content.

Presentation holds an ordered list of slides + current_index (bounded).
Each slide has visible content (Heading/Bullet/Image/Code/Spacer) plus
speaker notes. Modes: SPEAKER (notes visible), AUDIENCE (hidden), HANDOUT.
"""

from __future__ import annotations
from dataclasses import dataclass, field
from enum import Enum


class ElementKind(Enum):
    HEADING = "heading"
    BULLET = "bullet"
    IMAGE = "image"
    SPACER = "spacer"
    CODE = "code"


@dataclass
class SlideElement:
    kind: ElementKind
    text: str = ""
    level: int = 1        # HEADING
    indent: int = 0       # BULLET
    src: str = ""         # IMAGE
    alt: str = ""         # IMAGE
    language: str = ""    # CODE
    lines: int = 0        # SPACER


class Transition(Enum):
    NONE = "none"
    FADE = "fade"
    SLIDE = "slide"
    DISSOLVE = "dissolve"


@dataclass
class Slide:
    title: str
    content: list[SlideElement] = field(default_factory=list)
    notes: str = ""
    transition: Transition = Transition.NONE

    def heading(self, text: str, level: int) -> Slide:
        self.content.append(SlideElement(ElementKind.HEADING, text=text, level=level))
        return self

    def bullet(self, text: str, indent: int) -> Slide:
        self.content.append(SlideElement(ElementKind.BULLET, text=text, indent=indent))
        return self

    def image(self, src: str, alt: str) -> Slide:
        self.content.append(SlideElement(ElementKind.IMAGE, src=src, alt=alt))
        return self

    def code(self, text: str, language: str) -> Slide:
        self.content.append(SlideElement(ElementKind.CODE, text=text, language=language))
        return self

    def with_notes(self, notes: str) -> Slide:
        self.notes = notes
        return self


class PresentationMode(Enum):
    SPEAKER = "speaker"
    AUDIENCE = "audience"
    HANDOUT = "handout"


@dataclass
class Presentation:
    title: str
    slides: list[Slide] = field(default_factory=list)
    current_index: int = 0

    def add_slide(self, s: Slide) -> None:
        self.slides.append(s)

    def slide_count(self) -> int:
        return len(self.slides)

    def current(self) -> Slide | None:
        if 0 <= self.current_index < len(self.slides):
            return self.slides[self.current_index]
        return None

    def advance(self) -> bool:
        if self.current_index + 1 < len(self.slides):
            self.current_index += 1
            return True
        return False

    def back(self) -> bool:
        if self.current_index > 0:
            self.current_index -= 1
            return True
        return False

    def goto(self, index: int) -> bool:
        if 0 <= index < len(self.slides):
            self.current_index = index
            return True
        return False

    def progress(self) -> tuple[int, int]:
        if not self.slides:
            return (0, 0)
        return (self.current_index + 1, len(self.slides))

    def render_current(self, mode: PresentationMode) -> str:
        s = self.current()
        if s is None:
            return ""
        return render_slide(s, mode)

    def render_handout(self) -> str:
        out = [f"# {self.title}", ""]
        for i, s in enumerate(self.slides):
            out.append(f"--- Slide {i + 1} ---")
            out.append(render_slide(s, PresentationMode.HANDOUT))
            out.append("")
        return "\n".join(out)

    def search(self, needle: str) -> list[int]:
        lc = needle.lower()
        matches = []
        for i, s in enumerate(self.slides):
            if lc in s.title.lower():
                matches.append(i)
            elif any(lc in _element_text(e).lower() for e in s.content):
                matches.append(i)
        return matches

    def transition_stats(self) -> dict[str, int]:
        out: dict[str, int] = {}
        for s in self.slides:
            out[s.transition.value] = out.get(s.transition.value, 0) + 1
        return out


def _element_text(e: SlideElement) -> str:
    if e.kind == ElementKind.HEADING: return e.text
    if e.kind == ElementKind.BULLET: return e.text
    if e.kind == ElementKind.IMAGE: return e.alt
    if e.kind == ElementKind.CODE: return e.text
    return ""


def _render_element(e: SlideElement) -> str:
    if e.kind == ElementKind.HEADING:
        hashes = "#" * max(1, min(6, e.level))
        return f"{hashes} {e.text}"
    if e.kind == ElementKind.BULLET:
        pad = "  " * e.indent
        return f"{pad}- {e.text}"
    if e.kind == ElementKind.IMAGE:
        return f"![{e.alt}]({e.src})"
    if e.kind == ElementKind.CODE:
        return f"```{e.language}\n{e.text}\n```"
    return "\n" * e.lines


def render_slide(slide: Slide, mode: PresentationMode) -> str:
    lines = [slide.title, "=" * min(80, len(slide.title))]
    for e in slide.content:
        lines.append(_render_element(e))
    if mode in (PresentationMode.SPEAKER, PresentationMode.HANDOUT) and slide.notes:
        lines.append("")
        lines.append("[Speaker notes]")
        lines.append(slide.notes)
    return "\n".join(lines) + "\n"


def demo() -> None:
    p = Presentation("My Talk")
    p.add_slide(
        Slide("Introduction")
            .heading("Welcome", 1)
            .bullet("Agenda", 0)
            .bullet("Items", 1)
            .with_notes("Remember to pause for laughs.")
    )
    p.add_slide(
        Slide("The Problem")
            .heading("Pain points", 1)
            .bullet("Thing one", 0)
            .bullet("Thing two", 0)
            .with_notes("Be careful not to get political.")
    )

    print("=== audience view ===")
    print(p.render_current(PresentationMode.AUDIENCE))
    print("=== speaker view ===")
    print(p.render_current(PresentationMode.SPEAKER))

    p.advance()
    print(f"\nprogress: {p.progress()}")
    print(f"current: {p.current().title}")

    print(f"\nsearch 'pain': slides {p.search('pain')}")


if __name__ == "__main__":
    demo()
