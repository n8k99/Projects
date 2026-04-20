//! Report Generator — grouped reports with subtotal and grand-total rows.
//!
//! Fourth Databases project. Introduces **report as a structured
//! document**: list of typed rows (header, data, subtotal, total)
//! derived from a record stream. This is what `pandas.pivot_table`,
//! `SELECT ... GROUP BY WITH ROLLUP`, and every accounting report
//! generator produces.
//!
//! Distinctive moves:
//!   * **Columns as metadata** — each column declares `name`,
//!      `source_field`, and optionally a sum aggregator.
//!   * **Group-by with subtotals** — G089 did aggregation to a single
//!      value per group; G104 emits both data rows AND subtotal rows
//!      under each group.
//!   * **Row-kind tagging** — every output row is `Header`, `Data`,
//!      `Subtotal`, or `Total`. Callers dispatch on kind.
//!   * **Formatter closures** — column value → string. Different columns
//!      format differently (currency, dates, plain).

use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Int(i64),
    Text(String),
    Null,
}

impl Value {
    pub fn as_int(&self) -> Option<i64> {
        match self { Value::Int(n) => Some(*n), _ => None }
    }
    pub fn as_text(&self) -> Option<String> {
        match self {
            Value::Text(s) => Some(s.clone()),
            Value::Int(n) => Some(n.to_string()),
            Value::Null => None,
        }
    }
}

pub type Record = BTreeMap<String, Value>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColumnKind { Plain, SumAggregate }

#[derive(Debug, Clone)]
pub struct ColumnDef {
    pub name: String,
    pub field: String,
    pub kind: ColumnKind,
    pub format_cents: bool,   // if true, render int as dollars
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RowKind { Header, Data, Subtotal, Total }

#[derive(Debug, Clone, PartialEq)]
pub struct ReportRow {
    pub kind: RowKind,
    pub cells: Vec<String>,
    pub group_label: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ReportSpec {
    pub title: String,
    pub columns: Vec<ColumnDef>,
    pub group_by: Option<String>,   // field name
}

#[derive(Debug, Clone)]
pub struct Report {
    pub title: String,
    pub rows: Vec<ReportRow>,
}

/// Build a report from records according to the spec.
pub fn generate(records: &[Record], spec: &ReportSpec) -> Report {
    let mut rows = Vec::new();

    // Header row — just column names.
    rows.push(ReportRow {
        kind: RowKind::Header,
        cells: spec.columns.iter().map(|c| c.name.clone()).collect(),
        group_label: None,
    });

    // Group records if requested, otherwise single "ALL" group.
    let groups: Vec<(String, Vec<&Record>)> = if let Some(gfield) = &spec.group_by {
        let mut m: BTreeMap<String, Vec<&Record>> = BTreeMap::new();
        for r in records {
            let key = r.get(gfield).and_then(|v| v.as_text()).unwrap_or_else(|| "(none)".into());
            m.entry(key).or_default().push(r);
        }
        m.into_iter().collect()
    } else {
        vec![("__all__".into(), records.iter().collect())]
    };

    // Grand total accumulator.
    let mut grand_totals: BTreeMap<String, i64> = BTreeMap::new();

    for (group_key, group_records) in &groups {
        // Data rows for this group.
        for record in group_records {
            let cells: Vec<String> = spec.columns.iter()
                .map(|col| format_cell(record.get(&col.field), col))
                .collect();
            rows.push(ReportRow {
                kind: RowKind::Data,
                cells,
                group_label: if spec.group_by.is_some() { Some(group_key.clone()) } else { None },
            });
        }

        // Subtotal row (only if grouping).
        if spec.group_by.is_some() {
            let mut subtotals: BTreeMap<String, i64> = BTreeMap::new();
            for record in group_records {
                for col in &spec.columns {
                    if col.kind == ColumnKind::SumAggregate {
                        if let Some(v) = record.get(&col.field).and_then(|v| v.as_int()) {
                            *subtotals.entry(col.field.clone()).or_insert(0) += v;
                            *grand_totals.entry(col.field.clone()).or_insert(0) += v;
                        }
                    }
                }
            }
            let cells: Vec<String> = spec.columns.iter().enumerate().map(|(i, col)| {
                if i == 0 {
                    format!("{} subtotal", group_key)
                } else if col.kind == ColumnKind::SumAggregate {
                    let val = subtotals.get(&col.field).copied().unwrap_or(0);
                    format_int(val, col.format_cents)
                } else { String::new() }
            }).collect();
            rows.push(ReportRow {
                kind: RowKind::Subtotal,
                cells,
                group_label: Some(group_key.clone()),
            });
        }
    }

    // Grand total row (only if grouping or summary columns exist).
    if spec.columns.iter().any(|c| c.kind == ColumnKind::SumAggregate) {
        // If not grouped, compute grand totals from all records.
        if spec.group_by.is_none() {
            for record in records {
                for col in &spec.columns {
                    if col.kind == ColumnKind::SumAggregate {
                        if let Some(v) = record.get(&col.field).and_then(|v| v.as_int()) {
                            *grand_totals.entry(col.field.clone()).or_insert(0) += v;
                        }
                    }
                }
            }
        }
        let cells: Vec<String> = spec.columns.iter().enumerate().map(|(i, col)| {
            if i == 0 { "TOTAL".into() }
            else if col.kind == ColumnKind::SumAggregate {
                let val = grand_totals.get(&col.field).copied().unwrap_or(0);
                format_int(val, col.format_cents)
            } else { String::new() }
        }).collect();
        rows.push(ReportRow { kind: RowKind::Total, cells, group_label: None });
    }

    Report { title: spec.title.clone(), rows }
}

fn format_cell(value: Option<&Value>, col: &ColumnDef) -> String {
    match value {
        None | Some(Value::Null) => String::new(),
        Some(Value::Int(n)) => format_int(*n, col.format_cents),
        Some(Value::Text(s)) => s.clone(),
    }
}

fn format_int(n: i64, as_cents: bool) -> String {
    if as_cents {
        let abs = n.unsigned_abs();
        let dollars = abs / 100;
        let cents = abs % 100;
        let sign = if n < 0 { "-" } else { "" };
        format!("${sign}{dollars}.{cents:02}")
    } else {
        n.to_string()
    }
}

/// Render a report as a plain-text table (tab-delimited).
pub fn render(report: &Report) -> String {
    let mut out = String::new();
    if !report.title.is_empty() {
        out.push_str(&report.title);
        out.push('\n');
    }
    for row in &report.rows {
        let prefix = match row.kind {
            RowKind::Header => "#",
            RowKind::Data => " ",
            RowKind::Subtotal => "~",
            RowKind::Total => "=",
        };
        out.push_str(prefix);
        out.push('\t');
        out.push_str(&row.cells.join("\t"));
        out.push('\n');
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sales_records() -> Vec<Record> {
        let mk = |region: &str, product: &str, qty: i64, revenue_cents: i64| -> Record {
            let mut r = Record::new();
            r.insert("region".into(), Value::Text(region.into()));
            r.insert("product".into(), Value::Text(product.into()));
            r.insert("qty".into(), Value::Int(qty));
            r.insert("revenue".into(), Value::Int(revenue_cents));
            r
        };
        vec![
            mk("East", "Widget", 3, 1500),
            mk("East", "Gadget", 2, 2000),
            mk("West", "Widget", 5, 2500),
            mk("West", "Gizmo", 1, 750),
        ]
    }

    fn sales_spec() -> ReportSpec {
        ReportSpec {
            title: "Sales Report".into(),
            columns: vec![
                ColumnDef { name: "Region".into(), field: "region".into(),
                            kind: ColumnKind::Plain, format_cents: false },
                ColumnDef { name: "Product".into(), field: "product".into(),
                            kind: ColumnKind::Plain, format_cents: false },
                ColumnDef { name: "Qty".into(), field: "qty".into(),
                            kind: ColumnKind::SumAggregate, format_cents: false },
                ColumnDef { name: "Revenue".into(), field: "revenue".into(),
                            kind: ColumnKind::SumAggregate, format_cents: true },
            ],
            group_by: Some("region".into()),
        }
    }

    #[test]
    fn report_has_header_row() {
        let r = generate(&sales_records(), &sales_spec());
        assert_eq!(r.rows[0].kind, RowKind::Header);
        assert_eq!(r.rows[0].cells, vec!["Region", "Product", "Qty", "Revenue"]);
    }

    #[test]
    fn report_has_data_rows_per_record() {
        let r = generate(&sales_records(), &sales_spec());
        let data_count = r.rows.iter().filter(|row| row.kind == RowKind::Data).count();
        assert_eq!(data_count, 4);
    }

    #[test]
    fn report_has_subtotal_per_group() {
        let r = generate(&sales_records(), &sales_spec());
        let subtotals: Vec<&ReportRow> = r.rows.iter()
            .filter(|row| row.kind == RowKind::Subtotal).collect();
        assert_eq!(subtotals.len(), 2);
    }

    #[test]
    fn report_has_grand_total() {
        let r = generate(&sales_records(), &sales_spec());
        let total: Vec<&ReportRow> = r.rows.iter()
            .filter(|row| row.kind == RowKind::Total).collect();
        assert_eq!(total.len(), 1);
    }

    #[test]
    fn subtotal_sums_numeric_columns() {
        let r = generate(&sales_records(), &sales_spec());
        let east_subtotal = r.rows.iter().find(|row|
            row.kind == RowKind::Subtotal &&
            row.group_label.as_deref() == Some("East")
        ).unwrap();
        // Qty column (index 2): 3 + 2 = 5
        assert_eq!(east_subtotal.cells[2], "5");
        // Revenue column (index 3): 1500 + 2000 = 3500 cents = $35.00
        assert_eq!(east_subtotal.cells[3], "$35.00");
    }

    #[test]
    fn grand_total_sums_all_records() {
        let r = generate(&sales_records(), &sales_spec());
        let total = r.rows.iter().find(|row| row.kind == RowKind::Total).unwrap();
        // Qty: 3+2+5+1 = 11
        assert_eq!(total.cells[2], "11");
        // Revenue: 1500+2000+2500+750 = 6750 cents = $67.50
        assert_eq!(total.cells[3], "$67.50");
    }

    #[test]
    fn grand_total_label_in_first_cell() {
        let r = generate(&sales_records(), &sales_spec());
        let total = r.rows.iter().find(|row| row.kind == RowKind::Total).unwrap();
        assert_eq!(total.cells[0], "TOTAL");
    }

    #[test]
    fn subtotal_label_includes_group_key() {
        let r = generate(&sales_records(), &sales_spec());
        let east = r.rows.iter().find(|row|
            row.kind == RowKind::Subtotal &&
            row.group_label.as_deref() == Some("East")
        ).unwrap();
        assert!(east.cells[0].contains("East"));
    }

    #[test]
    fn report_without_grouping_has_no_subtotals() {
        let mut spec = sales_spec();
        spec.group_by = None;
        let r = generate(&sales_records(), &spec);
        let subtotals = r.rows.iter().filter(|row|
            row.kind == RowKind::Subtotal).count();
        assert_eq!(subtotals, 0);
        // Grand total still present.
        let total = r.rows.iter().filter(|row|
            row.kind == RowKind::Total).count();
        assert_eq!(total, 1);
    }

    #[test]
    fn report_without_aggregates_has_no_total() {
        let spec = ReportSpec {
            title: "".into(),
            columns: vec![
                ColumnDef { name: "Region".into(), field: "region".into(),
                            kind: ColumnKind::Plain, format_cents: false },
            ],
            group_by: None,
        };
        let r = generate(&sales_records(), &spec);
        let total = r.rows.iter().filter(|row|
            row.kind == RowKind::Total).count();
        assert_eq!(total, 0);
    }

    #[test]
    fn empty_records_produces_header_only() {
        let r = generate(&[], &sales_spec());
        assert_eq!(r.rows.iter().filter(|row| row.kind == RowKind::Data).count(), 0);
    }

    #[test]
    fn render_includes_title_and_rows() {
        let r = generate(&sales_records(), &sales_spec());
        let text = render(&r);
        assert!(text.starts_with("Sales Report\n"));
        assert!(text.contains("Region"));
        assert!(text.contains("TOTAL"));
    }

    #[test]
    fn currency_format_handles_zero_and_small_cents() {
        let record: Record = [
            ("region".to_string(), Value::Text("Only".into())),
            ("product".to_string(), Value::Text("Thing".into())),
            ("qty".to_string(), Value::Int(0)),
            ("revenue".to_string(), Value::Int(5)),  // 5 cents
        ].iter().cloned().collect();
        let r = generate(&[record], &sales_spec());
        let data = r.rows.iter().find(|row| row.kind == RowKind::Data).unwrap();
        assert_eq!(data.cells[3], "$0.05");
    }
}
