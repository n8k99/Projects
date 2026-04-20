"""PDF Generator — document pagination via block flow layout.

Page = list of (line, y, is_heading). Layout: blocks stack top-to-bottom;
overflow starts a new page. Paragraphs wrap at word boundaries.
No real PDF bytes — a pure layout model any renderer can emit.
"""

from __future__ import annotations
from dataclasses import dataclass, field
from enum import Enum


class BlockKind(Enum):
    HEADING = "heading"
    PARAGRAPH = "paragraph"
    SPACER = "spacer"
    PAGE_BREAK = "page_break"


@dataclass
class Block:
    kind: BlockKind
    text: str = ""
    level: int = 1
    height: int = 0


@dataclass
class PageSize:
    width_chars: int
    height_lines: int


A4_LIKE = PageSize(width_chars=80, height_lines=66)


@dataclass
class RenderedLine:
    text: str
    y: int
    is_heading: bool


@dataclass
class Page:
    lines: list[RenderedLine] = field(default_factory=list)
    number: int = 1


@dataclass
class Document:
    page_size: PageSize
    blocks: list[Block] = field(default_factory=list)

    def heading(self, text: str, level: int = 1) -> Document:
        self.blocks.append(Block(BlockKind.HEADING, text=text, level=level))
        return self

    def paragraph(self, text: str) -> Document:
        self.blocks.append(Block(BlockKind.PARAGRAPH, text=text))
        return self

    def spacer(self, height: int) -> Document:
        self.blocks.append(Block(BlockKind.SPACER, height=height))
        return self

    def page_break(self) -> Document:
        self.blocks.append(Block(BlockKind.PAGE_BREAK))
        return self

    def render(self) -> list[Page]:
        pages: list[Page] = []
        current = Page(number=1)
        y = 0

        def push_page() -> None:
            nonlocal current, y
            pages.append(current)
            current = Page(number=current.number + 1)
            y = 0

        for block in self.blocks:
            if block.kind == BlockKind.PAGE_BREAK:
                if current.lines:
                    push_page()
            elif block.kind == BlockKind.SPACER:
                if y + block.height > self.page_size.height_lines:
                    if current.lines:
                        push_page()
                else:
                    y += block.height
            elif block.kind == BlockKind.HEADING:
                h_height = 2
                if y + h_height > self.page_size.height_lines:
                    if current.lines:
                        push_page()
                rendered = {1: f"# {block.text}", 2: f"## {block.text}"}.get(
                    block.level, f"### {block.text}"
                )
                current.lines.append(RenderedLine(rendered, y, True))
                y += h_height
            elif block.kind == BlockKind.PARAGRAPH:
                wrapped = wrap_text(block.text, self.page_size.width_chars)
                for line in wrapped:
                    if y >= self.page_size.height_lines:
                        if current.lines:
                            push_page()
                    current.lines.append(RenderedLine(line, y, False))
                    y += 1
                if y < self.page_size.height_lines:
                    y += 1

        if current.lines:
            pages.append(current)
        return pages

    def page_count(self) -> int:
        return len(self.render())


def wrap_text(text: str, width: int) -> list[str]:
    lines: list[str] = []
    current = ""
    for word in text.split():
        if not current:
            current = word
        elif len(current) + 1 + len(word) <= width:
            current += " " + word
        else:
            lines.append(current)
            current = word
    if current:
        lines.append(current)
    if not lines:
        lines.append("")
    return lines


def demo() -> None:
    doc = Document(page_size=PageSize(width_chars=40, height_lines=10))
    doc.heading("Rosetta Stone", 1)
    doc.paragraph(
        "The quick brown fox jumps over the lazy dog. "
        "Pack my box with five dozen liquor jugs. "
        "How vexingly quick daft zebras jump."
    )
    doc.heading("Section 1", 2)
    doc.paragraph("Short second paragraph about layout.")
    doc.page_break()
    doc.heading("New Page", 1)
    doc.paragraph("Content that starts on a fresh page.")

    pages = doc.render()
    print(f"=== {len(pages)} page(s) rendered ===\n")
    for p in pages:
        print(f"--- Page {p.number} ---")
        for line in p.lines:
            marker = "H" if line.is_heading else " "
            print(f"  [{marker}] y={line.y:2}  {line.text}")
        print()


if __name__ == "__main__":
    demo()
