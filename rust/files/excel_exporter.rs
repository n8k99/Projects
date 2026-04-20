//! Excel Spreadsheet Exporter — cell-addressed grid with typed values and formulae.
//!
//! Eleventh Files project. A spreadsheet is a **sparse 2-D grid addressed
//! by (row, col)** holding typed cells (number, string, formula, blank).
//! The defining novel move: **formulae evaluate lazily with cycle
//! detection**. `=A1+B1` computes on read; circular references (`A1=B1,
//! B1=A1`) are caught and return an error value rather than looping.
//!
//! This is the minimum set that makes a spreadsheet a spreadsheet. Real
//! Excel adds: fonts, colours, merged cells, conditional formatting,
//! pivot tables, macros. The Rosetta Stone's job is the **calculation
//! model**.
//!
//! Export format: CSV (commas, quoted strings). Every real spreadsheet
//! can open CSV; we don't need to emit XLSX bytes to be useful.

use std::collections::{HashMap, HashSet};

/// Column letters A-Z are indices 0-25.
pub fn col_from_letter(letter: char) -> Option<u32> {
    let l = letter.to_ascii_uppercase();
    if l.is_ascii_alphabetic() { Some(l as u32 - 'A' as u32) }
    else { None }
}

pub fn col_to_letter(col: u32) -> char {
    char::from(b'A' + col as u8)
}

/// Parse an "A1" address into (row_idx_0, col_idx_0).
pub fn parse_address(addr: &str) -> Option<(u32, u32)> {
    let mut chars = addr.chars();
    let letter = chars.next()?;
    let col = col_from_letter(letter)?;
    let row_str: String = chars.collect();
    let row: u32 = row_str.parse().ok()?;
    if row == 0 { return None; }  // Excel is 1-indexed; row 0 invalid
    Some((row - 1, col))
}

pub fn format_address(row: u32, col: u32) -> String {
    format!("{}{}", col_to_letter(col), row + 1)
}

#[derive(Debug, Clone, PartialEq)]
pub enum Cell {
    Blank,
    Number(f64),
    Text(String),
    Formula(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Empty,
    Number(f64),
    Text(String),
    Error(String),
}

impl Value {
    pub fn as_number(&self) -> Option<f64> {
        match self {
            Value::Number(n) => Some(*n),
            Value::Text(s) => s.parse().ok(),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Sheet {
    cells: HashMap<(u32, u32), Cell>,
}

impl Sheet {
    pub fn new() -> Self { Self { cells: HashMap::new() } }

    pub fn set(&mut self, addr: &str, cell: Cell) -> Result<(), String> {
        let (row, col) = parse_address(addr).ok_or_else(|| format!("bad address: {addr}"))?;
        if matches!(cell, Cell::Blank) {
            self.cells.remove(&(row, col));
        } else {
            self.cells.insert((row, col), cell);
        }
        Ok(())
    }

    pub fn set_number(&mut self, addr: &str, n: f64) -> Result<(), String> {
        self.set(addr, Cell::Number(n))
    }
    pub fn set_text(&mut self, addr: &str, s: &str) -> Result<(), String> {
        self.set(addr, Cell::Text(s.into()))
    }
    pub fn set_formula(&mut self, addr: &str, f: &str) -> Result<(), String> {
        self.set(addr, Cell::Formula(f.into()))
    }

    pub fn raw(&self, addr: &str) -> Option<Cell> {
        let (row, col) = parse_address(addr)?;
        self.cells.get(&(row, col)).cloned()
    }

    pub fn evaluate(&self, addr: &str) -> Value {
        match parse_address(addr) {
            None => Value::Error(format!("bad address: {addr}")),
            Some((row, col)) => self.eval_cell(row, col, &mut HashSet::new()),
        }
    }

    fn eval_cell(&self, row: u32, col: u32, visiting: &mut HashSet<(u32, u32)>) -> Value {
        if !visiting.insert((row, col)) {
            return Value::Error("circular reference".into());
        }
        let result = match self.cells.get(&(row, col)) {
            None | Some(Cell::Blank) => Value::Empty,
            Some(Cell::Number(n)) => Value::Number(*n),
            Some(Cell::Text(s)) => Value::Text(s.clone()),
            Some(Cell::Formula(f)) => self.eval_formula(f, visiting),
        };
        visiting.remove(&(row, col));
        result
    }

    /// Tiny formula evaluator: `=OP(args)` or `=A1 OP B1` (+, -, *, /).
    fn eval_formula(&self, f: &str, visiting: &mut HashSet<(u32, u32)>) -> Value {
        let body = f.trim_start_matches('=').trim();

        // Built-in function: SUM(A1:A3), AVG(A1:A3), MIN, MAX
        if let Some(paren) = body.find('(') {
            if body.ends_with(')') {
                let fn_name = &body[..paren];
                let args = &body[paren + 1..body.len() - 1];
                return self.eval_function(fn_name, args, visiting);
            }
        }

        // Binary op: `A1 + B1`, `5 * C2`, etc.
        for op in ['+', '-', '*', '/'] {
            if let Some(pos) = body.find(op) {
                let lhs = body[..pos].trim();
                let rhs = body[pos + 1..].trim();
                let l = self.eval_operand(lhs, visiting);
                let r = self.eval_operand(rhs, visiting);
                return apply_op(op, l, r);
            }
        }

        // Just a cell reference.
        self.eval_operand(body, visiting)
    }

    fn eval_function(&self, name: &str, args: &str, visiting: &mut HashSet<(u32, u32)>) -> Value {
        let values = self.expand_range(args, visiting);
        let numbers: Vec<f64> = values.iter().filter_map(|v| v.as_number()).collect();
        match name {
            "SUM" => Value::Number(numbers.iter().sum()),
            "AVG" | "AVERAGE" => {
                if numbers.is_empty() { Value::Error("empty range for AVG".into()) }
                else { Value::Number(numbers.iter().sum::<f64>() / numbers.len() as f64) }
            }
            "MIN" => numbers.iter().cloned().fold(None, |acc, n| match acc {
                None => Some(n), Some(m) => Some(m.min(n)),
            }).map(Value::Number).unwrap_or_else(|| Value::Error("empty range".into())),
            "MAX" => numbers.iter().cloned().fold(None, |acc, n| match acc {
                None => Some(n), Some(m) => Some(m.max(n)),
            }).map(Value::Number).unwrap_or_else(|| Value::Error("empty range".into())),
            "COUNT" => Value::Number(numbers.len() as f64),
            other => Value::Error(format!("unknown function: {other}")),
        }
    }

    fn expand_range(&self, args: &str, visiting: &mut HashSet<(u32, u32)>) -> Vec<Value> {
        let mut out = Vec::new();
        for part in args.split(',') {
            let part = part.trim();
            if let Some(colon) = part.find(':') {
                let start = &part[..colon];
                let end = &part[colon + 1..];
                if let (Some((r1, c1)), Some((r2, c2))) = (parse_address(start), parse_address(end)) {
                    for r in r1..=r2 {
                        for c in c1..=c2 {
                            out.push(self.eval_cell(r, c, visiting));
                        }
                    }
                    continue;
                }
            }
            out.push(self.eval_operand(part, visiting));
        }
        out
    }

    fn eval_operand(&self, s: &str, visiting: &mut HashSet<(u32, u32)>) -> Value {
        let s = s.trim();
        if let Ok(n) = s.parse::<f64>() { return Value::Number(n); }
        if let Some((row, col)) = parse_address(s) {
            return self.eval_cell(row, col, visiting);
        }
        Value::Error(format!("bad operand: {s}"))
    }

    /// Export the sheet to CSV. Rows/cols are the bounding box of non-empty cells.
    pub fn to_csv(&self) -> String {
        if self.cells.is_empty() { return String::new(); }
        let max_row = self.cells.keys().map(|(r, _)| *r).max().unwrap();
        let max_col = self.cells.keys().map(|(_, c)| *c).max().unwrap();
        let mut out = String::new();
        for row in 0..=max_row {
            let mut row_cells = Vec::new();
            for col in 0..=max_col {
                let addr = format_address(row, col);
                let val = self.evaluate(&addr);
                row_cells.push(csv_escape(&format_value(&val)));
            }
            out.push_str(&row_cells.join(","));
            out.push('\n');
        }
        out
    }
}

impl Default for Sheet { fn default() -> Self { Self::new() } }

fn apply_op(op: char, l: Value, r: Value) -> Value {
    let la = l.as_number();
    let rb = r.as_number();
    match (la, rb) {
        (Some(a), Some(b)) => match op {
            '+' => Value::Number(a + b),
            '-' => Value::Number(a - b),
            '*' => Value::Number(a * b),
            '/' => if b == 0.0 { Value::Error("div by zero".into()) } else { Value::Number(a / b) },
            _ => Value::Error(format!("bad op: {op}")),
        },
        _ => Value::Error("non-numeric operand".into()),
    }
}

fn format_value(v: &Value) -> String {
    match v {
        Value::Empty => String::new(),
        Value::Number(n) => {
            if n.fract() == 0.0 && n.abs() < 1e15 { format!("{}", *n as i64) }
            else { format!("{n}") }
        }
        Value::Text(s) => s.clone(),
        Value::Error(e) => format!("#ERR: {e}"),
    }
}

fn csv_escape(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') {
        let escaped = s.replace('"', "\"\"");
        format!("\"{escaped}\"")
    } else {
        s.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn address_parsing() {
        assert_eq!(parse_address("A1"), Some((0, 0)));
        assert_eq!(parse_address("C3"), Some((2, 2)));
        assert_eq!(parse_address("Z99"), Some((98, 25)));
        assert_eq!(parse_address("A0"), None);
    }

    #[test]
    fn address_formatting() {
        assert_eq!(format_address(0, 0), "A1");
        assert_eq!(format_address(2, 2), "C3");
    }

    #[test]
    fn set_and_get_number() {
        let mut s = Sheet::new();
        s.set_number("A1", 42.0).unwrap();
        assert_eq!(s.evaluate("A1"), Value::Number(42.0));
    }

    #[test]
    fn empty_cell_evaluates_to_empty() {
        let s = Sheet::new();
        assert_eq!(s.evaluate("B5"), Value::Empty);
    }

    #[test]
    fn binary_op_formula() {
        let mut s = Sheet::new();
        s.set_number("A1", 10.0).unwrap();
        s.set_number("B1", 3.0).unwrap();
        s.set_formula("C1", "=A1 + B1").unwrap();
        assert_eq!(s.evaluate("C1"), Value::Number(13.0));
        s.set_formula("C2", "=A1 * B1").unwrap();
        assert_eq!(s.evaluate("C2"), Value::Number(30.0));
        s.set_formula("C3", "=A1 / B1").unwrap();
        match s.evaluate("C3") {
            Value::Number(n) => assert!((n - 3.333333).abs() < 0.001),
            _ => panic!("expected number"),
        }
    }

    #[test]
    fn circular_reference_detected() {
        let mut s = Sheet::new();
        s.set_formula("A1", "=B1 + 1").unwrap();
        s.set_formula("B1", "=A1 + 1").unwrap();
        assert!(matches!(s.evaluate("A1"), Value::Error(_)));
        assert!(matches!(s.evaluate("B1"), Value::Error(_)));
    }

    #[test]
    fn sum_over_range() {
        let mut s = Sheet::new();
        s.set_number("A1", 1.0).unwrap();
        s.set_number("A2", 2.0).unwrap();
        s.set_number("A3", 3.0).unwrap();
        s.set_number("A4", 4.0).unwrap();
        s.set_formula("B1", "=SUM(A1:A4)").unwrap();
        assert_eq!(s.evaluate("B1"), Value::Number(10.0));
    }

    #[test]
    fn avg_over_range() {
        let mut s = Sheet::new();
        s.set_number("A1", 10.0).unwrap();
        s.set_number("A2", 20.0).unwrap();
        s.set_number("A3", 30.0).unwrap();
        s.set_formula("B1", "=AVG(A1:A3)").unwrap();
        assert_eq!(s.evaluate("B1"), Value::Number(20.0));
    }

    #[test]
    fn min_max_over_range() {
        let mut s = Sheet::new();
        s.set_number("A1", 5.0).unwrap();
        s.set_number("A2", 2.0).unwrap();
        s.set_number("A3", 8.0).unwrap();
        s.set_formula("B1", "=MIN(A1:A3)").unwrap();
        s.set_formula("B2", "=MAX(A1:A3)").unwrap();
        assert_eq!(s.evaluate("B1"), Value::Number(2.0));
        assert_eq!(s.evaluate("B2"), Value::Number(8.0));
    }

    #[test]
    fn div_by_zero_is_error() {
        let mut s = Sheet::new();
        s.set_number("A1", 10.0).unwrap();
        s.set_number("B1", 0.0).unwrap();
        s.set_formula("C1", "=A1 / B1").unwrap();
        assert!(matches!(s.evaluate("C1"), Value::Error(_)));
    }

    #[test]
    fn csv_export_basic() {
        let mut s = Sheet::new();
        s.set_text("A1", "name").unwrap();
        s.set_text("B1", "age").unwrap();
        s.set_text("A2", "Alice").unwrap();
        s.set_number("B2", 30.0).unwrap();
        let csv = s.to_csv();
        assert!(csv.contains("name,age"));
        assert!(csv.contains("Alice,30"));
    }

    #[test]
    fn csv_quotes_fields_with_commas() {
        let mut s = Sheet::new();
        s.set_text("A1", "hello, world").unwrap();
        let csv = s.to_csv();
        assert!(csv.contains("\"hello, world\""));
    }

    #[test]
    fn csv_export_evaluates_formulae() {
        let mut s = Sheet::new();
        s.set_number("A1", 3.0).unwrap();
        s.set_number("B1", 4.0).unwrap();
        s.set_formula("C1", "=A1 + B1").unwrap();
        let csv = s.to_csv();
        assert!(csv.contains("3,4,7"));
    }

    #[test]
    fn raw_cell_preserves_formula_text() {
        let mut s = Sheet::new();
        s.set_formula("A1", "=1+2").unwrap();
        assert_eq!(s.raw("A1"), Some(Cell::Formula("=1+2".into())));
    }

    #[test]
    fn blank_cell_removes_from_store() {
        let mut s = Sheet::new();
        s.set_number("A1", 5.0).unwrap();
        s.set("A1", Cell::Blank).unwrap();
        assert_eq!(s.evaluate("A1"), Value::Empty);
    }
}
