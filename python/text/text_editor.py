"""Text Editor — buffer model with cursor, edit operations, and undo."""

from dataclasses import dataclass, field
import re


@dataclass
class CursorPos:
    line: int = 0
    col: int = 0


@dataclass
class EditAction:
    """Recorded edit for undo."""
    kind: str  # "insert_char", "delete_char", "insert_line", "delete_line", "replace"
    cursor_before: tuple[int, int] = (0, 0)
    data: dict = field(default_factory=dict)


class TextEditor:
    """A programmatic text buffer with cursor, edit operations, and undo stack."""

    def __init__(self, text: str = ""):
        if text:
            self.lines = text.split("\n")
        else:
            self.lines = [""]
        self.cursor = CursorPos(0, 0)
        self._undo_stack: list[EditAction] = []

    def _clamp_cursor(self):
        self.cursor.line = max(0, min(self.cursor.line, len(self.lines) - 1))
        self.cursor.col = max(0, min(self.cursor.col, len(self.lines[self.cursor.line])))

    def _save(self, kind: str, **data) -> EditAction:
        action = EditAction(
            kind=kind,
            cursor_before=(self.cursor.line, self.cursor.col),
            data=data,
        )
        self._undo_stack.append(action)
        return action

    def insert_char(self, ch: str):
        """Insert character at cursor position, advance cursor."""
        self._save("insert_char", char=ch)
        line = self.lines[self.cursor.line]
        col = self.cursor.col
        self.lines[self.cursor.line] = line[:col] + ch + line[col:]
        self.cursor.col += len(ch)

    def delete_char(self):
        """Backspace: delete character before cursor."""
        if self.cursor.col == 0 and self.cursor.line == 0:
            return
        if self.cursor.col > 0:
            line = self.lines[self.cursor.line]
            deleted = line[self.cursor.col - 1]
            self._save("delete_char", char=deleted)
            self.lines[self.cursor.line] = line[: self.cursor.col - 1] + line[self.cursor.col :]
            self.cursor.col -= 1
        else:
            # Join with previous line
            prev = self.cursor.line - 1
            join_col = len(self.lines[prev])
            self._save("delete_char_join", removed_line=self.lines[self.cursor.line], join_col=join_col)
            self.lines[prev] += self.lines[self.cursor.line]
            del self.lines[self.cursor.line]
            self.cursor.line = prev
            self.cursor.col = join_col

    def insert_line(self):
        """Split current line at cursor (like pressing Enter)."""
        self._save("insert_line")
        line = self.lines[self.cursor.line]
        before = line[: self.cursor.col]
        after = line[self.cursor.col :]
        self.lines[self.cursor.line] = before
        self.lines.insert(self.cursor.line + 1, after)
        self.cursor.line += 1
        self.cursor.col = 0

    def delete_line(self):
        """Remove current line entirely."""
        self._save("delete_line", content=self.lines[self.cursor.line], index=self.cursor.line)
        if len(self.lines) == 1:
            self.lines[0] = ""
            self.cursor.col = 0
        else:
            del self.lines[self.cursor.line]
            if self.cursor.line >= len(self.lines):
                self.cursor.line = len(self.lines) - 1
            self.cursor.col = 0

    def move_cursor(self, direction: str):
        """Move cursor: 'up', 'down', 'left', 'right'."""
        if direction == "left":
            if self.cursor.col > 0:
                self.cursor.col -= 1
            elif self.cursor.line > 0:
                self.cursor.line -= 1
                self.cursor.col = len(self.lines[self.cursor.line])
        elif direction == "right":
            if self.cursor.col < len(self.lines[self.cursor.line]):
                self.cursor.col += 1
            elif self.cursor.line < len(self.lines) - 1:
                self.cursor.line += 1
                self.cursor.col = 0
        elif direction == "up":
            if self.cursor.line > 0:
                self.cursor.line -= 1
                self._clamp_cursor()
        elif direction == "down":
            if self.cursor.line < len(self.lines) - 1:
                self.cursor.line += 1
                self._clamp_cursor()

    def goto(self, line: int, col: int):
        """Move cursor to specific position."""
        self.cursor.line = line
        self.cursor.col = col
        self._clamp_cursor()

    def get_line(self, n: int) -> str:
        """Get content of line n."""
        if 0 <= n < len(self.lines):
            return self.lines[n]
        raise IndexError(f"Line {n} out of range (0..{len(self.lines) - 1})")

    def get_text(self) -> str:
        """Return full buffer content."""
        return "\n".join(self.lines)

    def undo(self):
        """Revert the last edit operation."""
        if not self._undo_stack:
            return
        action = self._undo_stack.pop()
        kind = action.kind
        prev_line, prev_col = action.cursor_before

        if kind == "insert_char":
            ch = action.data["char"]
            line = self.lines[prev_line]
            self.lines[prev_line] = line[:prev_col] + line[prev_col + len(ch):]
        elif kind == "delete_char":
            ch = action.data["char"]
            line = self.lines[prev_line]
            self.lines[prev_line] = line[:prev_col - 1] + ch + line[prev_col - 1:]
        elif kind == "delete_char_join":
            join_col = action.data["join_col"]
            removed = action.data["removed_line"]
            combined = self.lines[prev_line]
            self.lines[prev_line] = combined[:join_col]
            self.lines.insert(prev_line + 1, removed)
        elif kind == "insert_line":
            # Rejoin the two lines
            if prev_line + 1 < len(self.lines):
                self.lines[prev_line] += self.lines[prev_line + 1]
                del self.lines[prev_line + 1]
        elif kind == "delete_line":
            idx = action.data["index"]
            content = action.data["content"]
            if len(self.lines) == 1 and self.lines[0] == "":
                self.lines[0] = content
            else:
                self.lines.insert(idx, content)
        elif kind == "replace":
            self.lines = action.data["old_lines"][:]

        self.cursor.line = prev_line
        self.cursor.col = prev_col

    def find(self, pattern: str) -> list[tuple[int, int]]:
        """Find all occurrences of pattern. Returns list of (line, col)."""
        results = []
        for i, line in enumerate(self.lines):
            start = 0
            while True:
                idx = line.find(pattern, start)
                if idx == -1:
                    break
                results.append((i, idx))
                start = idx + 1
        return results

    def replace(self, pattern: str, replacement: str, count: int = -1) -> int:
        """Replace occurrences of pattern. Returns number of replacements made."""
        self._save("replace", old_lines=self.lines[:])
        total = 0
        for i in range(len(self.lines)):
            if count == -1:
                new_line, n = re.subn(re.escape(pattern), replacement, self.lines[i])
            else:
                remaining = count - total
                if remaining <= 0:
                    break
                new_line, n = re.subn(re.escape(pattern), replacement, self.lines[i], count=remaining)
            if n > 0:
                self.lines[i] = new_line
                total += n
        if total == 0:
            self._undo_stack.pop()  # nothing changed, remove the undo record
        return total


if __name__ == "__main__":
    print("=== Text Editor Demo ===\n")

    ed = TextEditor()

    # Type "Hello, world!"
    for ch in "Hello, world!":
        ed.insert_char(ch)
    print(f"After typing: '{ed.get_text()}'")
    print(f"Cursor: line={ed.cursor.line}, col={ed.cursor.col}")

    # Press Enter and type a second line
    ed.insert_line()
    for ch in "This is line two.":
        ed.insert_char(ch)
    print(f"\nAfter second line:\n{ed.get_text()}")

    # Undo the entire second line
    for _ in range(len("This is line two.")):
        ed.undo()
    ed.undo()  # undo the Enter
    print(f"\nAfter undo back to one line: '{ed.get_text()}'")

    # Restore, add more text
    ed.insert_line()
    for ch in "The quick brown fox jumps.":
        ed.insert_char(ch)
    ed.insert_line()
    for ch in "The fox is quick.":
        ed.insert_char(ch)

    print(f"\nBuffer:\n{ed.get_text()}")

    # Find
    positions = ed.find("fox")
    print(f"\n'fox' found at: {positions}")

    # Replace
    n = ed.replace("fox", "cat")
    print(f"Replaced {n} occurrences of 'fox' -> 'cat'")
    print(f"Buffer:\n{ed.get_text()}")

    # Undo the replace
    ed.undo()
    print(f"\nAfter undo replace:\n{ed.get_text()}")
