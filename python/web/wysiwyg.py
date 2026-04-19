"""WYSIWYG Editor — document model where edit and render are the same structure.

The essence of WYSIWYG is that there is no distinction between "source" and
"rendered output." The document IS the display. Operations on the document
produce the display directly; there is no parsing, no compilation, no
intermediate format.

The document is a list of **runs**, each a contiguous span of characters
sharing the same attribute set. Insertions, deletions, and style changes
reshape runs — splitting where attrs change, merging where adjacent runs
end up with identical attrs. Positions are user-facing character offsets;
runs are the implementation detail.
"""

from __future__ import annotations
from dataclasses import dataclass, field
from enum import Enum


class Attr(Enum):
    BOLD = "b"
    ITALIC = "i"
    UNDERLINE = "u"
    H1 = "h1"
    H2 = "h2"


@dataclass
class Run:
    text: str
    attrs: frozenset[Attr] = frozenset()


class Document:
    def __init__(self) -> None:
        self._runs: list[Run] = []

    def __len__(self) -> int:
        return sum(len(r.text) for r in self._runs)

    def insert(self, pos: int, text: str, attrs: frozenset[Attr] = frozenset()) -> None:
        if not (0 <= pos <= len(self)):
            raise IndexError(f"insert position {pos} out of range (len={len(self)})")
        left, right = self._split_at(pos)
        self._runs = left + [Run(text=text, attrs=attrs)] + right
        self._normalize()

    def delete(self, start: int, end: int) -> None:
        if not (0 <= start <= end <= len(self)):
            raise IndexError(f"delete range [{start}, {end}) invalid for len={len(self)}")
        if start == end:
            return
        left, rest = self._split_at(start)
        _, right = _split_runs(rest, end - start)
        self._runs = left + right
        self._normalize()

    def apply_style(self, start: int, end: int, attr: Attr) -> None:
        self._modify_attrs(start, end, lambda s: s | {attr})

    def remove_style(self, start: int, end: int, attr: Attr) -> None:
        self._modify_attrs(start, end, lambda s: s - {attr})

    def _modify_attrs(self, start: int, end: int, fn) -> None:
        if not (0 <= start <= end <= len(self)):
            raise IndexError(f"style range [{start}, {end}) invalid for len={len(self)}")
        if start == end:
            return
        left, rest = self._split_at(start)
        middle, right = _split_runs(rest, end - start)
        self._runs = left + [Run(text=r.text, attrs=frozenset(fn(set(r.attrs))))
                             for r in middle] + right
        self._normalize()

    def to_plain(self) -> str:
        return "".join(r.text for r in self._runs)

    def to_html(self) -> str:
        parts: list[str] = []
        # Stable attribute order: block-level first (outermost wrap), then inline.
        block_order = (Attr.H1, Attr.H2)
        inline_order = (Attr.BOLD, Attr.ITALIC, Attr.UNDERLINE)
        for r in self._runs:
            text = (r.text.replace("&", "&amp;")
                         .replace("<", "&lt;")
                         .replace(">", "&gt;"))
            open_tags: list[str] = []
            close_tags: list[str] = []
            for a in block_order:
                if a in r.attrs:
                    open_tags.append(f"<{a.value}>")
                    close_tags.insert(0, f"</{a.value}>")
            for a in inline_order:
                if a in r.attrs:
                    open_tags.append(f"<{a.value}>")
                    close_tags.insert(0, f"</{a.value}>")
            parts.append("".join(open_tags) + text + "".join(close_tags))
        return "".join(parts)

    def runs(self) -> list[Run]:
        return list(self._runs)

    def _split_at(self, pos: int) -> tuple[list[Run], list[Run]]:
        return _split_runs(self._runs, pos)

    def _normalize(self) -> None:
        out: list[Run] = []
        for r in self._runs:
            if not r.text:
                continue
            if out and out[-1].attrs == r.attrs:
                out[-1] = Run(text=out[-1].text + r.text, attrs=r.attrs)
            else:
                out.append(r)
        self._runs = out


def _split_runs(runs: list[Run], pos: int) -> tuple[list[Run], list[Run]]:
    left: list[Run] = []
    right: list[Run] = []
    seen = 0
    for r in runs:
        n = len(r.text)
        if pos >= seen + n:
            left.append(r)
        elif pos <= seen:
            right.append(r)
        else:
            split = pos - seen
            left.append(Run(text=r.text[:split], attrs=r.attrs))
            right.append(Run(text=r.text[split:], attrs=r.attrs))
        seen += n
    return left, right


if __name__ == "__main__":
    d = Document()
    d.insert(0, "Hello, world! Welcome to WYSIWYG.", frozenset())
    d.apply_style(0, 6, Attr.BOLD)              # "Hello,"
    d.apply_style(7, 12, Attr.ITALIC)           # "world"
    d.apply_style(14, 21, Attr.UNDERLINE)       # "Welcome"
    d.apply_style(25, 32, Attr.BOLD)            # "WYSIWYG"

    print(f"plain: {d.to_plain()}")
    print(f"runs:  {len(d.runs())}")
    print(f"html:  {d.to_html()}")

    # Demonstrate that edit-and-render are the same structure: every
    # operation immediately reshapes what to_html returns.
    d.insert(0, "Title: ", frozenset({Attr.H1}))
    print()
    print(f"after H1 insert:")
    print(f"html:  {d.to_html()}")
