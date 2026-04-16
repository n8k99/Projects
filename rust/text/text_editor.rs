//! Text Editor — buffer model with cursor, edit operations, and undo.

/// Cursor position in the buffer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CursorPos {
    pub line: usize,
    pub col: usize,
}

/// A recorded edit action for the undo stack.
#[derive(Debug, Clone)]
enum EditorAction {
    InsertChar {
        cursor: CursorPos,
        ch: char,
    },
    DeleteChar {
        cursor: CursorPos,
        ch: char,
    },
    DeleteCharJoin {
        cursor: CursorPos,
        removed_line: String,
        join_col: usize,
    },
    InsertLine {
        cursor: CursorPos,
    },
    DeleteLine {
        cursor: CursorPos,
        content: String,
        index: usize,
    },
    Replace {
        cursor: CursorPos,
        old_lines: Vec<String>,
    },
}

/// A programmatic text buffer with cursor, edit operations, and undo stack.
pub struct TextEditor {
    lines: Vec<String>,
    cursor: CursorPos,
    undo_stack: Vec<EditorAction>,
}

impl TextEditor {
    /// Create a new editor, optionally initialized with text.
    pub fn new(text: &str) -> Self {
        let lines = if text.is_empty() {
            vec![String::new()]
        } else {
            text.split('\n').map(String::from).collect()
        };
        TextEditor {
            lines,
            cursor: CursorPos { line: 0, col: 0 },
            undo_stack: Vec::new(),
        }
    }

    /// Current cursor position.
    pub fn cursor(&self) -> CursorPos {
        self.cursor
    }

    /// Number of lines in the buffer.
    pub fn line_count(&self) -> usize {
        self.lines.len()
    }

    fn clamp_cursor(&mut self) {
        if self.cursor.line >= self.lines.len() {
            self.cursor.line = self.lines.len() - 1;
        }
        let line_len = self.lines[self.cursor.line].len();
        if self.cursor.col > line_len {
            self.cursor.col = line_len;
        }
    }

    /// Insert a character at the cursor position.
    pub fn insert_char(&mut self, ch: char) {
        self.undo_stack.push(EditorAction::InsertChar {
            cursor: self.cursor,
            ch,
        });
        let line = &mut self.lines[self.cursor.line];
        line.insert(self.cursor.col, ch);
        self.cursor.col += ch.len_utf8();
    }

    /// Delete the character before the cursor (backspace).
    pub fn delete_char(&mut self) {
        if self.cursor.col == 0 && self.cursor.line == 0 {
            return;
        }
        if self.cursor.col > 0 {
            let line = &self.lines[self.cursor.line];
            // Find the char boundary before cursor
            let before = &line[..self.cursor.col];
            let ch = before.chars().last().unwrap();
            let char_len = ch.len_utf8();
            self.undo_stack.push(EditorAction::DeleteChar {
                cursor: self.cursor,
                ch,
            });
            self.lines[self.cursor.line].remove(self.cursor.col - char_len);
            self.cursor.col -= char_len;
        } else {
            // Join with previous line
            let prev = self.cursor.line - 1;
            let join_col = self.lines[prev].len();
            let removed = self.lines.remove(self.cursor.line);
            self.undo_stack.push(EditorAction::DeleteCharJoin {
                cursor: self.cursor,
                removed_line: removed.clone(),
                join_col,
            });
            self.lines[prev].push_str(&removed);
            self.cursor.line = prev;
            self.cursor.col = join_col;
        }
    }

    /// Split the current line at the cursor (Enter key).
    pub fn insert_line(&mut self) {
        self.undo_stack.push(EditorAction::InsertLine {
            cursor: self.cursor,
        });
        let rest = self.lines[self.cursor.line][self.cursor.col..].to_string();
        self.lines[self.cursor.line].truncate(self.cursor.col);
        self.lines.insert(self.cursor.line + 1, rest);
        self.cursor.line += 1;
        self.cursor.col = 0;
    }

    /// Delete the current line entirely.
    pub fn delete_line(&mut self) {
        let content = self.lines[self.cursor.line].clone();
        let index = self.cursor.line;
        self.undo_stack.push(EditorAction::DeleteLine {
            cursor: self.cursor,
            content,
            index,
        });
        if self.lines.len() == 1 {
            self.lines[0].clear();
            self.cursor.col = 0;
        } else {
            self.lines.remove(self.cursor.line);
            if self.cursor.line >= self.lines.len() {
                self.cursor.line = self.lines.len() - 1;
            }
            self.cursor.col = 0;
        }
    }

    /// Move the cursor in a direction: "up", "down", "left", "right".
    pub fn move_cursor(&mut self, direction: &str) {
        match direction {
            "left" => {
                if self.cursor.col > 0 {
                    // Move back one char boundary
                    let before = &self.lines[self.cursor.line][..self.cursor.col];
                    if let Some(ch) = before.chars().last() {
                        self.cursor.col -= ch.len_utf8();
                    }
                } else if self.cursor.line > 0 {
                    self.cursor.line -= 1;
                    self.cursor.col = self.lines[self.cursor.line].len();
                }
            }
            "right" => {
                let line_len = self.lines[self.cursor.line].len();
                if self.cursor.col < line_len {
                    let ch = self.lines[self.cursor.line][self.cursor.col..]
                        .chars()
                        .next()
                        .unwrap();
                    self.cursor.col += ch.len_utf8();
                } else if self.cursor.line < self.lines.len() - 1 {
                    self.cursor.line += 1;
                    self.cursor.col = 0;
                }
            }
            "up" => {
                if self.cursor.line > 0 {
                    self.cursor.line -= 1;
                    self.clamp_cursor();
                }
            }
            "down" => {
                if self.cursor.line < self.lines.len() - 1 {
                    self.cursor.line += 1;
                    self.clamp_cursor();
                }
            }
            _ => {}
        }
    }

    /// Move cursor directly to a position.
    pub fn goto(&mut self, line: usize, col: usize) {
        self.cursor.line = line;
        self.cursor.col = col;
        self.clamp_cursor();
    }

    /// Get the content of line n.
    pub fn get_line(&self, n: usize) -> Option<&str> {
        self.lines.get(n).map(|s| s.as_str())
    }

    /// Get the full buffer content.
    pub fn get_text(&self) -> String {
        self.lines.join("\n")
    }

    /// Undo the last edit operation.
    pub fn undo(&mut self) {
        let action = match self.undo_stack.pop() {
            Some(a) => a,
            None => return,
        };
        match action {
            EditorAction::InsertChar { cursor, ch } => {
                self.lines[cursor.line].remove(cursor.col);
                let _ = ch; // char was at cursor.col
                self.cursor = cursor;
            }
            EditorAction::DeleteChar { cursor, ch } => {
                self.lines[cursor.line].insert(cursor.col - ch.len_utf8(), ch);
                self.cursor = cursor;
            }
            EditorAction::DeleteCharJoin {
                cursor,
                removed_line,
                join_col,
            } => {
                // Split the joined line back apart
                let combined = &self.lines[cursor.line - 1];
                let before = combined[..join_col].to_string();
                let _after = combined[join_col..].to_string();
                self.lines[cursor.line - 1] = before;
                self.lines.insert(cursor.line, removed_line);
                self.cursor = cursor;
            }
            EditorAction::InsertLine { cursor } => {
                // Rejoin the two lines
                if cursor.line + 1 < self.lines.len() {
                    let next = self.lines.remove(cursor.line + 1);
                    self.lines[cursor.line].push_str(&next);
                }
                self.cursor = cursor;
            }
            EditorAction::DeleteLine {
                cursor,
                content,
                index,
            } => {
                if self.lines.len() == 1 && self.lines[0].is_empty() {
                    self.lines[0] = content;
                } else {
                    self.lines.insert(index, content);
                }
                self.cursor = cursor;
            }
            EditorAction::Replace { cursor, old_lines } => {
                self.lines = old_lines;
                self.cursor = cursor;
            }
        }
    }

    /// Find all occurrences of a pattern. Returns vec of (line, col).
    pub fn find(&self, pattern: &str) -> Vec<(usize, usize)> {
        let mut results = Vec::new();
        for (i, line) in self.lines.iter().enumerate() {
            let mut start = 0;
            while let Some(idx) = line[start..].find(pattern) {
                results.push((i, start + idx));
                start += idx + 1;
            }
        }
        results
    }

    /// Replace occurrences of pattern with replacement.
    /// count=-1 replaces all. Returns number of replacements.
    pub fn replace(&mut self, pattern: &str, replacement: &str, count: i32) -> usize {
        self.undo_stack.push(EditorAction::Replace {
            cursor: self.cursor,
            old_lines: self.lines.clone(),
        });
        let mut total = 0usize;
        for i in 0..self.lines.len() {
            if count >= 0 && total >= count as usize {
                break;
            }
            let mut line = self.lines[i].clone();
            let mut new_line = String::new();
            let mut search_start = 0;
            while let Some(idx) = line[search_start..].find(pattern) {
                if count >= 0 && total >= count as usize {
                    new_line.push_str(&line[search_start..]);
                    search_start = line.len();
                    break;
                }
                new_line.push_str(&line[search_start..search_start + idx]);
                new_line.push_str(replacement);
                search_start += idx + pattern.len();
                total += 1;
            }
            new_line.push_str(&line[search_start..]);
            self.lines[i] = new_line;
        }
        if total == 0 {
            self.undo_stack.pop();
        }
        total
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_and_get_text() {
        let mut ed = TextEditor::new("");
        for ch in "hello".chars() {
            ed.insert_char(ch);
        }
        assert_eq!(ed.get_text(), "hello");
        assert_eq!(ed.cursor(), CursorPos { line: 0, col: 5 });
    }

    #[test]
    fn test_insert_line() {
        let mut ed = TextEditor::new("hello world");
        ed.goto(0, 5);
        ed.insert_line();
        assert_eq!(ed.line_count(), 2);
        assert_eq!(ed.get_line(0), Some("hello"));
        assert_eq!(ed.get_line(1), Some(" world"));
    }

    #[test]
    fn test_delete_char() {
        let mut ed = TextEditor::new("abc");
        ed.goto(0, 3);
        ed.delete_char();
        assert_eq!(ed.get_text(), "ab");
    }

    #[test]
    fn test_delete_char_join_lines() {
        let mut ed = TextEditor::new("ab\ncd");
        ed.goto(1, 0);
        ed.delete_char();
        assert_eq!(ed.get_text(), "abcd");
        assert_eq!(ed.cursor(), CursorPos { line: 0, col: 2 });
    }

    #[test]
    fn test_delete_line() {
        let mut ed = TextEditor::new("aaa\nbbb\nccc");
        ed.goto(1, 0);
        ed.delete_line();
        assert_eq!(ed.get_text(), "aaa\nccc");
    }

    #[test]
    fn test_move_cursor() {
        let mut ed = TextEditor::new("abc\ndef");
        ed.goto(0, 1);
        ed.move_cursor("right");
        assert_eq!(ed.cursor(), CursorPos { line: 0, col: 2 });
        ed.move_cursor("down");
        assert_eq!(ed.cursor(), CursorPos { line: 1, col: 2 });
        ed.move_cursor("left");
        assert_eq!(ed.cursor(), CursorPos { line: 1, col: 1 });
        ed.move_cursor("up");
        assert_eq!(ed.cursor(), CursorPos { line: 0, col: 1 });
    }

    #[test]
    fn test_undo_insert_char() {
        let mut ed = TextEditor::new("ab");
        ed.goto(0, 2);
        ed.insert_char('c');
        assert_eq!(ed.get_text(), "abc");
        ed.undo();
        assert_eq!(ed.get_text(), "ab");
    }

    #[test]
    fn test_undo_delete_char() {
        let mut ed = TextEditor::new("abc");
        ed.goto(0, 3);
        ed.delete_char();
        assert_eq!(ed.get_text(), "ab");
        ed.undo();
        assert_eq!(ed.get_text(), "abc");
    }

    #[test]
    fn test_undo_insert_line() {
        let mut ed = TextEditor::new("abcd");
        ed.goto(0, 2);
        ed.insert_line();
        assert_eq!(ed.get_text(), "ab\ncd");
        ed.undo();
        assert_eq!(ed.get_text(), "abcd");
    }

    #[test]
    fn test_undo_delete_line() {
        let mut ed = TextEditor::new("aaa\nbbb");
        ed.goto(0, 0);
        ed.delete_line();
        assert_eq!(ed.get_text(), "bbb");
        ed.undo();
        assert_eq!(ed.get_text(), "aaa\nbbb");
    }

    #[test]
    fn test_find() {
        let ed = TextEditor::new("the cat sat on the mat");
        let results = ed.find("the");
        assert_eq!(results, vec![(0, 0), (0, 15)]);
    }

    #[test]
    fn test_find_multiline() {
        let ed = TextEditor::new("fox here\nno fox there\nanother fox");
        let results = ed.find("fox");
        assert_eq!(results, vec![(0, 0), (1, 3), (2, 8)]);
    }

    #[test]
    fn test_replace_all() {
        let mut ed = TextEditor::new("the fox and the fox");
        let n = ed.replace("fox", "cat", -1);
        assert_eq!(n, 2);
        assert_eq!(ed.get_text(), "the cat and the cat");
    }

    #[test]
    fn test_replace_limited() {
        let mut ed = TextEditor::new("aaa bbb aaa bbb aaa");
        let n = ed.replace("aaa", "xxx", 2);
        assert_eq!(n, 2);
        assert_eq!(ed.get_text(), "xxx bbb xxx bbb aaa");
    }

    #[test]
    fn test_replace_undo() {
        let mut ed = TextEditor::new("foo bar foo");
        ed.replace("foo", "baz", -1);
        assert_eq!(ed.get_text(), "baz bar baz");
        ed.undo();
        assert_eq!(ed.get_text(), "foo bar foo");
    }

    #[test]
    fn test_new_with_multiline_text() {
        let ed = TextEditor::new("line one\nline two\nline three");
        assert_eq!(ed.line_count(), 3);
        assert_eq!(ed.get_line(1), Some("line two"));
    }
}
