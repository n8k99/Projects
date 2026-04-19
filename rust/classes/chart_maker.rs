//! Chart Making — a visualization pipeline as composed functions.
//!
//! data → scale → encode → layout → render, with each stage independently
//! testable. Scales are callable function objects; chart specs are declarative
//! structures; rendering is a pluggable backend (ASCII here).

#[derive(Debug, Clone)]
pub struct DataPoint {
    pub label: String,
    pub value: f64,
    pub series: String,
}

impl DataPoint {
    pub fn new(label: &str, value: f64) -> Self {
        Self { label: label.to_string(), value, series: String::new() }
    }
}

/// A function from a numeric domain to a numeric range.
#[derive(Debug, Clone, Copy)]
pub struct LinearScale {
    pub domain_min: f64,
    pub domain_max: f64,
    pub range_min: f64,
    pub range_max: f64,
}

impl LinearScale {
    pub fn new(domain_min: f64, domain_max: f64, range_min: f64, range_max: f64) -> Self {
        Self { domain_min, domain_max, range_min, range_max }
    }

    /// Evaluate the scale at x.
    pub fn at(&self, x: f64) -> f64 {
        if self.domain_max == self.domain_min {
            return (self.range_min + self.range_max) / 2.0;
        }
        let t = (x - self.domain_min) / (self.domain_max - self.domain_min);
        self.range_min + t * (self.range_max - self.range_min)
    }

    /// Fit a scale to a set of values, optionally including zero in the domain.
    pub fn fit(values: &[f64], range_min: f64, range_max: f64, include_zero: bool) -> Self {
        if values.is_empty() {
            return Self::new(0.0, 1.0, range_min, range_max);
        }
        let mut lo = values.iter().cloned().fold(f64::INFINITY, f64::min);
        let mut hi = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        if include_zero {
            lo = lo.min(0.0);
            hi = hi.max(0.0);
        }
        if lo == hi {
            lo -= 1.0;
            hi += 1.0;
        }
        Self::new(lo, hi, range_min, range_max)
    }
}

/// Maps discrete categories to consecutive positions in [range_min, range_max].
#[derive(Debug, Clone)]
pub struct OrdinalScale {
    pub categories: Vec<String>,
    pub range_min: i32,
    pub range_max: i32,
}

impl OrdinalScale {
    pub fn at(&self, category: &str) -> Option<i32> {
        let idx = self.categories.iter().position(|c| c == category)?;
        let n = self.categories.len();
        if n <= 1 {
            return Some((self.range_min + self.range_max) / 2);
        }
        let step = (self.range_max - self.range_min) as f64 / (n - 1) as f64;
        Some((self.range_min as f64 + idx as f64 * step).round() as i32)
    }
}

// --- chart specifications ---

pub struct BarChart {
    pub title: String,
    pub points: Vec<DataPoint>,
    pub width: usize,
}

impl BarChart {
    pub fn render(&self) -> String {
        if self.points.is_empty() {
            return format!("{}\n(no data)", self.title);
        }
        let values: Vec<f64> = self.points.iter().map(|p| p.value).collect();
        let scale = LinearScale::fit(&values, 0.0, self.width as f64, true);
        let label_w = self.points.iter().map(|p| p.label.chars().count()).max().unwrap_or(0);
        let sep_len = label_w + 4 + self.width + 10;
        let mut lines = vec![self.title.clone(), "=".repeat(sep_len)];
        for p in &self.points {
            let bar_len = scale.at(p.value).round().max(0.0) as usize;
            let bar = "█".repeat(bar_len);
            let bar_padded = format!("{:<width$}", bar, width = self.width);
            lines.push(format!("  {:<lw$}  {}  {:.2}", p.label, bar_padded, p.value, lw = label_w));
        }
        lines.join("\n")
    }
}

pub struct LineChart {
    pub title: String,
    pub points: Vec<DataPoint>,
    pub width: usize,
    pub height: usize,
}

impl LineChart {
    pub fn render(&self) -> String {
        if self.points.is_empty() {
            return format!("{}\n(no data)", self.title);
        }
        let n = self.points.len();
        let values: Vec<f64> = self.points.iter().map(|p| p.value).collect();
        let x_scale = LinearScale::new(0.0, (n.saturating_sub(1).max(1)) as f64,
                                       0.0, (self.width - 1) as f64);
        let y_scale = LinearScale::fit(&values, 0.0, (self.height - 1) as f64, false);
        let mut grid: Vec<Vec<char>> = vec![vec![' '; self.width]; self.height];
        let mut plot = |col: i32, row: i32, ch: char, grid: &mut Vec<Vec<char>>| {
            if col >= 0 && (col as usize) < self.width && row >= 0 && (row as usize) < self.height {
                grid[row as usize][col as usize] = ch;
            }
        };
        let mut prev: Option<(i32, i32)> = None;
        for (i, p) in self.points.iter().enumerate() {
            let col = x_scale.at(i as f64).round() as i32;
            let row = self.height as i32 - 1 - y_scale.at(p.value).round() as i32;
            plot(col, row, '●', &mut grid);
            if let Some((pc, pr)) = prev {
                let dx = col - pc;
                let dy = row - pr;
                let steps = dx.abs().max(dy.abs());
                if steps > 1 {
                    for s in 1..steps {
                        let t = s as f64 / steps as f64;
                        let ic = (pc as f64 + dx as f64 * t).round() as i32;
                        let ir = (pr as f64 + dy as f64 * t).round() as i32;
                        if ir >= 0 && (ir as usize) < self.height
                           && ic >= 0 && (ic as usize) < self.width
                           && grid[ir as usize][ic as usize] == ' ' {
                            plot(ic, ir, '·', &mut grid);
                        }
                    }
                }
            }
            prev = Some((col, row));
        }
        let mut out = vec![self.title.clone()];
        let hi = y_scale.domain_max;
        let lo = y_scale.domain_min;
        for r in 0..self.height {
            let left = if r == 0 {
                format!("{:>7.2} ┤", hi)
            } else if r == self.height - 1 {
                format!("{:>7.2} ┤", lo)
            } else {
                "        │".to_string()
            };
            let row: String = grid[r].iter().collect();
            out.push(format!("{}{}", left, row));
        }
        out.push(format!("        └{}", "─".repeat(self.width)));
        out.join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn linear_scale_is_pure_function() {
        let s = LinearScale::new(0.0, 100.0, 0.0, 10.0);
        assert!((s.at(0.0) - 0.0).abs() < 1e-9);
        assert!((s.at(100.0) - 10.0).abs() < 1e-9);
        assert!((s.at(50.0) - 5.0).abs() < 1e-9);
    }

    #[test]
    fn linear_scale_fit_expands_to_include_zero_when_requested() {
        let s = LinearScale::fit(&[10.0, 20.0, 30.0], 0.0, 100.0, true);
        assert_eq!(s.domain_min, 0.0); // zero included
        assert_eq!(s.domain_max, 30.0);
    }

    #[test]
    fn linear_scale_fit_collapses_single_value_domain() {
        // Single constant value would give a zero-width domain; fit should widen it.
        let s = LinearScale::fit(&[5.0, 5.0, 5.0], 0.0, 10.0, false);
        assert!(s.domain_min < s.domain_max);
    }

    #[test]
    fn ordinal_scale_places_categories_evenly() {
        let s = OrdinalScale {
            categories: vec!["A".into(), "B".into(), "C".into()],
            range_min: 0, range_max: 10,
        };
        assert_eq!(s.at("A"), Some(0));
        assert_eq!(s.at("C"), Some(10));
        assert_eq!(s.at("B"), Some(5));
        assert_eq!(s.at("D"), None);
    }

    #[test]
    fn bar_chart_renders_non_empty_output() {
        let chart = BarChart {
            title: "Test".into(), width: 20,
            points: vec![DataPoint::new("x", 10.0), DataPoint::new("y", 20.0)],
        };
        let out = chart.render();
        assert!(out.contains("Test"));
        assert!(out.contains("█"));
        assert!(out.contains("x"));
        assert!(out.contains("y"));
    }

    #[test]
    fn line_chart_renders_non_empty_output() {
        let pts: Vec<DataPoint> = (0..10)
            .map(|i| DataPoint::new(&format!("t{i}"), (i as f64).sin() * 10.0))
            .collect();
        let chart = LineChart { title: "sine".into(), points: pts, width: 40, height: 10 };
        let out = chart.render();
        assert!(out.contains("sine"));
        assert!(out.contains('●')); // at least one point plotted
    }
}
