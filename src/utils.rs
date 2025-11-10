//! Utility functions for array operations

use ndarray::{Array1, ArrayView2};

/// Computes the minimum value along axis 0 (column-wise minimum)
///
/// # Arguments
/// * `arr` - 2D array view of shape (n_points, n_dimensions)
///
/// # Returns
/// 1D array of minimum values for each column
pub fn min_along_axis0(arr: &ArrayView2<f64>) -> Array1<f64> {
    let n_cols = arr.ncols();
    let mut out = arr.row(0).to_owned();

    for row in arr.rows() {
        for j in 0..n_cols {
            if row[j] < out[j] {
                out[j] = row[j];
            }
        }
    }

    out
}

/// Computes the maximum value along axis 0 (column-wise maximum)
///
/// # Arguments
/// * `arr` - 2D array view of shape (n_points, n_dimensions)
///
/// # Returns
/// 1D array of maximum values for each column
pub fn max_along_axis0(arr: &ArrayView2<f64>) -> Array1<f64> {
    let n_cols = arr.ncols();
    let mut out = arr.row(0).to_owned();

    for row in arr.rows() {
        for j in 0..n_cols {
            if row[j] > out[j] {
                out[j] = row[j];
            }
        }
    }

    out
}

/// Computes the maximum value along axis 0 for i64 arrays (column-wise maximum)
///
/// # Arguments
/// * `arr` - 2D array view of shape (n_points, n_dimensions)
///
/// # Returns
/// 1D array of maximum values for each column
pub fn max_along_axis0_i64(arr: &ArrayView2<i64>) -> Array1<i64> {
    let n_cols = arr.ncols();
    let mut out = arr.row(0).to_owned();

    for row in arr.rows() {
        for j in 0..n_cols {
            if row[j] > out[j] {
                out[j] = row[j];
            }
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use ndarray::array;
    use approx::assert_abs_diff_eq;

    #[test]
    fn test_min_along_axis0() {
        let arr = array![
            [1.0, 5.0, 3.0],
            [4.0, 2.0, 6.0],
            [0.0, 8.0, 1.0]
        ];
        let result = min_along_axis0(&arr.view());
        let expected = array![0.0, 2.0, 1.0];

        for (r, e) in result.iter().zip(expected.iter()) {
            assert_abs_diff_eq!(r, e, epsilon = 1e-10);
        }
    }

    #[test]
    fn test_max_along_axis0() {
        let arr = array![
            [1.0, 5.0, 3.0],
            [4.0, 2.0, 6.0],
            [0.0, 8.0, 1.0]
        ];
        let result = max_along_axis0(&arr.view());
        let expected = array![4.0, 8.0, 6.0];

        for (r, e) in result.iter().zip(expected.iter()) {
            assert_abs_diff_eq!(r, e, epsilon = 1e-10);
        }
    }
}
