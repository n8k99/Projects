"""Remote SQL Tool — in-memory table engine + execution over parsed queries.

Connection holds typed tables; INSERT validates types and arity; SELECT
filters/projects/orders/limits. Transactions buffer pending rows until
COMMIT (or discard via ROLLBACK). Pending rows are visible to SELECT
inside the transaction.
"""

from __future__ import annotations
from dataclasses import dataclass, field
from enum import Enum


class ColumnType(Enum):
    INT = "int"
    TEXT = "text"


@dataclass
class CellValue:
    is_null: bool = False
    is_int: bool = False
    int_val: int = 0
    text_val: str = ""

    @classmethod
    def null(cls) -> CellValue: return cls(is_null=True)
    @classmethod
    def of_int(cls, n: int) -> CellValue: return cls(is_int=True, int_val=n)
    @classmethod
    def of_text(cls, s: str) -> CellValue: return cls(text_val=s)

    def type_of(self) -> ColumnType | None:
        if self.is_null: return None
        if self.is_int: return ColumnType.INT
        return ColumnType.TEXT


@dataclass
class Column:
    name: str
    ty: ColumnType


@dataclass
class Table:
    name: str
    columns: list[Column]
    rows: list[list[CellValue]] = field(default_factory=list)

    def column_index(self, name: str) -> int:
        for i, c in enumerate(self.columns):
            if c.name == name:
                return i
        return -1


class ExecError(Exception):
    pass


@dataclass
class ResultSet:
    columns: list[str]
    rows: list[list[CellValue]]


@dataclass
class WhereExpr:
    column: str
    op: str
    value: CellValue


@dataclass
class OrderByExpr:
    column: str
    desc: bool = False


@dataclass
class SelectQuery:
    project_all: bool
    columns: list[str]
    table: str
    where_clause: WhereExpr | None = None
    order_by: OrderByExpr | None = None
    limit: int | None = None


class Connection:
    def __init__(self) -> None:
        self.tables: dict[str, Table] = {}
        self._pending: dict[str, list[list[CellValue]]] | None = None

    def in_transaction(self) -> bool:
        return self._pending is not None

    def create_table(self, name: str, columns: list[Column]) -> None:
        if name in self.tables:
            raise ExecError(f"table exists: {name}")
        self.tables[name] = Table(name=name, columns=columns)

    def insert(self, table: str, values: list[CellValue]) -> None:
        if table not in self.tables:
            raise ExecError(f"unknown table: {table}")
        t = self.tables[table]
        if len(values) != len(t.columns):
            raise ExecError(f"expected {len(t.columns)} values, got {len(values)}")
        for col, val in zip(t.columns, values):
            vt = val.type_of()
            if vt is not None and vt != col.ty:
                raise ExecError(f"type mismatch on column {col.name}")
        if self._pending is not None:
            self._pending.setdefault(table, []).append(values)
        else:
            t.rows.append(values)

    def begin(self) -> None:
        self._pending = {}

    def commit(self) -> None:
        if self._pending is None:
            raise ExecError("no transaction")
        for name, rows in self._pending.items():
            if name in self.tables:
                self.tables[name].rows.extend(rows)
        self._pending = None

    def rollback(self) -> None:
        if self._pending is None:
            raise ExecError("no transaction")
        self._pending = None

    def _rows_of(self, table: str) -> list[list[CellValue]]:
        t = self.tables.get(table)
        out = [list(r) for r in t.rows] if t else []
        if self._pending is not None and table in self._pending:
            out.extend(list(r) for r in self._pending[table])
        return out

    def select(self, q: SelectQuery) -> ResultSet:
        t = self.tables.get(q.table)
        if t is None:
            raise ExecError(f"unknown table: {q.table}")

        if q.project_all:
            proj = list(range(len(t.columns)))
        else:
            proj = []
            for c in q.columns:
                idx = t.column_index(c)
                if idx < 0:
                    raise ExecError(f"unknown column: {c}")
                proj.append(idx)

        where_idx = None
        if q.where_clause is not None:
            where_idx = t.column_index(q.where_clause.column)
            if where_idx < 0:
                raise ExecError(f"unknown column: {q.where_clause.column}")

        order_info = None
        if q.order_by is not None:
            oi = t.column_index(q.order_by.column)
            if oi < 0:
                raise ExecError(f"unknown column: {q.order_by.column}")
            order_info = (oi, q.order_by.desc)

        base_rows = self._rows_of(q.table)
        filtered = [
            r for r in base_rows
            if q.where_clause is None
            or _eval_pred(r[where_idx], q.where_clause.op, q.where_clause.value)
        ]

        if order_info is not None:
            oi, desc = order_info
            filtered.sort(key=lambda r: _sort_key(r[oi]), reverse=desc)

        projected = [[r[i] for i in proj] for r in filtered]
        if q.limit is not None:
            projected = projected[:q.limit]

        column_names = [t.columns[i].name for i in proj]
        return ResultSet(columns=column_names, rows=projected)


def _compare(a: CellValue, b: CellValue) -> int:
    if a.is_null and b.is_null: return 0
    if a.is_null: return -1
    if b.is_null: return 1
    if a.is_int and b.is_int:
        return (a.int_val > b.int_val) - (a.int_val < b.int_val)
    if not a.is_int and not b.is_int:
        return (a.text_val > b.text_val) - (a.text_val < b.text_val)
    return 0  # type mismatch → equal


def _eval_pred(cell: CellValue, op: str, rhs: CellValue) -> bool:
    c = _compare(cell, rhs)
    return {
        "=": c == 0, "!=": c != 0,
        "<": c < 0, "<=": c <= 0,
        ">": c > 0, ">=": c >= 0,
    }.get(op, False)


def _sort_key(v: CellValue):
    # For sorting: NULL sorts first, then int, then text.
    if v.is_null: return (0, 0)
    if v.is_int: return (1, v.int_val)
    return (2, v.text_val)


def demo() -> None:
    c = Connection()
    c.create_table("users", [
        Column("id", ColumnType.INT),
        Column("name", ColumnType.TEXT),
        Column("age", ColumnType.INT),
    ])
    c.insert("users", [CellValue.of_int(1), CellValue.of_text("Alice"), CellValue.of_int(30)])
    c.insert("users", [CellValue.of_int(2), CellValue.of_text("Bob"), CellValue.of_int(25)])
    c.insert("users", [CellValue.of_int(3), CellValue.of_text("Carol"), CellValue.of_int(35)])

    print("=== SELECT name FROM users WHERE age > 26 ORDER BY age DESC LIMIT 2 ===")
    r = c.select(SelectQuery(
        project_all=False, columns=["name"], table="users",
        where_clause=WhereExpr("age", ">", CellValue.of_int(26)),
        order_by=OrderByExpr("age", desc=True),
        limit=2,
    ))
    print(f"  columns: {r.columns}")
    for row in r.rows:
        print(f"  {[v.text_val if not v.is_int else v.int_val for v in row]}")

    print("\n=== transaction: BEGIN; INSERT Dave; ROLLBACK ===")
    c.begin()
    c.insert("users", [CellValue.of_int(4), CellValue.of_text("Dave"), CellValue.of_int(40)])
    print(f"  before rollback: {len(c._rows_of('users'))} rows")
    c.rollback()
    print(f"  after rollback: {len(c._rows_of('users'))} rows")


if __name__ == "__main__":
    demo()
