//! Remote SQL Tool — in-memory table engine + execution over a parsed AST.
//!
//! Second Databases project. G101 built the parser; G102 builds the
//! executor. A `Connection` holds tables; tables have typed columns;
//! rows are inserted; SELECTs evaluate by scanning and filtering.
//! Transactions buffer changes until COMMIT (or discard via ROLLBACK).
//!
//! This is the minimum shape of every embedded DB engine — SQLite,
//! DuckDB, an in-memory Postgres. Real engines add indexes, joins,
//! query planning, constraints. The Rosetta Stone version captures
//! the core: store, filter, project, sort, limit.
//!
//! Design choices worth noting:
//!   * **Column types** are discrete (Int, Text) — no silent coercion.
//!   * **Row storage** is a list of Values matching column order.
//!   * **Evaluation** walks rows linearly; no indexes (future work).
//!   * **Transactions** buffer inserted rows; commit appends, rollback discards.

use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColumnType { Int, Text }

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CellValue {
    Int(i64),
    Text(String),
    Null,
}

impl CellValue {
    pub fn type_of(&self) -> Option<ColumnType> {
        match self {
            CellValue::Int(_) => Some(ColumnType::Int),
            CellValue::Text(_) => Some(ColumnType::Text),
            CellValue::Null => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Column {
    pub name: String,
    pub ty: ColumnType,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Table {
    pub name: String,
    pub columns: Vec<Column>,
    pub rows: Vec<Vec<CellValue>>,
}

impl Table {
    pub fn column_index(&self, name: &str) -> Option<usize> {
        self.columns.iter().position(|c| c.name == name)
    }

    pub fn column_names(&self) -> Vec<String> {
        self.columns.iter().map(|c| c.name.clone()).collect()
    }
}

#[derive(Debug, Clone)]
pub struct Connection {
    tables: BTreeMap<String, Table>,
    pending: Option<BTreeMap<String, Vec<Vec<CellValue>>>>,  // table -> pending rows
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExecError {
    TableExists(String),
    UnknownTable(String),
    UnknownColumn(String),
    TypeMismatch { column: String, expected: ColumnType, got: Option<ColumnType> },
    NoTransaction,
    BadValue(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResultSet {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<CellValue>>,
}

impl Connection {
    pub fn new() -> Self {
        Self { tables: BTreeMap::new(), pending: None }
    }

    pub fn create_table(&mut self, name: &str, columns: Vec<Column>) -> Result<(), ExecError> {
        if self.tables.contains_key(name) {
            return Err(ExecError::TableExists(name.into()));
        }
        self.tables.insert(name.into(), Table {
            name: name.into(),
            columns,
            rows: Vec::new(),
        });
        Ok(())
    }

    pub fn insert(&mut self, table: &str, values: Vec<CellValue>) -> Result<(), ExecError> {
        let t = self.tables.get(table)
            .ok_or_else(|| ExecError::UnknownTable(table.into()))?;
        if values.len() != t.columns.len() {
            return Err(ExecError::BadValue(
                format!("expected {} values, got {}", t.columns.len(), values.len())));
        }
        for (col, val) in t.columns.iter().zip(&values) {
            if let Some(vt) = val.type_of() {
                if vt != col.ty {
                    return Err(ExecError::TypeMismatch {
                        column: col.name.clone(),
                        expected: col.ty, got: Some(vt),
                    });
                }
            }
        }
        // If a transaction is open, buffer; else append directly.
        if let Some(pending) = &mut self.pending {
            pending.entry(table.into()).or_default().push(values);
        } else {
            self.tables.get_mut(table).unwrap().rows.push(values);
        }
        Ok(())
    }

    pub fn begin(&mut self) {
        self.pending = Some(BTreeMap::new());
    }

    pub fn commit(&mut self) -> Result<(), ExecError> {
        let pending = self.pending.take()
            .ok_or(ExecError::NoTransaction)?;
        for (name, rows) in pending {
            if let Some(t) = self.tables.get_mut(&name) {
                t.rows.extend(rows);
            }
        }
        Ok(())
    }

    pub fn rollback(&mut self) -> Result<(), ExecError> {
        if self.pending.is_none() {
            return Err(ExecError::NoTransaction);
        }
        self.pending = None;
        Ok(())
    }

    pub fn in_transaction(&self) -> bool { self.pending.is_some() }

    /// Row view over a table including pending rows.
    fn rows_of<'a>(&'a self, table: &str) -> Vec<&'a Vec<CellValue>> {
        let mut out: Vec<&Vec<CellValue>> = self.tables.get(table)
            .map(|t| t.rows.iter().collect())
            .unwrap_or_default();
        if let Some(pending) = &self.pending {
            if let Some(p) = pending.get(table) {
                out.extend(p.iter());
            }
        }
        out
    }

    pub fn table(&self, name: &str) -> Option<&Table> { self.tables.get(name) }

    pub fn select(&self, q: &SelectQuery) -> Result<ResultSet, ExecError> {
        let table = self.tables.get(&q.table)
            .ok_or_else(|| ExecError::UnknownTable(q.table.clone()))?;

        // Resolve projection columns.
        let proj_indices: Vec<usize> = if q.project_all {
            (0..table.columns.len()).collect()
        } else {
            let mut idxs = Vec::with_capacity(q.columns.len());
            for c in &q.columns {
                let i = table.column_index(c)
                    .ok_or_else(|| ExecError::UnknownColumn(c.clone()))?;
                idxs.push(i);
            }
            idxs
        };

        // Validate WHERE column.
        let where_idx = if let Some(w) = &q.where_clause {
            Some(table.column_index(&w.column)
                .ok_or_else(|| ExecError::UnknownColumn(w.column.clone()))?)
        } else { None };

        // Validate ORDER BY column.
        let order_idx = if let Some(o) = &q.order_by {
            Some((table.column_index(&o.column)
                .ok_or_else(|| ExecError::UnknownColumn(o.column.clone()))?, o.desc))
        } else { None };

        // Filter rows.
        let mut rows: Vec<Vec<CellValue>> = self.rows_of(&q.table).iter()
            .filter(|r| match (&q.where_clause, where_idx) {
                (Some(w), Some(i)) => eval_predicate(&r[i], &w.op, &w.value),
                _ => true,
            })
            .cloned()
            .map(|r| proj_indices.iter().map(|&i| r[i].clone()).collect())
            .collect();

        // ORDER BY operates on the *projected* result. We need to know the
        // index of the ORDER BY column within the projection. If the column
        // wasn't projected, sort it using original table position by selecting
        // it through the filter step... Simpler: re-select ORDER BY values
        // from the unfiltered rows and match them. Here we re-do the
        // filtered scan separately for sort keys.
        if let Some((orig_idx, desc)) = order_idx {
            let mut with_keys: Vec<(Vec<CellValue>, CellValue)> = self.rows_of(&q.table).iter()
                .filter(|r| match (&q.where_clause, where_idx) {
                    (Some(w), Some(i)) => eval_predicate(&r[i], &w.op, &w.value),
                    _ => true,
                })
                .cloned()
                .map(|r| {
                    let key = r[orig_idx].clone();
                    let projected: Vec<CellValue> = proj_indices.iter().map(|&i| r[i].clone()).collect();
                    (projected, key)
                })
                .collect();
            with_keys.sort_by(|a, b| {
                let ord = compare_values(&a.1, &b.1);
                if desc { ord.reverse() } else { ord }
            });
            rows = with_keys.into_iter().map(|(r, _)| r).collect();
        }

        if let Some(n) = q.limit {
            rows.truncate(n as usize);
        }

        let columns: Vec<String> = proj_indices.iter()
            .map(|&i| table.columns[i].name.clone())
            .collect();
        Ok(ResultSet { columns, rows })
    }
}

impl Default for Connection { fn default() -> Self { Self::new() } }

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectQuery {
    pub project_all: bool,
    pub columns: Vec<String>,
    pub table: String,
    pub where_clause: Option<WhereExpr>,
    pub order_by: Option<OrderByExpr>,
    pub limit: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WhereExpr {
    pub column: String,
    pub op: String,
    pub value: CellValue,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrderByExpr {
    pub column: String,
    pub desc: bool,
}

fn eval_predicate(cell: &CellValue, op: &str, rhs: &CellValue) -> bool {
    let ord = compare_values(cell, rhs);
    use std::cmp::Ordering::*;
    match op {
        "=" => ord == Equal,
        "!=" => ord != Equal,
        "<" => ord == Less,
        "<=" => ord != Greater,
        ">" => ord == Greater,
        ">=" => ord != Less,
        _ => false,
    }
}

fn compare_values(a: &CellValue, b: &CellValue) -> std::cmp::Ordering {
    use std::cmp::Ordering::*;
    match (a, b) {
        (CellValue::Int(x), CellValue::Int(y)) => x.cmp(y),
        (CellValue::Text(x), CellValue::Text(y)) => x.cmp(y),
        (CellValue::Null, CellValue::Null) => Equal,
        (CellValue::Null, _) => Less,     // NULL sorts first
        (_, CellValue::Null) => Greater,
        _ => Equal,                        // type mismatch: equal by convention
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn users_table() -> Connection {
        let mut c = Connection::new();
        c.create_table("users", vec![
            Column { name: "id".into(), ty: ColumnType::Int },
            Column { name: "name".into(), ty: ColumnType::Text },
            Column { name: "age".into(), ty: ColumnType::Int },
        ]).unwrap();
        c.insert("users", vec![CellValue::Int(1), CellValue::Text("Alice".into()), CellValue::Int(30)]).unwrap();
        c.insert("users", vec![CellValue::Int(2), CellValue::Text("Bob".into()), CellValue::Int(25)]).unwrap();
        c.insert("users", vec![CellValue::Int(3), CellValue::Text("Carol".into()), CellValue::Int(35)]).unwrap();
        c
    }

    #[test]
    fn create_and_insert() {
        let c = users_table();
        let t = c.table("users").unwrap();
        assert_eq!(t.rows.len(), 3);
    }

    #[test]
    fn create_table_twice_errors() {
        let mut c = Connection::new();
        c.create_table("x", vec![Column { name: "id".into(), ty: ColumnType::Int }]).unwrap();
        let err = c.create_table("x", vec![Column { name: "id".into(), ty: ColumnType::Int }]);
        assert!(matches!(err, Err(ExecError::TableExists(_))));
    }

    #[test]
    fn insert_wrong_arity_errors() {
        let mut c = Connection::new();
        c.create_table("x", vec![
            Column { name: "a".into(), ty: ColumnType::Int },
            Column { name: "b".into(), ty: ColumnType::Text },
        ]).unwrap();
        let err = c.insert("x", vec![CellValue::Int(1)]);
        assert!(matches!(err, Err(ExecError::BadValue(_))));
    }

    #[test]
    fn insert_wrong_type_errors() {
        let mut c = Connection::new();
        c.create_table("x", vec![Column { name: "a".into(), ty: ColumnType::Int }]).unwrap();
        let err = c.insert("x", vec![CellValue::Text("not an int".into())]);
        assert!(matches!(err, Err(ExecError::TypeMismatch { .. })));
    }

    #[test]
    fn select_all() {
        let c = users_table();
        let r = c.select(&SelectQuery {
            project_all: true, columns: vec![], table: "users".into(),
            where_clause: None, order_by: None, limit: None,
        }).unwrap();
        assert_eq!(r.rows.len(), 3);
        assert_eq!(r.columns, vec!["id", "name", "age"]);
    }

    #[test]
    fn select_projection() {
        let c = users_table();
        let r = c.select(&SelectQuery {
            project_all: false, columns: vec!["name".into()],
            table: "users".into(),
            where_clause: None, order_by: None, limit: None,
        }).unwrap();
        assert_eq!(r.columns, vec!["name"]);
        assert_eq!(r.rows[0].len(), 1);
    }

    #[test]
    fn select_where_equals() {
        let c = users_table();
        let r = c.select(&SelectQuery {
            project_all: true, columns: vec![], table: "users".into(),
            where_clause: Some(WhereExpr {
                column: "age".into(), op: "=".into(),
                value: CellValue::Int(30),
            }),
            order_by: None, limit: None,
        }).unwrap();
        assert_eq!(r.rows.len(), 1);
        assert_eq!(r.rows[0][1], CellValue::Text("Alice".into()));
    }

    #[test]
    fn select_where_greater_than() {
        let c = users_table();
        let r = c.select(&SelectQuery {
            project_all: false, columns: vec!["name".into()],
            table: "users".into(),
            where_clause: Some(WhereExpr {
                column: "age".into(), op: ">".into(),
                value: CellValue::Int(26),
            }),
            order_by: None, limit: None,
        }).unwrap();
        assert_eq!(r.rows.len(), 2);  // Alice(30), Carol(35)
    }

    #[test]
    fn select_order_by_asc() {
        let c = users_table();
        let r = c.select(&SelectQuery {
            project_all: false, columns: vec!["name".into()],
            table: "users".into(),
            where_clause: None,
            order_by: Some(OrderByExpr { column: "age".into(), desc: false }),
            limit: None,
        }).unwrap();
        assert_eq!(r.rows[0][0], CellValue::Text("Bob".into()));     // 25
        assert_eq!(r.rows[1][0], CellValue::Text("Alice".into()));   // 30
        assert_eq!(r.rows[2][0], CellValue::Text("Carol".into()));   // 35
    }

    #[test]
    fn select_order_by_desc_limit() {
        let c = users_table();
        let r = c.select(&SelectQuery {
            project_all: false, columns: vec!["name".into()],
            table: "users".into(),
            where_clause: None,
            order_by: Some(OrderByExpr { column: "age".into(), desc: true }),
            limit: Some(2),
        }).unwrap();
        assert_eq!(r.rows.len(), 2);
        assert_eq!(r.rows[0][0], CellValue::Text("Carol".into()));
        assert_eq!(r.rows[1][0], CellValue::Text("Alice".into()));
    }

    #[test]
    fn select_unknown_table_errors() {
        let c = users_table();
        let err = c.select(&SelectQuery {
            project_all: true, columns: vec![], table: "nope".into(),
            where_clause: None, order_by: None, limit: None,
        });
        assert!(matches!(err, Err(ExecError::UnknownTable(_))));
    }

    #[test]
    fn select_unknown_column_errors() {
        let c = users_table();
        let err = c.select(&SelectQuery {
            project_all: false, columns: vec!["birthday".into()],
            table: "users".into(),
            where_clause: None, order_by: None, limit: None,
        });
        assert!(matches!(err, Err(ExecError::UnknownColumn(_))));
    }

    #[test]
    fn transaction_commit_persists_rows() {
        let mut c = users_table();
        c.begin();
        c.insert("users", vec![CellValue::Int(4), CellValue::Text("Dave".into()), CellValue::Int(40)]).unwrap();
        assert!(c.in_transaction());
        c.commit().unwrap();
        let t = c.table("users").unwrap();
        assert_eq!(t.rows.len(), 4);
    }

    #[test]
    fn transaction_rollback_discards_rows() {
        let mut c = users_table();
        c.begin();
        c.insert("users", vec![CellValue::Int(4), CellValue::Text("Dave".into()), CellValue::Int(40)]).unwrap();
        c.rollback().unwrap();
        let t = c.table("users").unwrap();
        assert_eq!(t.rows.len(), 3);  // rollback discarded Dave
    }

    #[test]
    fn transaction_visible_to_select_before_commit() {
        let mut c = users_table();
        c.begin();
        c.insert("users", vec![CellValue::Int(4), CellValue::Text("Dave".into()), CellValue::Int(40)]).unwrap();
        let r = c.select(&SelectQuery {
            project_all: true, columns: vec![], table: "users".into(),
            where_clause: None, order_by: None, limit: None,
        }).unwrap();
        assert_eq!(r.rows.len(), 4);  // Dave is visible inside the txn
    }

    #[test]
    fn commit_without_begin_errors() {
        let mut c = Connection::new();
        assert!(matches!(c.commit(), Err(ExecError::NoTransaction)));
    }

    #[test]
    fn rollback_without_begin_errors() {
        let mut c = Connection::new();
        assert!(matches!(c.rollback(), Err(ExecError::NoTransaction)));
    }
}
