//! Database Translation — dialect-aware DDL emission.
//!
//! Twelfth Databases project. Every team that's migrated between
//! MySQL and Postgres (or SQLite, or SQL Server) has hit this: the
//! *logical* schema is the same, but the *syntax* differs per dialect.
//! TINYINT becomes SMALLINT. AUTO_INCREMENT becomes SERIAL. NOW()
//! becomes CURRENT_TIMESTAMP. G112 captures the translation rules.
//!
//! The distinctive moves:
//!   * **Abstract type** — `ColumnType` (Integer, BigInt, Text,
//!     Varchar(n), Boolean, Timestamp, etc.) that every dialect
//!     renders differently.
//!   * **Dialect-specific rendering functions** for types, auto-
//!     increment, default-now.
//!   * **DDL emission** — `CREATE TABLE` statement per table, with
//!     primary keys, not-nulls, uniques, defaults, and auto-increment.
//!   * **Cross-dialect portability** — the same logical schema emits
//!     valid DDL for any supported dialect.

use std::fmt::Write;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dialect { MySql, Postgres, Sqlite }

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ColumnType {
    SmallInt,
    Integer,
    BigInt,
    Text,
    Varchar(u32),
    Boolean,
    Timestamp,
    Date,
    Float,
    Decimal(u32, u32),  // precision, scale
}

impl ColumnType {
    pub fn render(&self, dialect: Dialect) -> String {
        match (self, dialect) {
            (ColumnType::SmallInt, Dialect::MySql) => "TINYINT".into(),
            (ColumnType::SmallInt, Dialect::Postgres) => "SMALLINT".into(),
            (ColumnType::SmallInt, Dialect::Sqlite) => "INTEGER".into(),
            (ColumnType::Integer, _) => "INTEGER".into(),
            (ColumnType::BigInt, Dialect::Sqlite) => "INTEGER".into(),
            (ColumnType::BigInt, _) => "BIGINT".into(),
            (ColumnType::Text, _) => "TEXT".into(),
            (ColumnType::Varchar(n), _) => format!("VARCHAR({n})"),
            (ColumnType::Boolean, Dialect::MySql) => "TINYINT(1)".into(),
            (ColumnType::Boolean, Dialect::Postgres) => "BOOLEAN".into(),
            (ColumnType::Boolean, Dialect::Sqlite) => "INTEGER".into(),
            (ColumnType::Timestamp, Dialect::MySql) => "DATETIME".into(),
            (ColumnType::Timestamp, Dialect::Postgres) => "TIMESTAMP".into(),
            (ColumnType::Timestamp, Dialect::Sqlite) => "TEXT".into(),
            (ColumnType::Date, Dialect::Sqlite) => "TEXT".into(),
            (ColumnType::Date, _) => "DATE".into(),
            (ColumnType::Float, _) => "REAL".into(),
            (ColumnType::Decimal(p, s), _) => format!("DECIMAL({p},{s})"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DefaultValue {
    /// SQL literal, e.g. "'active'" (caller provides quoting).
    Literal(String),
    /// The current timestamp; dialect-specific.
    Now,
}

impl DefaultValue {
    pub fn render(&self, dialect: Dialect) -> String {
        match self {
            DefaultValue::Literal(s) => s.clone(),
            DefaultValue::Now => match dialect {
                Dialect::MySql => "NOW()".into(),
                Dialect::Postgres | Dialect::Sqlite => "CURRENT_TIMESTAMP".into(),
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DbColumn {
    pub name: String,
    pub ty: ColumnType,
    pub nullable: bool,
    pub primary_key: bool,
    pub unique: bool,
    pub auto_increment: bool,
    pub default: Option<DefaultValue>,
}

impl DbColumn {
    pub fn new(name: &str, ty: ColumnType) -> Self {
        Self {
            name: name.into(), ty,
            nullable: true, primary_key: false, unique: false,
            auto_increment: false, default: None,
        }
    }
    pub fn pk(mut self) -> Self { self.primary_key = true; self.nullable = false; self }
    pub fn not_null(mut self) -> Self { self.nullable = false; self }
    pub fn unique(mut self) -> Self { self.unique = true; self }
    pub fn auto_increment(mut self) -> Self { self.auto_increment = true; self }
    pub fn default(mut self, d: DefaultValue) -> Self { self.default = Some(d); self }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DbTranslatorTable {
    pub name: String,
    pub columns: Vec<DbColumn>,
}

fn render_auto_increment(dialect: Dialect) -> &'static str {
    match dialect {
        Dialect::MySql => "AUTO_INCREMENT",
        Dialect::Postgres => "GENERATED ALWAYS AS IDENTITY",
        Dialect::Sqlite => "AUTOINCREMENT",
    }
}

fn render_column(col: &DbColumn, dialect: Dialect) -> String {
    let mut out = String::new();
    // For Postgres with auto_increment on integer types, use SERIAL-family.
    // Actually the modern Postgres approach is IDENTITY; emit that.
    write!(&mut out, "{} {}", col.name, col.ty.render(dialect)).unwrap();
    if col.primary_key {
        out.push_str(" PRIMARY KEY");
    }
    if !col.nullable && !col.primary_key {
        out.push_str(" NOT NULL");
    }
    if col.unique && !col.primary_key {
        out.push_str(" UNIQUE");
    }
    if col.auto_increment {
        // Sqlite requires "INTEGER PRIMARY KEY AUTOINCREMENT".
        if dialect == Dialect::Sqlite && col.primary_key {
            out.push_str(" AUTOINCREMENT");
        } else if dialect != Dialect::Sqlite {
            out.push(' ');
            out.push_str(render_auto_increment(dialect));
        }
    }
    if let Some(d) = &col.default {
        out.push_str(&format!(" DEFAULT {}", d.render(dialect)));
    }
    out
}

/// Emit a `CREATE TABLE` statement for the given dialect.
pub fn emit_create_table(table: &DbTranslatorTable, dialect: Dialect) -> String {
    let mut out = String::new();
    writeln!(&mut out, "CREATE TABLE {} (", table.name).unwrap();
    let cols: Vec<String> = table.columns.iter()
        .map(|c| format!("  {}", render_column(c, dialect)))
        .collect();
    out.push_str(&cols.join(",\n"));
    out.push('\n');
    out.push_str(");\n");
    out
}

/// Emit CREATE TABLE for all tables in a given dialect, concatenated.
pub fn emit_schema(tables: &[DbTranslatorTable], dialect: Dialect) -> String {
    let mut out = String::new();
    for table in tables {
        out.push_str(&emit_create_table(table, dialect));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn users_table() -> DbTranslatorTable {
        DbTranslatorTable {
            name: "users".into(),
            columns: vec![
                DbColumn::new("id", ColumnType::Integer).pk().auto_increment(),
                DbColumn::new("email", ColumnType::Varchar(255)).not_null().unique(),
                DbColumn::new("name", ColumnType::Text).not_null(),
                DbColumn::new("active", ColumnType::Boolean).not_null()
                    .default(DefaultValue::Literal("1".into())),
                DbColumn::new("created_at", ColumnType::Timestamp).not_null()
                    .default(DefaultValue::Now),
            ],
        }
    }

    #[test]
    fn tinyint_is_mysql_specific() {
        assert_eq!(ColumnType::SmallInt.render(Dialect::MySql), "TINYINT");
        assert_eq!(ColumnType::SmallInt.render(Dialect::Postgres), "SMALLINT");
    }

    #[test]
    fn boolean_differs_per_dialect() {
        assert_eq!(ColumnType::Boolean.render(Dialect::MySql), "TINYINT(1)");
        assert_eq!(ColumnType::Boolean.render(Dialect::Postgres), "BOOLEAN");
        assert_eq!(ColumnType::Boolean.render(Dialect::Sqlite), "INTEGER");
    }

    #[test]
    fn timestamp_differs_per_dialect() {
        assert_eq!(ColumnType::Timestamp.render(Dialect::MySql), "DATETIME");
        assert_eq!(ColumnType::Timestamp.render(Dialect::Postgres), "TIMESTAMP");
        assert_eq!(ColumnType::Timestamp.render(Dialect::Sqlite), "TEXT");
    }

    #[test]
    fn varchar_preserves_length() {
        assert_eq!(ColumnType::Varchar(255).render(Dialect::MySql), "VARCHAR(255)");
    }

    #[test]
    fn decimal_preserves_precision_scale() {
        assert_eq!(ColumnType::Decimal(10, 2).render(Dialect::Postgres), "DECIMAL(10,2)");
    }

    #[test]
    fn now_default_differs_per_dialect() {
        assert_eq!(DefaultValue::Now.render(Dialect::MySql), "NOW()");
        assert_eq!(DefaultValue::Now.render(Dialect::Postgres), "CURRENT_TIMESTAMP");
    }

    #[test]
    fn mysql_create_table_uses_auto_increment() {
        let ddl = emit_create_table(&users_table(), Dialect::MySql);
        assert!(ddl.contains("AUTO_INCREMENT"));
        assert!(ddl.contains("NOW()"));
        assert!(ddl.contains("DATETIME"));
        assert!(ddl.contains("TINYINT(1)"));
    }

    #[test]
    fn postgres_create_table_uses_identity() {
        let ddl = emit_create_table(&users_table(), Dialect::Postgres);
        assert!(ddl.contains("GENERATED ALWAYS AS IDENTITY"));
        assert!(ddl.contains("CURRENT_TIMESTAMP"));
        assert!(ddl.contains("TIMESTAMP"));
        assert!(ddl.contains("BOOLEAN"));
    }

    #[test]
    fn sqlite_create_table_uses_autoincrement() {
        let ddl = emit_create_table(&users_table(), Dialect::Sqlite);
        assert!(ddl.contains("AUTOINCREMENT"));
        // Sqlite flattens booleans and timestamps to its native types.
        assert!(ddl.contains("INTEGER"));
    }

    #[test]
    fn primary_key_implies_not_null() {
        let table = DbTranslatorTable {
            name: "t".into(),
            columns: vec![
                DbColumn::new("id", ColumnType::Integer).pk(),
                DbColumn::new("name", ColumnType::Text).not_null(),
            ],
        };
        let ddl = emit_create_table(&table, Dialect::Postgres);
        // PK line has "PRIMARY KEY" but not separate "NOT NULL".
        let pk_line: &str = ddl.lines().find(|l| l.contains("id ")).unwrap();
        assert!(pk_line.contains("PRIMARY KEY"));
        assert!(!pk_line.contains("NOT NULL"));
    }

    #[test]
    fn unique_constraint_emitted() {
        let ddl = emit_create_table(&users_table(), Dialect::Postgres);
        assert!(ddl.contains("email VARCHAR(255) NOT NULL UNIQUE"));
    }

    #[test]
    fn default_literal_passes_through() {
        let ddl = emit_create_table(&users_table(), Dialect::Postgres);
        assert!(ddl.contains("DEFAULT 1"));
    }

    #[test]
    fn emit_schema_concatenates_tables() {
        let tables = vec![
            users_table(),
            DbTranslatorTable {
                name: "posts".into(),
                columns: vec![
                    DbColumn::new("id", ColumnType::Integer).pk().auto_increment(),
                    DbColumn::new("title", ColumnType::Text).not_null(),
                ],
            },
        ];
        let ddl = emit_schema(&tables, Dialect::Postgres);
        assert!(ddl.contains("CREATE TABLE users"));
        assert!(ddl.contains("CREATE TABLE posts"));
    }

    #[test]
    fn same_schema_emits_different_ddl_per_dialect() {
        let mysql = emit_create_table(&users_table(), Dialect::MySql);
        let pg = emit_create_table(&users_table(), Dialect::Postgres);
        assert_ne!(mysql, pg);
    }

    #[test]
    fn bigint_is_integer_in_sqlite() {
        assert_eq!(ColumnType::BigInt.render(Dialect::Sqlite), "INTEGER");
        assert_eq!(ColumnType::BigInt.render(Dialect::Postgres), "BIGINT");
    }

    #[test]
    fn date_is_text_in_sqlite() {
        assert_eq!(ColumnType::Date.render(Dialect::Sqlite), "TEXT");
        assert_eq!(ColumnType::Date.render(Dialect::Postgres), "DATE");
    }
}
