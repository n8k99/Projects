//! SQL Query Analyzer — recursive-descent parser + static analysis over a SQL subset.
//!
//! First Databases project. Every database ships one of these: a parser
//! that turns SQL text into an AST, then a static analyser that reports
//! what the query touches (tables, columns), what's wrong with it
//! (unknown columns, missing WHERE, LIMIT without ORDER), and a cost hint.
//!
//! The SQL subset:
//!   `SELECT col[, col...] FROM table [WHERE col op value] [ORDER BY col [ASC|DESC]] [LIMIT n]`
//!
//! This is the *shape* of SQL every developer intuits before they learn
//! the full grammar. G101 formalises it.
//!
//! Distinctive move: **parsing and analysis are separate phases**. The
//! parser produces an AST without judging; the analyser walks the AST and
//! produces a list of findings. Same pattern as every real SQL engine
//! (Postgres, SQLite, MySQL, DuckDB).

use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    Word(String),       // identifier or keyword
    Number(i64),
    String(String),
    Op(String),         // =, !=, <, >, <=, >=
    Comma,
    Star,
    Semicolon,
}

/// Tokenise a SQL-ish input. Keywords are lowercased to `Word`.
pub fn tokenise(input: &str) -> Result<Vec<Token>, String> {
    let mut tokens = Vec::new();
    let chars: Vec<char> = input.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if c.is_whitespace() { i += 1; continue; }
        if c == ',' { tokens.push(Token::Comma); i += 1; continue; }
        if c == ';' { tokens.push(Token::Semicolon); i += 1; continue; }
        if c == '*' { tokens.push(Token::Star); i += 1; continue; }
        if c == '\'' {
            let mut s = String::new();
            i += 1;
            while i < chars.len() && chars[i] != '\'' {
                s.push(chars[i]);
                i += 1;
            }
            if i >= chars.len() { return Err("unterminated string".into()); }
            i += 1;
            tokens.push(Token::String(s));
            continue;
        }
        if c == '=' { tokens.push(Token::Op("=".into())); i += 1; continue; }
        if c == '<' || c == '>' || c == '!' {
            let mut op = String::new();
            op.push(c);
            i += 1;
            if i < chars.len() && chars[i] == '=' {
                op.push('=');
                i += 1;
            }
            if op == "!" { return Err("bad operator: !".into()); }
            tokens.push(Token::Op(op));
            continue;
        }
        if c.is_ascii_digit() || (c == '-' && i + 1 < chars.len() && chars[i + 1].is_ascii_digit()) {
            let start = i;
            if c == '-' { i += 1; }
            while i < chars.len() && chars[i].is_ascii_digit() { i += 1; }
            let s: String = chars[start..i].iter().collect();
            let n: i64 = s.parse().map_err(|_| format!("bad number: {s}"))?;
            tokens.push(Token::Number(n));
            continue;
        }
        if c.is_ascii_alphabetic() || c == '_' {
            let start = i;
            while i < chars.len() && (chars[i].is_ascii_alphanumeric() || chars[i] == '_') {
                i += 1;
            }
            let word: String = chars[start..i].iter().collect::<String>().to_lowercase();
            tokens.push(Token::Word(word));
            continue;
        }
        return Err(format!("unexpected char: {c}"));
    }
    Ok(tokens)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Projection {
    All,
    Columns(Vec<String>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    Num(i64),
    Str(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WhereClause {
    pub column: String,
    pub op: String,
    pub value: Value,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Order { Asc, Desc }

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrderBy {
    pub column: String,
    pub order: Order,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Query {
    pub projection: Projection,
    pub table: String,
    pub where_clause: Option<WhereClause>,
    pub order_by: Option<OrderBy>,
    pub limit: Option<u32>,
}

/// Recursive-descent parser. Returns a Query or a parse error.
pub fn parse(input: &str) -> Result<Query, String> {
    let mut tokens = tokenise(input)?;
    // Accept an optional trailing semicolon.
    if matches!(tokens.last(), Some(Token::Semicolon)) { tokens.pop(); }
    let mut p = Parser { tokens, pos: 0 };
    let q = p.parse_query()?;
    if p.pos < p.tokens.len() {
        return Err(format!("trailing tokens after query"));
    }
    Ok(q)
}

struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    fn peek(&self) -> Option<&Token> { self.tokens.get(self.pos) }
    fn bump(&mut self) -> Option<Token> {
        let t = self.tokens.get(self.pos).cloned();
        self.pos += 1;
        t
    }
    fn expect_word(&mut self, w: &str) -> Result<(), String> {
        match self.bump() {
            Some(Token::Word(s)) if s == w => Ok(()),
            Some(t) => Err(format!("expected '{w}', got {t:?}")),
            None => Err(format!("expected '{w}', got end of input")),
        }
    }

    fn parse_query(&mut self) -> Result<Query, String> {
        self.expect_word("select")?;
        let projection = self.parse_projection()?;
        self.expect_word("from")?;
        let table = self.parse_identifier()?;
        let where_clause = if matches!(self.peek(), Some(Token::Word(w)) if w == "where") {
            self.bump();
            Some(self.parse_where()?)
        } else { None };
        let order_by = if matches!(self.peek(), Some(Token::Word(w)) if w == "order") {
            self.bump();
            self.expect_word("by")?;
            Some(self.parse_order_by()?)
        } else { None };
        let limit = if matches!(self.peek(), Some(Token::Word(w)) if w == "limit") {
            self.bump();
            match self.bump() {
                Some(Token::Number(n)) if n >= 0 => Some(n as u32),
                other => return Err(format!("expected positive number after LIMIT, got {other:?}")),
            }
        } else { None };
        Ok(Query { projection, table, where_clause, order_by, limit })
    }

    fn parse_projection(&mut self) -> Result<Projection, String> {
        if matches!(self.peek(), Some(Token::Star)) {
            self.bump();
            return Ok(Projection::All);
        }
        let mut cols = vec![self.parse_identifier()?];
        while matches!(self.peek(), Some(Token::Comma)) {
            self.bump();
            cols.push(self.parse_identifier()?);
        }
        Ok(Projection::Columns(cols))
    }

    fn parse_identifier(&mut self) -> Result<String, String> {
        match self.bump() {
            Some(Token::Word(s)) => Ok(s),
            other => Err(format!("expected identifier, got {other:?}")),
        }
    }

    fn parse_where(&mut self) -> Result<WhereClause, String> {
        let column = self.parse_identifier()?;
        let op = match self.bump() {
            Some(Token::Op(s)) => s,
            other => return Err(format!("expected operator, got {other:?}")),
        };
        let value = match self.bump() {
            Some(Token::Number(n)) => Value::Num(n),
            Some(Token::String(s)) => Value::Str(s),
            other => return Err(format!("expected value, got {other:?}")),
        };
        Ok(WhereClause { column, op, value })
    }

    fn parse_order_by(&mut self) -> Result<OrderBy, String> {
        let column = self.parse_identifier()?;
        let order = match self.peek() {
            Some(Token::Word(w)) if w == "asc" => { self.bump(); Order::Asc }
            Some(Token::Word(w)) if w == "desc" => { self.bump(); Order::Desc }
            _ => Order::Asc,
        };
        Ok(OrderBy { column, order })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Analysis {
    pub tables: BTreeSet<String>,
    pub columns: BTreeSet<String>,
    pub findings: Vec<Finding>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Severity { Info, Warning, Error }

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Finding {
    pub severity: Severity,
    pub code: String,
    pub message: String,
}

/// Known schema passed to the analyser. Maps table name → column set.
#[derive(Debug, Clone, Default)]
pub struct Schema {
    pub tables: std::collections::BTreeMap<String, BTreeSet<String>>,
}

impl Schema {
    pub fn new() -> Self { Self::default() }
    pub fn add_table(&mut self, name: &str, columns: &[&str]) {
        self.tables.insert(
            name.to_string(),
            columns.iter().map(|c| c.to_string()).collect(),
        );
    }
}

/// Analyse a parsed query against a known schema.
pub fn analyse(query: &Query, schema: &Schema) -> Analysis {
    let mut tables = BTreeSet::new();
    let mut columns = BTreeSet::new();
    let mut findings = Vec::new();

    tables.insert(query.table.clone());
    let known = schema.tables.get(&query.table);
    if known.is_none() {
        findings.push(Finding {
            severity: Severity::Error,
            code: "UNKNOWN_TABLE".into(),
            message: format!("unknown table: {}", query.table),
        });
    }

    let referenced_columns = collect_referenced_columns(query);
    for col in &referenced_columns {
        columns.insert(col.clone());
        if let Some(k) = known {
            if !k.contains(col) {
                findings.push(Finding {
                    severity: Severity::Error,
                    code: "UNKNOWN_COLUMN".into(),
                    message: format!("unknown column {col} in table {}", query.table),
                });
            }
        }
    }

    // SELECT *
    if matches!(query.projection, Projection::All) {
        findings.push(Finding {
            severity: Severity::Warning,
            code: "SELECT_STAR".into(),
            message: "SELECT * pulls every column; name them explicitly".into(),
        });
    }

    // LIMIT without ORDER BY → nondeterministic.
    if query.limit.is_some() && query.order_by.is_none() {
        findings.push(Finding {
            severity: Severity::Warning,
            code: "LIMIT_WITHOUT_ORDER".into(),
            message: "LIMIT without ORDER BY returns arbitrary rows".into(),
        });
    }

    // Missing WHERE on a large table (can't know size here — just inform).
    if query.where_clause.is_none() && query.limit.is_none() {
        findings.push(Finding {
            severity: Severity::Info,
            code: "FULL_SCAN".into(),
            message: "no WHERE or LIMIT — query will scan every row".into(),
        });
    }

    Analysis { tables, columns, findings }
}

fn collect_referenced_columns(q: &Query) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    match &q.projection {
        Projection::All => {}
        Projection::Columns(cols) => {
            for c in cols { out.insert(c.clone()); }
        }
    }
    if let Some(w) = &q.where_clause { out.insert(w.column.clone()); }
    if let Some(o) = &q.order_by { out.insert(o.column.clone()); }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_schema() -> Schema {
        let mut s = Schema::new();
        s.add_table("users", &["id", "name", "email", "created_at"]);
        s.add_table("orders", &["id", "user_id", "amount", "status"]);
        s
    }

    #[test]
    fn tokenise_keywords_lowercase() {
        let t = tokenise("SELECT * FROM Users").unwrap();
        assert_eq!(t[0], Token::Word("select".into()));
        assert_eq!(t[1], Token::Star);
        assert_eq!(t[2], Token::Word("from".into()));
        assert_eq!(t[3], Token::Word("users".into()));
    }

    #[test]
    fn tokenise_string_literal() {
        let t = tokenise("WHERE name = 'Alice'").unwrap();
        assert!(t.contains(&Token::String("Alice".into())));
    }

    #[test]
    fn tokenise_operators() {
        let t = tokenise("a <= 5 AND b != 'x'").unwrap();
        assert!(t.contains(&Token::Op("<=".into())));
        assert!(t.contains(&Token::Op("!=".into())));
    }

    #[test]
    fn parse_simple_select_star() {
        let q = parse("SELECT * FROM users").unwrap();
        assert_eq!(q.projection, Projection::All);
        assert_eq!(q.table, "users");
        assert!(q.where_clause.is_none());
    }

    #[test]
    fn parse_with_columns_and_where() {
        let q = parse("SELECT id, name FROM users WHERE id = 42").unwrap();
        assert_eq!(q.projection, Projection::Columns(vec!["id".into(), "name".into()]));
        assert_eq!(q.where_clause.as_ref().unwrap().column, "id");
        assert_eq!(q.where_clause.as_ref().unwrap().op, "=");
        assert_eq!(q.where_clause.as_ref().unwrap().value, Value::Num(42));
    }

    #[test]
    fn parse_with_order_and_limit() {
        let q = parse("SELECT name FROM users ORDER BY created_at DESC LIMIT 10").unwrap();
        assert_eq!(q.order_by.as_ref().unwrap().column, "created_at");
        assert_eq!(q.order_by.as_ref().unwrap().order, Order::Desc);
        assert_eq!(q.limit, Some(10));
    }

    #[test]
    fn parse_trailing_semicolon_accepted() {
        let q = parse("SELECT * FROM users;").unwrap();
        assert_eq!(q.table, "users");
    }

    #[test]
    fn parse_malformed_errors() {
        assert!(parse("SELECT").is_err());
        assert!(parse("SELECT * users").is_err());
        assert!(parse("SELECT FROM users").is_err());
    }

    #[test]
    fn analyse_unknown_table_flags_error() {
        let schema = sample_schema();
        let q = parse("SELECT * FROM widgets").unwrap();
        let a = analyse(&q, &schema);
        assert!(a.findings.iter().any(|f| f.code == "UNKNOWN_TABLE"));
    }

    #[test]
    fn analyse_unknown_column_flags_error() {
        let schema = sample_schema();
        let q = parse("SELECT id, birthday FROM users").unwrap();
        let a = analyse(&q, &schema);
        assert!(a.findings.iter().any(|f|
            f.code == "UNKNOWN_COLUMN" && f.message.contains("birthday")));
    }

    #[test]
    fn analyse_select_star_warning() {
        let schema = sample_schema();
        let q = parse("SELECT * FROM users").unwrap();
        let a = analyse(&q, &schema);
        assert!(a.findings.iter().any(|f| f.code == "SELECT_STAR"));
    }

    #[test]
    fn analyse_limit_without_order_warning() {
        let schema = sample_schema();
        let q = parse("SELECT name FROM users LIMIT 10").unwrap();
        let a = analyse(&q, &schema);
        assert!(a.findings.iter().any(|f| f.code == "LIMIT_WITHOUT_ORDER"));
    }

    #[test]
    fn analyse_full_scan_info() {
        let schema = sample_schema();
        let q = parse("SELECT name FROM users").unwrap();
        let a = analyse(&q, &schema);
        assert!(a.findings.iter().any(|f| f.code == "FULL_SCAN"));
    }

    #[test]
    fn analyse_clean_query_has_no_warnings() {
        let schema = sample_schema();
        let q = parse("SELECT id, name FROM users WHERE id = 42 ORDER BY name LIMIT 10").unwrap();
        let a = analyse(&q, &schema);
        assert!(a.findings.iter().all(|f| f.severity != Severity::Error
                                         && f.severity != Severity::Warning));
    }

    #[test]
    fn analyse_collects_referenced_columns() {
        let schema = sample_schema();
        let q = parse("SELECT name FROM users WHERE email = 'x' ORDER BY created_at").unwrap();
        let a = analyse(&q, &schema);
        assert!(a.columns.contains("name"));
        assert!(a.columns.contains("email"));
        assert!(a.columns.contains("created_at"));
    }

    #[test]
    fn analyse_tables_set() {
        let schema = sample_schema();
        let q = parse("SELECT * FROM orders").unwrap();
        let a = analyse(&q, &schema);
        assert!(a.tables.contains("orders"));
    }

    #[test]
    fn parse_string_where_value() {
        let q = parse("SELECT * FROM users WHERE name = 'Alice'").unwrap();
        assert_eq!(q.where_clause.unwrap().value, Value::Str("Alice".into()));
    }
}
