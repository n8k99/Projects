//! Database Backup Script Maker — code generation with shell-safe quoting.
//!
//! Fifth Databases project. Produces executable **bash scripts** from a
//! `BackupSpec` — database name, tables, destination, compression flag,
//! and a few options. Introduces the Rosetta Stone's first **code-
//! generator**: structured input → text output that another interpreter
//! (bash) runs.
//!
//! The hard problems are **quoting** and **validation**:
//!   * Shell values MUST be wrapped in single quotes with embedded
//!      single quotes escaped via `'\''`.
//!   * Table names MUST match `[a-zA-Z_][a-zA-Z0-9_]*` (else reject —
//!      no injection via names).
//!   * The spec is validated before any script emission. Invalid specs
//!      yield structured errors; valid specs yield a deterministic
//!      script.
//!
//! Not tested: actually running the script. That's the executor's job.
//! G105's contract is "given this config, produce this exact script
//! text", verifiable via string equality.

use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Format { Sql, CsvPerTable }

#[derive(Debug, Clone)]
pub struct BackupSpec {
    pub database: String,
    pub host: String,
    pub port: u16,
    pub user: String,
    pub tables_include: Vec<String>,   // empty = all
    pub tables_exclude: Vec<String>,
    pub destination_dir: String,
    pub compress: bool,
    pub format: Format,
}

impl BackupSpec {
    pub fn new(database: &str) -> Self {
        Self {
            database: database.into(),
            host: "localhost".into(),
            port: 5432,
            user: "postgres".into(),
            tables_include: vec![],
            tables_exclude: vec![],
            destination_dir: "/var/backups".into(),
            compress: true,
            format: Format::Sql,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationError {
    EmptyDatabase,
    InvalidIdentifier(String),
    EmptyDestination,
    PortZero,
}

/// Validate a `BackupSpec` before emission. Returns a list of errors
/// (empty = OK).
pub fn validate(spec: &BackupSpec) -> Vec<ValidationError> {
    let mut errors = Vec::new();
    if spec.database.is_empty() { errors.push(ValidationError::EmptyDatabase); }
    if spec.destination_dir.is_empty() { errors.push(ValidationError::EmptyDestination); }
    if spec.port == 0 { errors.push(ValidationError::PortZero); }
    if !is_valid_identifier(&spec.database) {
        errors.push(ValidationError::InvalidIdentifier(spec.database.clone()));
    }
    for t in spec.tables_include.iter().chain(spec.tables_exclude.iter()) {
        if !is_valid_identifier(t) {
            errors.push(ValidationError::InvalidIdentifier(t.clone()));
        }
    }
    errors
}

fn is_valid_identifier(s: &str) -> bool {
    let mut chars = s.chars();
    match chars.next() {
        Some(c) if c.is_ascii_alphabetic() || c == '_' => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// Wrap a string in single quotes, escaping embedded single quotes via `'\''`.
pub fn shell_quote(s: &str) -> String {
    let escaped = s.replace('\'', "'\\''");
    format!("'{escaped}'")
}

/// Emit the full bash script. Returns Err with validation errors if invalid.
pub fn emit_script(spec: &BackupSpec) -> Result<String, Vec<ValidationError>> {
    let errors = validate(spec);
    if !errors.is_empty() { return Err(errors); }

    let mut out = String::new();
    out.push_str("#!/usr/bin/env bash\n");
    out.push_str("set -euo pipefail\n\n");
    out.push_str(&format!("DB={}\n", shell_quote(&spec.database)));
    out.push_str(&format!("HOST={}\n", shell_quote(&spec.host)));
    out.push_str(&format!("PORT={}\n", spec.port));
    out.push_str(&format!("USER={}\n", shell_quote(&spec.user)));
    out.push_str(&format!("DEST={}\n", shell_quote(&spec.destination_dir)));
    out.push_str("STAMP=$(date -u +%Y%m%dT%H%M%SZ)\n");
    out.push_str("mkdir -p \"$DEST\"\n\n");

    match spec.format {
        Format::Sql => {
            let mut cmd = String::from("pg_dump -h \"$HOST\" -p \"$PORT\" -U \"$USER\" \"$DB\"");
            // Include/exclude flags.
            let mut includes: BTreeSet<&String> = spec.tables_include.iter().collect();
            for t in &spec.tables_exclude {
                includes.remove(t);
                cmd.push_str(&format!(" --exclude-table={}", shell_quote(t)));
            }
            for t in &includes {
                cmd.push_str(&format!(" --table={}", shell_quote(t)));
            }
            if spec.compress {
                out.push_str(&format!("{cmd} | gzip > \"$DEST/${{DB}}_${{STAMP}}.sql.gz\"\n"));
            } else {
                out.push_str(&format!("{cmd} > \"$DEST/${{DB}}_${{STAMP}}.sql\"\n"));
            }
        }
        Format::CsvPerTable => {
            let tables: Vec<String> = if spec.tables_include.is_empty() {
                // Placeholder — would be introspected at runtime.
                vec!["$(psql -h \"$HOST\" -p \"$PORT\" -U \"$USER\" -t -c \"\\\\dt\" \"$DB\" | awk '{print $3}')".into()]
            } else {
                spec.tables_include.iter()
                    .filter(|t| !spec.tables_exclude.contains(t))
                    .map(|t| shell_quote(t))
                    .collect()
            };
            for t in &tables {
                let base = format!("psql -h \"$HOST\" -p \"$PORT\" -U \"$USER\" -c \"\\\\copy {t} TO STDOUT WITH CSV HEADER\" \"$DB\"");
                if spec.compress {
                    out.push_str(&format!("{base} | gzip > \"$DEST/${{DB}}_{t}_${{STAMP}}.csv.gz\"\n"));
                } else {
                    out.push_str(&format!("{base} > \"$DEST/${{DB}}_{t}_${{STAMP}}.csv\"\n"));
                }
            }
        }
    }
    out.push_str("\necho \"backup complete: $DEST\"\n");
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_identifier_accepts() {
        assert!(is_valid_identifier("users"));
        assert!(is_valid_identifier("_underscore"));
        assert!(is_valid_identifier("order_lines"));
    }

    #[test]
    fn valid_identifier_rejects() {
        assert!(!is_valid_identifier(""));
        assert!(!is_valid_identifier("1users"));
        assert!(!is_valid_identifier("has space"));
        assert!(!is_valid_identifier("drop; --"));
        assert!(!is_valid_identifier("user'name"));
    }

    #[test]
    fn shell_quote_basic() {
        assert_eq!(shell_quote("hello"), "'hello'");
    }

    #[test]
    fn shell_quote_with_spaces() {
        assert_eq!(shell_quote("hello world"), "'hello world'");
    }

    #[test]
    fn shell_quote_with_embedded_quote() {
        assert_eq!(shell_quote("it's"), "'it'\\''s'");
    }

    #[test]
    fn shell_quote_empty() {
        assert_eq!(shell_quote(""), "''");
    }

    #[test]
    fn validate_rejects_empty_database() {
        let mut s = BackupSpec::new("x");
        s.database = String::new();
        let e = validate(&s);
        assert!(e.contains(&ValidationError::EmptyDatabase));
    }

    #[test]
    fn validate_rejects_bad_identifier() {
        let s = BackupSpec::new("drop; --");
        let e = validate(&s);
        assert!(e.iter().any(|err| matches!(err, ValidationError::InvalidIdentifier(_))));
    }

    #[test]
    fn validate_rejects_bad_table_names() {
        let mut s = BackupSpec::new("ok");
        s.tables_include.push("drop; --".into());
        let e = validate(&s);
        assert!(e.iter().any(|err| matches!(err, ValidationError::InvalidIdentifier(_))));
    }

    #[test]
    fn validate_ok_when_clean() {
        let s = BackupSpec::new("appdb");
        assert!(validate(&s).is_empty());
    }

    #[test]
    fn emit_script_happy_path() {
        let mut s = BackupSpec::new("mydb");
        s.compress = true;
        let script = emit_script(&s).unwrap();
        assert!(script.starts_with("#!/usr/bin/env bash\n"));
        assert!(script.contains("DB='mydb'"));
        assert!(script.contains("pg_dump"));
        assert!(script.contains("gzip"));
        assert!(script.contains(".sql.gz"));
    }

    #[test]
    fn emit_script_uncompressed() {
        let mut s = BackupSpec::new("mydb");
        s.compress = false;
        let script = emit_script(&s).unwrap();
        assert!(!script.contains("gzip"));
        assert!(script.contains(".sql\""));
    }

    #[test]
    fn emit_script_includes_tables() {
        let mut s = BackupSpec::new("mydb");
        s.tables_include = vec!["users".into(), "orders".into()];
        let script = emit_script(&s).unwrap();
        assert!(script.contains("--table='users'"));
        assert!(script.contains("--table='orders'"));
    }

    #[test]
    fn emit_script_excludes_tables() {
        let mut s = BackupSpec::new("mydb");
        s.tables_exclude = vec!["logs".into(), "sessions".into()];
        let script = emit_script(&s).unwrap();
        assert!(script.contains("--exclude-table='logs'"));
        assert!(script.contains("--exclude-table='sessions'"));
    }

    #[test]
    fn emit_script_csv_format() {
        let mut s = BackupSpec::new("mydb");
        s.format = Format::CsvPerTable;
        s.tables_include = vec!["users".into()];
        let script = emit_script(&s).unwrap();
        assert!(script.contains("psql"));
        assert!(script.contains("\\\\copy"));
        assert!(script.contains(".csv.gz"));
    }

    #[test]
    fn emit_script_fails_on_invalid_spec() {
        let s = BackupSpec::new("");
        assert!(emit_script(&s).is_err());
    }

    #[test]
    fn emit_script_deterministic() {
        let s = BackupSpec::new("mydb");
        assert_eq!(emit_script(&s), emit_script(&s));
    }
}
