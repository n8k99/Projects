"""SQL Query Analyzer — recursive-descent parser + static analysis.

Parse a small SELECT grammar, then analyse against a schema:
unknown table/column, SELECT *, LIMIT without ORDER BY, missing WHERE.
"""

from __future__ import annotations
from dataclasses import dataclass, field
from enum import Enum


class TokenKind(Enum):
    WORD = "word"
    NUMBER = "number"
    STRING = "string"
    OP = "op"
    COMMA = "comma"
    STAR = "star"
    SEMICOLON = "semicolon"


@dataclass
class Token:
    kind: TokenKind
    word: str = ""
    number: int = 0
    string: str = ""
    op: str = ""


def tokenise(s: str) -> list[Token]:
    out: list[Token] = []
    i, n = 0, len(s)
    while i < n:
        c = s[i]
        if c.isspace():
            i += 1; continue
        if c == ",": out.append(Token(TokenKind.COMMA)); i += 1; continue
        if c == ";": out.append(Token(TokenKind.SEMICOLON)); i += 1; continue
        if c == "*": out.append(Token(TokenKind.STAR)); i += 1; continue
        if c == "'":
            i += 1
            start = i
            while i < n and s[i] != "'":
                i += 1
            if i >= n:
                raise ValueError("unterminated string")
            out.append(Token(TokenKind.STRING, string=s[start:i]))
            i += 1
            continue
        if c == "=":
            out.append(Token(TokenKind.OP, op="=")); i += 1; continue
        if c in "<>!":
            op = c
            i += 1
            if i < n and s[i] == "=":
                op += "="
                i += 1
            if op == "!":
                raise ValueError("bad operator: !")
            out.append(Token(TokenKind.OP, op=op))
            continue
        if c.isdigit() or (c == "-" and i + 1 < n and s[i + 1].isdigit()):
            start = i
            if c == "-": i += 1
            while i < n and s[i].isdigit(): i += 1
            out.append(Token(TokenKind.NUMBER, number=int(s[start:i])))
            continue
        if c.isalpha() or c == "_":
            start = i
            while i < n and (s[i].isalnum() or s[i] == "_"):
                i += 1
            out.append(Token(TokenKind.WORD, word=s[start:i].lower()))
            continue
        raise ValueError(f"unexpected char: {c}")
    return out


class Order(Enum):
    ASC = "asc"
    DESC = "desc"


@dataclass
class Value:
    is_num: bool
    num: int = 0
    text: str = ""


@dataclass
class WhereClause:
    column: str
    op: str
    value: Value


@dataclass
class OrderBy:
    column: str
    order: Order


@dataclass
class Query:
    projection_all: bool
    projection_columns: list[str]
    table: str
    where_clause: WhereClause | None = None
    order_by: OrderBy | None = None
    limit: int | None = None


class _Parser:
    def __init__(self, tokens: list[Token]):
        self.tokens = tokens
        self.pos = 0

    def peek(self) -> Token | None:
        return self.tokens[self.pos] if self.pos < len(self.tokens) else None

    def bump(self) -> Token:
        t = self.tokens[self.pos]
        self.pos += 1
        return t

    def expect_word(self, w: str) -> None:
        t = self.bump() if self.pos < len(self.tokens) else None
        if t is None or t.kind != TokenKind.WORD or t.word != w:
            raise ValueError(f"expected '{w}', got {t}")

    def parse_query(self) -> Query:
        self.expect_word("select")
        all_cols, cols = self.parse_projection()
        self.expect_word("from")
        table = self.parse_identifier()
        where = None
        if self.peek_word() == "where":
            self.bump()
            where = self.parse_where()
        order = None
        if self.peek_word() == "order":
            self.bump()
            self.expect_word("by")
            order = self.parse_order_by()
        limit = None
        if self.peek_word() == "limit":
            self.bump()
            t = self.bump()
            if t.kind != TokenKind.NUMBER or t.number < 0:
                raise ValueError(f"expected positive number after LIMIT")
            limit = t.number
        return Query(
            projection_all=all_cols,
            projection_columns=cols,
            table=table,
            where_clause=where,
            order_by=order,
            limit=limit,
        )

    def peek_word(self) -> str | None:
        t = self.peek()
        return t.word if t is not None and t.kind == TokenKind.WORD else None

    def parse_projection(self) -> tuple[bool, list[str]]:
        t = self.peek()
        if t is not None and t.kind == TokenKind.STAR:
            self.bump()
            return (True, [])
        cols = [self.parse_identifier()]
        while self.peek() is not None and self.peek().kind == TokenKind.COMMA:
            self.bump()
            cols.append(self.parse_identifier())
        return (False, cols)

    def parse_identifier(self) -> str:
        t = self.bump()
        if t.kind != TokenKind.WORD:
            raise ValueError(f"expected identifier, got {t}")
        return t.word

    def parse_where(self) -> WhereClause:
        column = self.parse_identifier()
        op_tok = self.bump()
        if op_tok.kind != TokenKind.OP:
            raise ValueError(f"expected operator, got {op_tok}")
        val_tok = self.bump()
        if val_tok.kind == TokenKind.NUMBER:
            value = Value(is_num=True, num=val_tok.number)
        elif val_tok.kind == TokenKind.STRING:
            value = Value(is_num=False, text=val_tok.string)
        else:
            raise ValueError(f"expected value, got {val_tok}")
        return WhereClause(column=column, op=op_tok.op, value=value)

    def parse_order_by(self) -> OrderBy:
        column = self.parse_identifier()
        order = Order.ASC
        t = self.peek()
        if t is not None and t.kind == TokenKind.WORD:
            if t.word == "asc": self.bump()
            elif t.word == "desc": self.bump(); order = Order.DESC
        return OrderBy(column=column, order=order)


def parse(s: str) -> Query:
    tokens = tokenise(s)
    if tokens and tokens[-1].kind == TokenKind.SEMICOLON:
        tokens = tokens[:-1]
    p = _Parser(tokens)
    q = p.parse_query()
    if p.pos < len(p.tokens):
        raise ValueError("trailing tokens after query")
    return q


class Severity(Enum):
    INFO = "info"
    WARNING = "warning"
    ERROR = "error"


@dataclass
class Finding:
    severity: Severity
    code: str
    message: str


@dataclass
class Analysis:
    tables: set[str]
    columns: set[str]
    findings: list[Finding]


@dataclass
class Schema:
    tables: dict[str, set[str]] = field(default_factory=dict)

    def add_table(self, name: str, columns: list[str]) -> None:
        self.tables[name] = set(columns)


def analyse(query: Query, schema: Schema) -> Analysis:
    tables = {query.table}
    columns: set[str] = set()
    findings: list[Finding] = []

    known = schema.tables.get(query.table)
    if known is None:
        findings.append(Finding(Severity.ERROR, "UNKNOWN_TABLE",
                                 f"unknown table: {query.table}"))

    referenced = set()
    if not query.projection_all:
        referenced.update(query.projection_columns)
    if query.where_clause is not None:
        referenced.add(query.where_clause.column)
    if query.order_by is not None:
        referenced.add(query.order_by.column)

    for col in referenced:
        columns.add(col)
        if known is not None and col not in known:
            findings.append(Finding(Severity.ERROR, "UNKNOWN_COLUMN",
                                     f"unknown column {col} in table {query.table}"))

    if query.projection_all:
        findings.append(Finding(Severity.WARNING, "SELECT_STAR",
                                 "SELECT * pulls every column; name them explicitly"))

    if query.limit is not None and query.order_by is None:
        findings.append(Finding(Severity.WARNING, "LIMIT_WITHOUT_ORDER",
                                 "LIMIT without ORDER BY returns arbitrary rows"))

    if query.where_clause is None and query.limit is None:
        findings.append(Finding(Severity.INFO, "FULL_SCAN",
                                 "no WHERE or LIMIT — query will scan every row"))

    return Analysis(tables=tables, columns=columns, findings=findings)


def demo() -> None:
    schema = Schema()
    schema.add_table("users", ["id", "name", "email", "created_at"])
    schema.add_table("orders", ["id", "user_id", "amount", "status"])

    queries = [
        "SELECT id, name FROM users WHERE id = 42",
        "SELECT * FROM users LIMIT 10",
        "SELECT name FROM users ORDER BY created_at DESC LIMIT 5",
        "SELECT birthday FROM widgets",
    ]
    for s in queries:
        print(f"\n=== {s} ===")
        try:
            q = parse(s)
            a = analyse(q, schema)
            print(f"  tables:  {sorted(a.tables)}")
            print(f"  columns: {sorted(a.columns)}")
            for f in a.findings:
                print(f"  [{f.severity.value:7}] {f.code}: {f.message}")
        except ValueError as e:
            print(f"  parse error: {e}")


if __name__ == "__main__":
    demo()
