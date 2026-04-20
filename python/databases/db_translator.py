"""Database Translation — dialect-aware DDL emission.

Abstract types render differently per dialect:
  SmallInt → MySQL TINYINT / Postgres SMALLINT / SQLite INTEGER
  Boolean  → MySQL TINYINT(1) / Postgres BOOLEAN / SQLite INTEGER
  Timestamp → MySQL DATETIME / Postgres TIMESTAMP / SQLite TEXT
AUTO_INCREMENT / IDENTITY / AUTOINCREMENT each have their own syntax.
NOW() vs CURRENT_TIMESTAMP for default-timestamp.
"""

from __future__ import annotations
from dataclasses import dataclass, field
from enum import Enum


class Dialect(Enum):
    MYSQL = "mysql"
    POSTGRES = "postgres"
    SQLITE = "sqlite"


class ColumnKind(Enum):
    SMALLINT = "smallint"
    INTEGER = "integer"
    BIGINT = "bigint"
    TEXT = "text"
    VARCHAR = "varchar"
    BOOLEAN = "boolean"
    TIMESTAMP = "timestamp"
    DATE = "date"
    FLOAT = "float"
    DECIMAL = "decimal"


@dataclass
class ColumnType:
    kind: ColumnKind
    length: int = 0             # VARCHAR length
    precision: int = 0          # DECIMAL precision
    scale: int = 0              # DECIMAL scale

    @classmethod
    def small_int(cls) -> ColumnType: return cls(ColumnKind.SMALLINT)
    @classmethod
    def integer(cls) -> ColumnType: return cls(ColumnKind.INTEGER)
    @classmethod
    def big_int(cls) -> ColumnType: return cls(ColumnKind.BIGINT)
    @classmethod
    def text(cls) -> ColumnType: return cls(ColumnKind.TEXT)
    @classmethod
    def varchar(cls, length: int) -> ColumnType: return cls(ColumnKind.VARCHAR, length=length)
    @classmethod
    def boolean(cls) -> ColumnType: return cls(ColumnKind.BOOLEAN)
    @classmethod
    def timestamp(cls) -> ColumnType: return cls(ColumnKind.TIMESTAMP)
    @classmethod
    def date(cls) -> ColumnType: return cls(ColumnKind.DATE)
    @classmethod
    def float_(cls) -> ColumnType: return cls(ColumnKind.FLOAT)
    @classmethod
    def decimal(cls, precision: int, scale: int) -> ColumnType:
        return cls(ColumnKind.DECIMAL, precision=precision, scale=scale)

    def render(self, dialect: Dialect) -> str:
        k = self.kind
        if k == ColumnKind.SMALLINT:
            if dialect == Dialect.MYSQL: return "TINYINT"
            if dialect == Dialect.POSTGRES: return "SMALLINT"
            return "INTEGER"
        if k == ColumnKind.INTEGER:
            return "INTEGER"
        if k == ColumnKind.BIGINT:
            return "INTEGER" if dialect == Dialect.SQLITE else "BIGINT"
        if k == ColumnKind.TEXT:
            return "TEXT"
        if k == ColumnKind.VARCHAR:
            return f"VARCHAR({self.length})"
        if k == ColumnKind.BOOLEAN:
            if dialect == Dialect.MYSQL: return "TINYINT(1)"
            if dialect == Dialect.POSTGRES: return "BOOLEAN"
            return "INTEGER"
        if k == ColumnKind.TIMESTAMP:
            if dialect == Dialect.MYSQL: return "DATETIME"
            if dialect == Dialect.POSTGRES: return "TIMESTAMP"
            return "TEXT"
        if k == ColumnKind.DATE:
            return "TEXT" if dialect == Dialect.SQLITE else "DATE"
        if k == ColumnKind.FLOAT:
            return "REAL"
        if k == ColumnKind.DECIMAL:
            return f"DECIMAL({self.precision},{self.scale})"
        return "UNKNOWN"


class DefaultKind(Enum):
    LITERAL = "literal"
    NOW = "now"


@dataclass
class DefaultValue:
    kind: DefaultKind
    literal: str = ""

    @classmethod
    def of_literal(cls, s: str) -> DefaultValue:
        return cls(DefaultKind.LITERAL, literal=s)

    @classmethod
    def now(cls) -> DefaultValue:
        return cls(DefaultKind.NOW)

    def render(self, dialect: Dialect) -> str:
        if self.kind == DefaultKind.LITERAL:
            return self.literal
        if dialect == Dialect.MYSQL:
            return "NOW()"
        return "CURRENT_TIMESTAMP"


@dataclass
class DbColumn:
    name: str
    ty: ColumnType
    nullable: bool = True
    primary_key: bool = False
    unique: bool = False
    auto_increment: bool = False
    default: DefaultValue | None = None

    def pk(self) -> DbColumn:
        self.primary_key = True
        self.nullable = False
        return self

    def not_null(self) -> DbColumn:
        self.nullable = False
        return self

    def make_unique(self) -> DbColumn:
        self.unique = True
        return self

    def auto_inc(self) -> DbColumn:
        self.auto_increment = True
        return self

    def with_default(self, d: DefaultValue) -> DbColumn:
        self.default = d
        return self


@dataclass
class Table:
    name: str
    columns: list[DbColumn] = field(default_factory=list)


def _render_auto_increment(dialect: Dialect) -> str:
    return {
        Dialect.MYSQL: "AUTO_INCREMENT",
        Dialect.POSTGRES: "GENERATED ALWAYS AS IDENTITY",
        Dialect.SQLITE: "AUTOINCREMENT",
    }[dialect]


def _render_column(col: DbColumn, dialect: Dialect) -> str:
    parts = [col.name, col.ty.render(dialect)]
    if col.primary_key:
        parts.append("PRIMARY KEY")
    if not col.nullable and not col.primary_key:
        parts.append("NOT NULL")
    if col.unique and not col.primary_key:
        parts.append("UNIQUE")
    if col.auto_increment:
        if dialect == Dialect.SQLITE and col.primary_key:
            parts.append("AUTOINCREMENT")
        elif dialect != Dialect.SQLITE:
            parts.append(_render_auto_increment(dialect))
    if col.default is not None:
        parts.append(f"DEFAULT {col.default.render(dialect)}")
    return " ".join(parts)


def emit_create_table(table: Table, dialect: Dialect) -> str:
    lines = [f"CREATE TABLE {table.name} ("]
    cols = [f"  {_render_column(c, dialect)}" for c in table.columns]
    lines.append(",\n".join(cols))
    lines.append(");")
    return "\n".join(lines) + "\n"


def emit_schema(tables: list[Table], dialect: Dialect) -> str:
    return "".join(emit_create_table(t, dialect) for t in tables)


def demo() -> None:
    users = Table(
        name="users",
        columns=[
            DbColumn("id", ColumnType.integer()).pk().auto_inc(),
            DbColumn("email", ColumnType.varchar(255)).not_null().make_unique(),
            DbColumn("name", ColumnType.text()).not_null(),
            DbColumn("active", ColumnType.boolean()).not_null()
                    .with_default(DefaultValue.of_literal("1")),
            DbColumn("created_at", ColumnType.timestamp()).not_null()
                    .with_default(DefaultValue.now()),
        ],
    )
    for dialect in [Dialect.MYSQL, Dialect.POSTGRES, Dialect.SQLITE]:
        print(f"=== {dialect.value} ===")
        print(emit_create_table(users, dialect))


if __name__ == "__main__":
    demo()
