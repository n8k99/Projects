"""Code Snippet Manager — templated code fragments with tab-stop expansion.

${N:default}, ${N}, $N, $0 placeholder syntax. Expansion returns text +
ordered tab stops (non-zero ascending, then 0 last). Library indexes by
trigger, language, tags.
"""

from __future__ import annotations
from dataclasses import dataclass, field


@dataclass
class TabStop:
    number: int
    offset: int
    length: int


@dataclass
class Expansion:
    text: str
    stops: list[TabStop]


@dataclass
class Snippet:
    trigger: str
    language: str
    body: str
    tags: list[str] = field(default_factory=list)
    description: str = ""

    def expand(self) -> Expansion:
        return _parse_and_expand(self.body)


class SnippetLibrary:
    def __init__(self) -> None:
        self.snippets: list[Snippet] = []

    def add(self, s: Snippet) -> None:
        self.snippets.append(s)

    def find_by_trigger(self, trigger: str, language: str) -> Snippet | None:
        for s in self.snippets:
            if s.trigger == trigger and s.language == language:
                return s
        return None

    def find_by_language(self, language: str) -> list[Snippet]:
        return sorted(
            [s for s in self.snippets if s.language == language],
            key=lambda s: s.trigger,
        )

    def find_by_tag(self, tag: str) -> list[Snippet]:
        return sorted(
            [s for s in self.snippets if tag in s.tags],
            key=lambda s: s.trigger,
        )

    def __len__(self) -> int:
        return len(self.snippets)


def _parse_and_expand(body: str) -> Expansion:
    out = []
    stops: list[TabStop] = []
    i = 0
    n = len(body)
    while i < n:
        if body[i] == "$" and i + 1 < n:
            if body[i + 1] == "{":
                close = _find_matching(body, i + 1)
                if close is not None:
                    inside = body[i + 2:close]
                    if ":" in inside:
                        num_str, default = inside.split(":", 1)
                    else:
                        num_str, default = inside, ""
                    try:
                        number = int(num_str)
                    except ValueError:
                        number = 0
                    stops.append(TabStop(number, len("".join(out)), len(default)))
                    out.append(default)
                    i = close + 1
                    continue
            elif body[i + 1].isdigit():
                j = i + 1
                while j < n and body[j].isdigit():
                    j += 1
                number = int(body[i + 1:j])
                stops.append(TabStop(number, len("".join(out)), 0))
                i = j
                continue
        out.append(body[i])
        i += 1

    def sort_key(stop: TabStop) -> tuple[int, int]:
        # 0 sorts last; non-zero sort ascending.
        return (1 if stop.number == 0 else 0, stop.number)

    stops.sort(key=sort_key)
    return Expansion("".join(out), stops)


def _find_matching(body: str, open_idx: int) -> int | None:
    depth = 1
    i = open_idx + 1
    while i < len(body):
        if body[i] == "{":
            depth += 1
        elif body[i] == "}":
            depth -= 1
            if depth == 0:
                return i
        i += 1
    return None


def group_stops_by_number(stops: list[TabStop]) -> dict[int, list[TabStop]]:
    out: dict[int, list[TabStop]] = {}
    for s in stops:
        out.setdefault(s.number, []).append(s)
    return out


def demo() -> None:
    lib = SnippetLibrary()
    lib.add(Snippet(
        trigger="for", language="python", tags=["loop"],
        body="for ${1:item} in ${2:collection}:\n    ${3:pass}\n$0",
        description="for loop",
    ))
    lib.add(Snippet(
        trigger="fn", language="rust", tags=["function"],
        body="fn ${1:name}(${2}) -> ${3:()} {\n    $0\n}",
        description="function definition",
    ))

    for s in [lib.find_by_trigger("for", "python"), lib.find_by_trigger("fn", "rust")]:
        e = s.expand()
        print(f"=== {s.trigger} ({s.language}) ===")
        print(f"expanded:\n{e.text}")
        print("stops:")
        for st in e.stops:
            print(f"  #{st.number} @offset={st.offset} length={st.length}")
        print()


if __name__ == "__main__":
    demo()
