"""Excel Spreadsheet Exporter — cell-addressed grid with typed values and formulae.

Sparse grid; lazy formula evaluation with cycle detection. Supported ops:
+ - * / over operands; SUM/AVG/MIN/MAX/COUNT over ranges (A1:A5).
Export: CSV with proper quoting.
"""

from __future__ import annotations
from dataclasses import dataclass
from enum import Enum


class CellKind(Enum):
    BLANK = "blank"
    NUMBER = "number"
    TEXT = "text"
    FORMULA = "formula"


@dataclass
class Cell:
    kind: CellKind
    number: float = 0.0
    text: str = ""
    formula: str = ""


class ValueKind(Enum):
    EMPTY = "empty"
    NUMBER = "number"
    TEXT = "text"
    ERROR = "error"


@dataclass
class Value:
    kind: ValueKind
    number: float = 0.0
    text: str = ""

    @staticmethod
    def empty() -> Value: return Value(ValueKind.EMPTY)
    @staticmethod
    def num(n: float) -> Value: return Value(ValueKind.NUMBER, number=n)
    @staticmethod
    def txt(s: str) -> Value: return Value(ValueKind.TEXT, text=s)
    @staticmethod
    def err(s: str) -> Value: return Value(ValueKind.ERROR, text=s)

    def as_number(self) -> float | None:
        if self.kind == ValueKind.NUMBER:
            return self.number
        if self.kind == ValueKind.TEXT:
            try:
                return float(self.text)
            except ValueError:
                return None
        return None


def parse_address(addr: str) -> tuple[int, int] | None:
    if not addr or not addr[0].isalpha():
        return None
    letter = addr[0].upper()
    col = ord(letter) - ord("A")
    try:
        row = int(addr[1:])
    except ValueError:
        return None
    if row < 1:
        return None
    return (row - 1, col)


def format_address(row: int, col: int) -> str:
    return chr(ord("A") + col) + str(row + 1)


class Sheet:
    def __init__(self) -> None:
        self.cells: dict[tuple[int, int], Cell] = {}

    def set(self, addr: str, cell: Cell) -> None:
        pos = parse_address(addr)
        if pos is None:
            raise ValueError(f"bad address: {addr}")
        if cell.kind == CellKind.BLANK:
            self.cells.pop(pos, None)
        else:
            self.cells[pos] = cell

    def set_number(self, addr: str, n: float) -> None:
        self.set(addr, Cell(CellKind.NUMBER, number=n))

    def set_text(self, addr: str, s: str) -> None:
        self.set(addr, Cell(CellKind.TEXT, text=s))

    def set_formula(self, addr: str, f: str) -> None:
        self.set(addr, Cell(CellKind.FORMULA, formula=f))

    def raw(self, addr: str) -> Cell | None:
        pos = parse_address(addr)
        return self.cells.get(pos) if pos else None

    def evaluate(self, addr: str) -> Value:
        pos = parse_address(addr)
        if pos is None:
            return Value.err(f"bad address: {addr}")
        return self._eval_cell(pos, set())

    def _eval_cell(self, pos: tuple[int, int], visiting: set) -> Value:
        if pos in visiting:
            return Value.err("circular reference")
        visiting = visiting | {pos}
        cell = self.cells.get(pos)
        if cell is None or cell.kind == CellKind.BLANK:
            return Value.empty()
        if cell.kind == CellKind.NUMBER:
            return Value.num(cell.number)
        if cell.kind == CellKind.TEXT:
            return Value.txt(cell.text)
        return self._eval_formula(cell.formula, visiting)

    def _eval_formula(self, f: str, visiting: set) -> Value:
        body = f.lstrip("=").strip()

        # Function call: NAME(args)
        paren = body.find("(")
        if paren > 0 and body.endswith(")"):
            name = body[:paren]
            args = body[paren + 1:-1]
            return self._eval_function(name, args, visiting)

        # Binary op
        for op in "+-*/":
            pos = body.find(op)
            if pos > 0:
                l = self._eval_operand(body[:pos].strip(), visiting)
                r = self._eval_operand(body[pos + 1:].strip(), visiting)
                return _apply_op(op, l, r)

        return self._eval_operand(body, visiting)

    def _eval_function(self, name: str, args: str, visiting: set) -> Value:
        values = self._expand_range(args, visiting)
        nums = [v.as_number() for v in values]
        nums = [n for n in nums if n is not None]
        name_u = name.upper()
        if name_u == "SUM":
            return Value.num(sum(nums))
        if name_u in ("AVG", "AVERAGE"):
            return Value.err("empty range for AVG") if not nums else Value.num(sum(nums) / len(nums))
        if name_u == "MIN":
            return Value.err("empty range") if not nums else Value.num(min(nums))
        if name_u == "MAX":
            return Value.err("empty range") if not nums else Value.num(max(nums))
        if name_u == "COUNT":
            return Value.num(len(nums))
        return Value.err(f"unknown function: {name}")

    def _expand_range(self, args: str, visiting: set) -> list[Value]:
        out = []
        for part in args.split(","):
            part = part.strip()
            if ":" in part:
                start, end = part.split(":", 1)
                s_pos = parse_address(start)
                e_pos = parse_address(end)
                if s_pos and e_pos:
                    for r in range(s_pos[0], e_pos[0] + 1):
                        for c in range(s_pos[1], e_pos[1] + 1):
                            out.append(self._eval_cell((r, c), visiting))
                    continue
            out.append(self._eval_operand(part, visiting))
        return out

    def _eval_operand(self, s: str, visiting: set) -> Value:
        s = s.strip()
        try:
            return Value.num(float(s))
        except ValueError:
            pass
        pos = parse_address(s)
        if pos is not None:
            return self._eval_cell(pos, visiting)
        return Value.err(f"bad operand: {s}")

    def to_csv(self) -> str:
        if not self.cells:
            return ""
        max_row = max(r for (r, _) in self.cells)
        max_col = max(c for (_, c) in self.cells)
        out = []
        for row in range(max_row + 1):
            cells = []
            for col in range(max_col + 1):
                val = self.evaluate(format_address(row, col))
                cells.append(_csv_escape(_format_value(val)))
            out.append(",".join(cells))
        return "\n".join(out) + "\n"


def _apply_op(op: str, l: Value, r: Value) -> Value:
    la = l.as_number()
    rb = r.as_number()
    if la is None or rb is None:
        return Value.err("non-numeric operand")
    if op == "+": return Value.num(la + rb)
    if op == "-": return Value.num(la - rb)
    if op == "*": return Value.num(la * rb)
    if op == "/":
        return Value.err("div by zero") if rb == 0 else Value.num(la / rb)
    return Value.err(f"bad op: {op}")


def _format_value(v: Value) -> str:
    if v.kind == ValueKind.EMPTY:
        return ""
    if v.kind == ValueKind.NUMBER:
        if v.number.is_integer() and abs(v.number) < 1e15:
            return str(int(v.number))
        return str(v.number)
    if v.kind == ValueKind.TEXT:
        return v.text
    return f"#ERR: {v.text}"


def _csv_escape(s: str) -> str:
    if "," in s or '"' in s or "\n" in s:
        return '"' + s.replace('"', '""') + '"'
    return s


def demo() -> None:
    s = Sheet()
    s.set_text("A1", "name")
    s.set_text("B1", "age")
    s.set_text("C1", "squared")
    s.set_text("A2", "Alice")
    s.set_number("B2", 30)
    s.set_formula("C2", "=B2 * B2")
    s.set_text("A3", "Bob")
    s.set_number("B3", 25)
    s.set_formula("C3", "=B3 * B3")
    s.set_text("A4", "Total")
    s.set_formula("B4", "=SUM(B2:B3)")
    s.set_formula("C4", "=SUM(C2:C3)")

    print("=== sheet CSV ===")
    print(s.to_csv())
    print(f"B4 = {s.evaluate('B4').number}")
    print(f"C4 = {s.evaluate('C4').number}")


if __name__ == "__main__":
    demo()
