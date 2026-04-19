"""Page Scraper — parse HTML-like markup into a tree, query with selectors.

The inverse of G069's render-to-HTML. Parsing is forgiving: real HTML is
messy, and a scraper that refuses malformed input is useless. Unknown
constructs become text; stray close tags are dropped; the parser recovers
at the next valid boundary.

Selectors: CSS subset covering 90% of real scraping — tag names, ``.class``,
``#id``, combinations like ``tag.class``, and descendant combination via
whitespace (``article p`` = any ``p`` inside any ``article``).
"""

from __future__ import annotations
from dataclasses import dataclass, field


VOID_TAGS = {"area", "base", "br", "col", "embed", "hr", "img",
             "input", "link", "meta", "source", "track", "wbr"}


@dataclass
class Text:
    text: str


@dataclass
class Element:
    tag: str
    attrs: dict[str, str] = field(default_factory=dict)
    children: list = field(default_factory=list)


Node = Element | Text


class _Parser:
    def __init__(self, s: str) -> None:
        self.s = s
        self.i = 0

    def eof(self) -> bool:
        return self.i >= len(self.s)

    def peek(self, n: int = 1) -> str:
        return self.s[self.i:self.i + n]

    def starts_with(self, prefix: str) -> bool:
        return self.s.startswith(prefix, self.i)

    def consume_while(self, pred) -> str:
        start = self.i
        while self.i < len(self.s) and pred(self.s[self.i]):
            self.i += 1
        return self.s[start:self.i]

    def consume_ws(self) -> None:
        self.consume_while(str.isspace)

    def parse_nodes(self, parent_tag: str | None = None) -> list[Node]:
        out: list[Node] = []
        while not self.eof():
            if parent_tag is not None and self.starts_with(f"</{parent_tag}"):
                break
            if self.starts_with("</"):
                # stray close tag — skip to '>'
                self.consume_while(lambda c: c != ">")
                if not self.eof(): self.i += 1
                continue
            if self.peek() == "<" and self.i + 1 < len(self.s) and self.s[self.i + 1].isalpha():
                node = self.parse_element()
                if node:
                    out.append(node)
            else:
                t = self.consume_while(lambda c: c != "<")
                if t:
                    out.append(Text(text=t))
        return out

    def parse_element(self) -> Element | None:
        assert self.s[self.i] == "<"
        self.i += 1
        tag = self.consume_while(lambda c: c.isalnum() or c == "-").lower()
        if not tag:
            return None
        attrs = self.parse_attrs()
        self_close = False
        if self.starts_with("/>"):
            self.i += 2
            self_close = True
        elif self.starts_with(">"):
            self.i += 1
        else:
            return None
        if self_close or tag in VOID_TAGS:
            return Element(tag=tag, attrs=attrs, children=[])
        children = self.parse_nodes(parent_tag=tag)
        if self.starts_with(f"</{tag}"):
            self.consume_while(lambda c: c != ">")
            if not self.eof(): self.i += 1
        return Element(tag=tag, attrs=attrs, children=children)

    def parse_attrs(self) -> dict[str, str]:
        attrs: dict[str, str] = {}
        while True:
            self.consume_ws()
            if self.eof() or self.peek() in (">", "/"):
                return attrs
            name = self.consume_while(lambda c: c.isalnum() or c in "-_").lower()
            if not name:
                self.i += 1
                continue
            self.consume_ws()
            value = ""
            if self.starts_with("="):
                self.i += 1
                self.consume_ws()
                if not self.eof() and self.peek() in ("\"", "'"):
                    q = self.peek()
                    self.i += 1
                    value = self.consume_while(lambda c: c != q)
                    if not self.eof(): self.i += 1
                else:
                    value = self.consume_while(
                        lambda c: not c.isspace() and c != ">" and c != "/")
            attrs[name] = value


def parse(html: str) -> list[Node]:
    return _Parser(html).parse_nodes()


@dataclass
class _Simple:
    tag: str | None = None
    cls: str | None = None
    node_id: str | None = None


def _parse_simple(s: str) -> _Simple:
    simp = _Simple()
    buf = ""
    kind = " "

    def flush() -> None:
        nonlocal buf, kind
        if not buf:
            return
        if kind == ".":
            simp.cls = buf
        elif kind == "#":
            simp.node_id = buf
        else:
            simp.tag = buf.lower()
        buf = ""

    for c in s:
        if c in ".#":
            flush()
            kind = c
        else:
            buf += c
    flush()
    return simp


def _matches(simp: _Simple, el: Element) -> bool:
    if simp.tag and simp.tag != el.tag:
        return False
    if simp.node_id and el.attrs.get("id") != simp.node_id:
        return False
    if simp.cls:
        if simp.cls not in el.attrs.get("class", "").split():
            return False
    return True


def select(nodes: list[Node], selector: str) -> list[Element]:
    chain = [_parse_simple(t) for t in selector.split()]
    out: list[Element] = []

    def recurse(ns: list[Node], idx: int) -> None:
        for n in ns:
            if not isinstance(n, Element):
                continue
            match = _matches(chain[idx], n)
            if match and idx == len(chain) - 1:
                out.append(n)
                recurse(n.children, idx)
            elif match:
                recurse(n.children, idx + 1)
            else:
                recurse(n.children, idx)

    recurse(nodes, 0)
    return out


def text_of(node: Node) -> str:
    if isinstance(node, Text):
        return node.text
    return "".join(text_of(c) for c in node.children)


if __name__ == "__main__":
    html = """
    <article id="main">
      <h1>The Rosetta Stone</h1>
      <p class="lead">A corpus that demonstrates what InnateScript is.</p>
      <ul>
        <li><a href="/numbers">Numbers</a></li>
        <li><a href="/text">Text</a></li>
        <li><a href="/classes" class="featured">Classes</a></li>
      </ul>
    </article>
    """
    nodes = parse(html)
    print(f"title: {text_of(select(nodes, 'h1')[0]).strip()}")
    print(f"lead:  {text_of(select(nodes, 'p.lead')[0]).strip()}")
    print("links:")
    for a in select(nodes, "ul a"):
        href = a.attrs.get("href", "")
        print(f"  {text_of(a):10s} -> {href}")
    print(f"featured: {text_of(select(nodes, '.featured')[0]).strip()}")
