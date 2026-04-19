//! Matrix Class — a 2D value class whose shape constrains its operations.
//!
//! Shape is part of the matrix's identity. Addition needs matching shapes;
//! multiplication needs the inner dimensions to match; determinant needs square.
//! These preconditions are checked at runtime — a more expressive type system
//! could encode them statically (dependent types), but Rust's does not. Errors
//! at runtime are the tradeoff.

#[derive(Debug)]
pub enum MatrixError {
    Empty,
    NotRectangular,
    ShapeMismatch(String),
    NotSquare((usize, usize)),
    NonPositiveDimension,
}

#[derive(Debug, Clone)]
pub struct Matrix {
    rows: Vec<Vec<f64>>,
}

impl Matrix {
    pub fn new(rows: Vec<Vec<f64>>) -> Result<Self, MatrixError> {
        if rows.is_empty() { return Err(MatrixError::Empty); }
        let cols = rows[0].len();
        if cols == 0 { return Err(MatrixError::Empty); }
        if rows.iter().any(|r| r.len() != cols) {
            return Err(MatrixError::NotRectangular);
        }
        Ok(Self { rows })
    }

    pub fn zeros(rows: usize, cols: usize) -> Result<Self, MatrixError> {
        if rows == 0 || cols == 0 { return Err(MatrixError::NonPositiveDimension); }
        Ok(Self { rows: vec![vec![0.0; cols]; rows] })
    }

    pub fn identity(n: usize) -> Result<Self, MatrixError> {
        if n == 0 { return Err(MatrixError::NonPositiveDimension); }
        let mut m = Self::zeros(n, n)?;
        for i in 0..n {
            m.rows[i][i] = 1.0;
        }
        Ok(m)
    }

    pub fn nrows(&self) -> usize { self.rows.len() }
    pub fn ncols(&self) -> usize { self.rows[0].len() }
    pub fn shape(&self) -> (usize, usize) { (self.nrows(), self.ncols()) }
    pub fn is_square(&self) -> bool { self.nrows() == self.ncols() }

    pub fn get(&self, i: usize, j: usize) -> f64 { self.rows[i][j] }
    pub fn set(&mut self, i: usize, j: usize, v: f64) { self.rows[i][j] = v; }

    pub fn add(&self, other: &Self) -> Result<Self, MatrixError> {
        if self.shape() != other.shape() {
            return Err(MatrixError::ShapeMismatch(format!(
                "cannot add: {:?} + {:?}", self.shape(), other.shape())));
        }
        let mut out = Self::zeros(self.nrows(), self.ncols())?;
        for i in 0..self.nrows() {
            for j in 0..self.ncols() {
                out.rows[i][j] = self.rows[i][j] + other.rows[i][j];
            }
        }
        Ok(out)
    }

    pub fn sub(&self, other: &Self) -> Result<Self, MatrixError> {
        if self.shape() != other.shape() {
            return Err(MatrixError::ShapeMismatch(format!(
                "cannot subtract: {:?} - {:?}", self.shape(), other.shape())));
        }
        let mut out = Self::zeros(self.nrows(), self.ncols())?;
        for i in 0..self.nrows() {
            for j in 0..self.ncols() {
                out.rows[i][j] = self.rows[i][j] - other.rows[i][j];
            }
        }
        Ok(out)
    }

    pub fn scalar_mul(&self, k: f64) -> Self {
        let rows = self
            .rows
            .iter()
            .map(|r| r.iter().map(|&v| k * v).collect())
            .collect();
        Self { rows }
    }

    pub fn matmul(&self, other: &Self) -> Result<Self, MatrixError> {
        if self.ncols() != other.nrows() {
            return Err(MatrixError::ShapeMismatch(format!(
                "cannot multiply: inner dimensions differ {:?} x {:?}",
                self.shape(), other.shape())));
        }
        let m = self.nrows();
        let n = self.ncols();
        let p = other.ncols();
        let mut out = Self::zeros(m, p)?;
        for i in 0..m {
            for j in 0..p {
                let mut s = 0.0;
                for k in 0..n {
                    s += self.rows[i][k] * other.rows[k][j];
                }
                out.rows[i][j] = s;
            }
        }
        Ok(out)
    }

    pub fn transpose(&self) -> Self {
        let (m, n) = self.shape();
        let mut out = Self::zeros(n, m).unwrap();
        for i in 0..m {
            for j in 0..n {
                out.rows[j][i] = self.rows[i][j];
            }
        }
        out
    }

    pub fn determinant(&self) -> Result<f64, MatrixError> {
        if !self.is_square() {
            return Err(MatrixError::NotSquare(self.shape()));
        }
        Ok(det(&self.rows))
    }

    pub fn approx_eq(&self, other: &Self, eps: f64) -> bool {
        if self.shape() != other.shape() { return false; }
        for i in 0..self.nrows() {
            for j in 0..self.ncols() {
                if (self.rows[i][j] - other.rows[i][j]).abs() > eps {
                    return false;
                }
            }
        }
        true
    }
}

fn det(rows: &[Vec<f64>]) -> f64 {
    let n = rows.len();
    if n == 1 { return rows[0][0]; }
    if n == 2 {
        return rows[0][0] * rows[1][1] - rows[0][1] * rows[1][0];
    }
    let mut total = 0.0;
    for c in 0..n {
        let minor: Vec<Vec<f64>> = (1..n)
            .map(|i| (0..n).filter(|&k| k != c).map(|k| rows[i][k]).collect())
            .collect();
        let sign = if c % 2 == 0 { 1.0 } else { -1.0 };
        total += sign * rows[0][c] * det(&minor);
    }
    total
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn addition_requires_matching_shapes() {
        let a = Matrix::new(vec![vec![1.0, 2.0], vec![3.0, 4.0]]).unwrap();
        let b = Matrix::new(vec![vec![1.0, 2.0, 3.0]]).unwrap();
        assert!(matches!(a.add(&b), Err(MatrixError::ShapeMismatch(_))));
    }

    #[test]
    fn matmul_requires_inner_dimensions_to_match() {
        let a = Matrix::new(vec![vec![1.0, 2.0, 3.0], vec![4.0, 5.0, 6.0]]).unwrap(); // 2x3
        let b = Matrix::new(vec![vec![7.0, 8.0], vec![9.0, 10.0]]).unwrap();          // 2x2
        assert!(matches!(a.matmul(&b), Err(MatrixError::ShapeMismatch(_))));
    }

    #[test]
    fn matmul_shape_is_outer_dimensions() {
        let a = Matrix::new(vec![vec![1.0, 2.0, 3.0], vec![4.0, 5.0, 6.0]]).unwrap();    // 2x3
        let b = Matrix::new(vec![vec![7.0, 8.0], vec![9.0, 10.0], vec![11.0, 12.0]]).unwrap(); // 3x2
        let c = a.matmul(&b).unwrap();
        assert_eq!(c.shape(), (2, 2));
    }

    #[test]
    fn matmul_is_non_commutative() {
        let a = Matrix::new(vec![vec![1.0, 2.0, 3.0], vec![4.0, 5.0, 6.0]]).unwrap();    // 2x3
        let b = Matrix::new(vec![vec![7.0, 8.0], vec![9.0, 10.0], vec![11.0, 12.0]]).unwrap(); // 3x2
        let ab = a.matmul(&b).unwrap();
        let ba = b.matmul(&a).unwrap();
        assert_ne!(ab.shape(), ba.shape()); // 2x2 vs 3x3 — not even the same shape
    }

    #[test]
    fn identity_is_two_sided_neutral() {
        let a = Matrix::new(vec![vec![1.0, 2.0, 3.0], vec![4.0, 5.0, 6.0]]).unwrap(); // 2x3
        let i2 = Matrix::identity(2).unwrap();
        let i3 = Matrix::identity(3).unwrap();
        assert!(i2.matmul(&a).unwrap().approx_eq(&a, 1e-9));
        assert!(a.matmul(&i3).unwrap().approx_eq(&a, 1e-9));
    }

    #[test]
    fn determinant_rejects_non_square() {
        let a = Matrix::new(vec![vec![1.0, 2.0, 3.0], vec![4.0, 5.0, 6.0]]).unwrap();
        assert!(matches!(a.determinant(), Err(MatrixError::NotSquare(_))));
    }

    #[test]
    fn determinant_2x2_and_3x3_match_known_values() {
        let a = Matrix::new(vec![vec![3.0, 8.0], vec![4.0, 6.0]]).unwrap();
        assert!((a.determinant().unwrap() - (-14.0)).abs() < 1e-9);
        let b = Matrix::new(vec![
            vec![4.0, 3.0, 0.0],
            vec![2.0, 5.0, 6.0],
            vec![1.0, 0.0, 2.0],
        ]).unwrap();
        // |b| = 4*(5*2 - 6*0) - 3*(2*2 - 6*1) + 0 = 40 - 3*(-2) = 46
        assert!((b.determinant().unwrap() - 46.0).abs() < 1e-9);
    }

    #[test]
    fn transpose_is_involutive() {
        let a = Matrix::new(vec![vec![1.0, 2.0, 3.0], vec![4.0, 5.0, 6.0]]).unwrap();
        assert!(a.transpose().transpose().approx_eq(&a, 1e-12));
    }
}
