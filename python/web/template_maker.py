"""Template Maker — meta-level tool for building and validating templates.

Where G081 consumed pre-built templates, G083 is the builder for templates
themselves. A `TemplateBuilder` holds the work-in-progress: slot
declarations, HTML draft, sample values for preview. Validation finds
mismatches between slot declarations and placeholder references.

First Rosetta Stone project where the code producing templates is itself
a data model — inspect, edit, validate, serialise without running code.
"""

from __future__ import annotations
import re
from dataclasses import dataclass, field
from enum import Enum


# Reuse SlotKind + Slot from G081.
from .ecard import Slot, SlotKind


@dataclass
class TemplateBuilder:
    id: int
    name: str
    category: str
    slots: list[Slot] = field(default_factory=list)
    html_draft: str = ""
    sample_data: dict[str, str] = field(default_factory=dict)


class IssueKind(Enum):
    UNUSED_SLOT = "unused_slot"
    UNDECLARED_PLACEHOLDER = "undeclared_placeholder"
    MISSING_SAMPLE = "missing_sample"
    DUPLICATE_SLOT = "duplicate_slot"
    INVALID_SLOT_NAME = "invalid_slot_name"


@dataclass(frozen=True)
class ValidationIssue:
    kind: IssueKind
    name: str


class BuilderError(Exception):
    pass


class UnknownBuilder(BuilderError):
    def __init__(self, bid: int) -> None:
        super().__init__(f"unknown builder: {bid}")


class SlotAlreadyExists(BuilderError):
    def __init__(self, name: str) -> None:
        super().__init__(f"slot already exists: {name}")


class SlotNotFound(BuilderError):
    def __init__(self, name: str) -> None:
        super().__init__(f"slot not found: {name}")


_PLACEHOLDER_RE = re.compile(r"\{\{([^{}]+)\}\}")
_SLOT_NAME_RE = re.compile(r"^[A-Za-z0-9_]+$")


class TemplateMaker:
    def __init__(self) -> None:
        self._builders: list[TemplateBuilder] = []
        self._next_id = 1

    def new_template(self, name: str, category: str) -> int:
        bid = self._next_id
        self._next_id += 1
        self._builders.append(TemplateBuilder(id=bid, name=name, category=category))
        return bid

    def builder(self, bid: int) -> TemplateBuilder | None:
        return next((b for b in self._builders if b.id == bid), None)

    def _require(self, bid: int) -> TemplateBuilder:
        b = self.builder(bid)
        if b is None:
            raise UnknownBuilder(bid)
        return b

    def add_slot(self, bid: int, slot: Slot) -> None:
        b = self._require(bid)
        if any(s.name == slot.name for s in b.slots):
            raise SlotAlreadyExists(slot.name)
        b.slots.append(slot)

    def remove_slot(self, bid: int, slot_name: str) -> None:
        b = self._require(bid)
        pos = next((i for i, s in enumerate(b.slots) if s.name == slot_name), None)
        if pos is None:
            raise SlotNotFound(slot_name)
        del b.slots[pos]
        b.sample_data.pop(slot_name, None)

    def update_html(self, bid: int, html: str) -> None:
        self._require(bid).html_draft = html

    def set_sample(self, bid: int, slot_name: str, value: str) -> None:
        b = self._require(bid)
        if not any(s.name == slot_name for s in b.slots):
            raise SlotNotFound(slot_name)
        b.sample_data[slot_name] = value

    def validate(self, bid: int) -> list[ValidationIssue]:
        b = self.builder(bid)
        if b is None:
            return []
        issues: list[ValidationIssue] = []
        seen: set[str] = set()
        for slot in b.slots:
            if not slot.name or not _SLOT_NAME_RE.match(slot.name):
                issues.append(ValidationIssue(IssueKind.INVALID_SLOT_NAME, slot.name))
            if slot.name in seen:
                issues.append(ValidationIssue(IssueKind.DUPLICATE_SLOT, slot.name))
            seen.add(slot.name)

        declared = {s.name for s in b.slots}
        referenced = set(_PLACEHOLDER_RE.findall(b.html_draft))
        for slot in b.slots:
            if slot.name not in referenced:
                issues.append(ValidationIssue(IssueKind.UNUSED_SLOT, slot.name))
        for ref in referenced:
            if ref.startswith("_"):
                continue  # special _recipient/_sender
            if ref not in declared:
                issues.append(ValidationIssue(IssueKind.UNDECLARED_PLACEHOLDER, ref))
        for slot in b.slots:
            if slot.required and slot.name not in b.sample_data:
                issues.append(ValidationIssue(IssueKind.MISSING_SAMPLE, slot.name))
        return issues

    def preview(self, bid: int) -> str | None:
        b = self.builder(bid)
        if b is None:
            return None
        out = b.html_draft
        for slot in b.slots:
            value = b.sample_data.get(slot.name) or slot.default
            if value is not None:
                out = out.replace(f"{{{{{slot.name}}}}}", value)
        out = out.replace("{{_recipient}}", "[preview recipient]")
        out = out.replace("{{_sender}}", "[preview sender]")
        return out

    def remove_builder(self, bid: int) -> bool:
        for i, b in enumerate(self._builders):
            if b.id == bid:
                del self._builders[i]
                return True
        return False

    def builders(self) -> list[TemplateBuilder]:
        return list(self._builders)


if __name__ == "__main__":
    m = TemplateMaker()
    bid = m.new_template("Classic Birthday", "birthday")
    m.add_slot(bid, Slot("name", SlotKind.TEXT, required=True))
    m.add_slot(bid, Slot("age", SlotKind.TEXT, default="another year older"))
    m.add_slot(bid, Slot("color", SlotKind.COLOR, default="#ffcc00"))

    m.update_html(bid,
        '<div style="background:{{color}}">'
        '<h1>Happy Birthday, {{name}}!</h1>'
        '<p>{{age}}. Typo: {{sendr}}.</p>'
        '</div>')

    print("issues before fixing:")
    for issue in m.validate(bid):
        print(f"  - {issue.kind.value}: {issue.name}")

    # Fix: remove the typo, add a sample for the required slot.
    m.update_html(bid,
        '<div style="background:{{color}}">'
        '<h1>Happy Birthday, {{name}}!</h1>'
        '<p>{{age}}. From {{_sender}}.</p>'
        '</div>')
    m.set_sample(bid, "name", "Alice")

    print()
    print("issues after fixing:")
    issues = m.validate(bid)
    if not issues:
        print("  (none — template is clean)")
    else:
        for issue in issues:
            print(f"  - {issue.kind.value}: {issue.name}")

    print()
    print("preview:")
    print(m.preview(bid))
