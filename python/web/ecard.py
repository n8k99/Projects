"""E-Card Generator — template + data rendering with slot validation.

First Rosetta Stone project where content and presentation are separated
by a template. A `Template` declares the slots it expects (name, kind,
required) and owns the rendering surface (HTML with `{{slot_name}}`
placeholders). A `Card` is a filled-in instance. Rendering substitutes
values into placeholders.

Validation happens at card creation, not at render time — stored cards
are always valid; rendering can never fail on a stored card.
"""

from __future__ import annotations
from dataclasses import dataclass, field
from enum import Enum


class SlotKind(Enum):
    TEXT = "text"
    LONG_TEXT = "long_text"
    IMAGE_URL = "image_url"
    COLOR = "color"
    DATE = "date"


@dataclass
class Slot:
    name: str
    kind: SlotKind
    required: bool = False
    default: str | None = None


@dataclass
class Template:
    id: int
    name: str
    category: str
    slots: list[Slot]
    html_template: str

    def slot(self, name: str) -> Slot | None:
        return next((s for s in self.slots if s.name == name), None)

    def slot_names(self) -> set[str]:
        return {s.name for s in self.slots}


@dataclass
class Card:
    id: int
    template_id: int
    filled: dict[str, str]
    recipient: str
    sender: str
    created_at_ms: int


class ValidationError(Exception):
    pass


class UnknownTemplate(ValidationError):
    def __init__(self, tid: int) -> None:
        super().__init__(f"unknown template: {tid}")
        self.template_id = tid


class MissingRequired(ValidationError):
    def __init__(self, name: str) -> None:
        super().__init__(f"missing required slot: {name}")
        self.slot_name = name


class UnknownSlot(ValidationError):
    def __init__(self, name: str) -> None:
        super().__init__(f"unknown slot: {name}")
        self.slot_name = name


class Gallery:
    def __init__(self) -> None:
        self._templates: list[Template] = []
        self._cards: list[Card] = []
        self._next_template_id = 1
        self._next_card_id = 1

    def add_template(self, name: str, category: str,
                     slots: list[Slot], html_template: str) -> int:
        tid = self._next_template_id
        self._next_template_id += 1
        self._templates.append(Template(
            id=tid, name=name, category=category,
            slots=list(slots), html_template=html_template,
        ))
        return tid

    def template(self, tid: int) -> Template | None:
        return next((t for t in self._templates if t.id == tid), None)

    def templates(self) -> list[Template]:
        return list(self._templates)

    def templates_in_category(self, category: str) -> list[Template]:
        return [t for t in self._templates if t.category == category]

    def create_card(self, template_id: int, filled: dict[str, str],
                    recipient: str, sender: str, now_ms: int) -> int:
        template = self.template(template_id)
        if template is None:
            raise UnknownTemplate(template_id)

        known = template.slot_names()
        for key in filled:
            if key not in known:
                raise UnknownSlot(key)

        for slot in template.slots:
            if slot.required and slot.name not in filled:
                raise MissingRequired(slot.name)

        final_filled = dict(filled)
        for slot in template.slots:
            if slot.name not in final_filled and slot.default is not None:
                final_filled[slot.name] = slot.default

        cid = self._next_card_id
        self._next_card_id += 1
        self._cards.append(Card(
            id=cid, template_id=template_id, filled=final_filled,
            recipient=recipient, sender=sender, created_at_ms=now_ms,
        ))
        return cid

    def card(self, cid: int) -> Card | None:
        return next((c for c in self._cards if c.id == cid), None)

    def cards(self) -> list[Card]:
        return list(self._cards)

    def cards_by_template(self, template_id: int) -> list[Card]:
        return [c for c in self._cards if c.template_id == template_id]

    def render(self, card_id: int) -> str | None:
        card = self.card(card_id)
        if card is None:
            return None
        template = self.template(card.template_id)
        if template is None:
            return None
        out = template.html_template
        for k, v in card.filled.items():
            out = out.replace(f"{{{{{k}}}}}", v)
        out = out.replace("{{_recipient}}", card.recipient)
        out = out.replace("{{_sender}}", card.sender)
        return out

    def render_plain(self, card_id: int) -> str | None:
        html = self.render(card_id)
        if html is None:
            return None
        return _strip_tags(html)


def _strip_tags(s: str) -> str:
    out = []
    in_tag = False
    for c in s:
        if c == "<":
            in_tag = True
        elif c == ">":
            in_tag = False
        elif not in_tag:
            out.append(c)
    return "".join(out)


if __name__ == "__main__":
    g = Gallery()

    birthday_slots = [
        Slot("name", SlotKind.TEXT, required=True),
        Slot("age", SlotKind.TEXT, default="another year older"),
        Slot("color", SlotKind.COLOR, default="#ffcc00"),
    ]
    birthday_html = (
        '<div style="background:{{color}};padding:20px;border-radius:10px">'
        '<h1>Happy Birthday, {{name}}!</h1>'
        '<p>Wishing you — {{age}}.</p>'
        '<footer>From {{_sender}} to {{_recipient}}</footer>'
        '</div>'
    )
    birthday_tid = g.add_template("Classic Birthday", "birthday",
                                   birthday_slots, birthday_html)

    anniversary_slots = [
        Slot("years", SlotKind.TEXT, required=True),
        Slot("memory", SlotKind.LONG_TEXT, default="so many good memories"),
    ]
    anniversary_html = (
        "<article><h2>{{years}} years together</h2>"
        "<blockquote>{{memory}}</blockquote>"
        "<footer>{{_sender}} → {{_recipient}}</footer></article>"
    )
    g.add_template("Anniversary", "anniversary",
                   anniversary_slots, anniversary_html)

    # Create a card.
    alice_card = g.create_card(
        birthday_tid,
        {"name": "Alice", "age": "turning 30!"},
        recipient="Alice Jones",
        sender="Bob Smith",
        now_ms=1000,
    )
    print(g.render(alice_card))
    print()
    print("plain version:")
    print(g.render_plain(alice_card))
    print()

    # Try a validation failure.
    try:
        g.create_card(birthday_tid, {}, "Carol", "Dave", 2000)
    except MissingRequired as e:
        print(f"validation rejected: {e}")

    try:
        g.create_card(birthday_tid, {"typo": "x"}, "Carol", "Dave", 2000)
    except UnknownSlot as e:
        print(f"validation rejected: {e}")

    # Browse templates by category.
    print()
    print("Birthday templates:")
    for t in g.templates_in_category("birthday"):
        print(f"  - {t.name} ({len(t.slots)} slots)")
