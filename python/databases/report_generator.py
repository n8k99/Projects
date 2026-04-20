"""Report Generator — grouped reports with subtotal and grand-total rows.

Each output row is tagged Header/Data/Subtotal/Total. Columns declare
whether they're plain text, summable, and whether to render cents as $.
"""

from __future__ import annotations
from dataclasses import dataclass, field
from enum import Enum


class ValueKind(Enum):
    INT = "int"
    TEXT = "text"
    NULL = "null"


@dataclass
class Value:
    kind: ValueKind
    int_val: int = 0
    text_val: str = ""

    @classmethod
    def of_int(cls, n: int) -> Value: return cls(ValueKind.INT, int_val=n)
    @classmethod
    def of_text(cls, s: str) -> Value: return cls(ValueKind.TEXT, text_val=s)
    @classmethod
    def null(cls) -> Value: return cls(ValueKind.NULL)

    def as_int(self) -> int | None:
        return self.int_val if self.kind == ValueKind.INT else None

    def as_text(self) -> str | None:
        if self.kind == ValueKind.TEXT: return self.text_val
        if self.kind == ValueKind.INT: return str(self.int_val)
        return None


Record = dict[str, Value]


class ColumnKind(Enum):
    PLAIN = "plain"
    SUM_AGGREGATE = "sum_aggregate"


@dataclass
class ColumnDef:
    name: str
    field: str
    kind: ColumnKind = ColumnKind.PLAIN
    format_cents: bool = False


class RowKind(Enum):
    HEADER = "header"
    DATA = "data"
    SUBTOTAL = "subtotal"
    TOTAL = "total"


@dataclass
class ReportRow:
    kind: RowKind
    cells: list[str]
    group_label: str | None = None


@dataclass
class ReportSpec:
    title: str
    columns: list[ColumnDef]
    group_by: str | None = None


@dataclass
class Report:
    title: str
    rows: list[ReportRow]


def _format_int(n: int, as_cents: bool) -> str:
    if not as_cents:
        return str(n)
    abs_n = abs(n)
    dollars = abs_n // 100
    cents = abs_n % 100
    sign = "-" if n < 0 else ""
    return f"${sign}{dollars}.{cents:02d}"


def _format_cell(value: Value | None, col: ColumnDef) -> str:
    if value is None or value.kind == ValueKind.NULL:
        return ""
    if value.kind == ValueKind.INT:
        return _format_int(value.int_val, col.format_cents)
    return value.text_val


def generate(records: list[Record], spec: ReportSpec) -> Report:
    rows: list[ReportRow] = []
    rows.append(ReportRow(RowKind.HEADER, [c.name for c in spec.columns]))

    if spec.group_by is not None:
        groups: dict[str, list[Record]] = {}
        for r in records:
            v = r.get(spec.group_by)
            key = v.as_text() if v is not None else "(none)"
            if key is None: key = "(none)"
            groups.setdefault(key, []).append(r)
        sorted_groups = sorted(groups.items())
    else:
        sorted_groups = [("__all__", list(records))]

    grand_totals: dict[str, int] = {}

    for group_key, group_records in sorted_groups:
        for record in group_records:
            cells = [_format_cell(record.get(c.field), c) for c in spec.columns]
            rows.append(ReportRow(
                RowKind.DATA, cells,
                group_label=group_key if spec.group_by is not None else None
            ))
        if spec.group_by is not None:
            subtotals: dict[str, int] = {}
            for record in group_records:
                for c in spec.columns:
                    if c.kind == ColumnKind.SUM_AGGREGATE:
                        v = record.get(c.field)
                        if v is not None and v.as_int() is not None:
                            subtotals[c.field] = subtotals.get(c.field, 0) + v.as_int()
                            grand_totals[c.field] = grand_totals.get(c.field, 0) + v.as_int()
            cells = []
            for i, c in enumerate(spec.columns):
                if i == 0:
                    cells.append(f"{group_key} subtotal")
                elif c.kind == ColumnKind.SUM_AGGREGATE:
                    cells.append(_format_int(subtotals.get(c.field, 0), c.format_cents))
                else:
                    cells.append("")
            rows.append(ReportRow(RowKind.SUBTOTAL, cells, group_label=group_key))

    has_aggregate = any(c.kind == ColumnKind.SUM_AGGREGATE for c in spec.columns)
    if has_aggregate:
        if spec.group_by is None:
            for record in records:
                for c in spec.columns:
                    if c.kind == ColumnKind.SUM_AGGREGATE:
                        v = record.get(c.field)
                        if v is not None and v.as_int() is not None:
                            grand_totals[c.field] = grand_totals.get(c.field, 0) + v.as_int()
        cells = []
        for i, c in enumerate(spec.columns):
            if i == 0:
                cells.append("TOTAL")
            elif c.kind == ColumnKind.SUM_AGGREGATE:
                cells.append(_format_int(grand_totals.get(c.field, 0), c.format_cents))
            else:
                cells.append("")
        rows.append(ReportRow(RowKind.TOTAL, cells))

    return Report(title=spec.title, rows=rows)


def render(report: Report) -> str:
    out = []
    if report.title:
        out.append(report.title)
    for row in report.rows:
        prefix = {RowKind.HEADER: "#", RowKind.DATA: " ",
                  RowKind.SUBTOTAL: "~", RowKind.TOTAL: "="}[row.kind]
        out.append(prefix + "\t" + "\t".join(row.cells))
    return "\n".join(out) + "\n"


def demo() -> None:
    records = []
    for region, product, qty, rev in [
        ("East", "Widget", 3, 1500), ("East", "Gadget", 2, 2000),
        ("West", "Widget", 5, 2500), ("West", "Gizmo", 1, 750),
    ]:
        records.append({
            "region": Value.of_text(region),
            "product": Value.of_text(product),
            "qty": Value.of_int(qty),
            "revenue": Value.of_int(rev),
        })

    spec = ReportSpec(
        title="Sales Report",
        columns=[
            ColumnDef("Region", "region", ColumnKind.PLAIN),
            ColumnDef("Product", "product", ColumnKind.PLAIN),
            ColumnDef("Qty", "qty", ColumnKind.SUM_AGGREGATE),
            ColumnDef("Revenue", "revenue", ColumnKind.SUM_AGGREGATE, format_cents=True),
        ],
        group_by="region",
    )
    print(render(generate(records, spec)), end="")


if __name__ == "__main__":
    demo()
