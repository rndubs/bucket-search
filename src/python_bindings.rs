//! Python bindings for the bucket-search library

use ndarray::{Array1, Array2};
use numpy::{IntoPyArray, PyArray1, PyArray2, PyReadonlyArray1, PyReadonlyArray2};
use pyo3::prelude::*;
use pyo3::types::PyModule;

use crate::PointBin3D as RustPointBin3D;

/// Python wrapper for PointBin3D
///
/// A 3D spatial indexing structure for efficient radius searches.
/// Points are organized into bins for fast spatial queries.
///
/// Parameters
/// ----------
/// points : numpy.ndarray
///     2D array of shape (n_points, 3) containing point coordinates
/// bin_widths : numpy.ndarray
///     1D array of shape (3,) containing bin widths for x, y, z dimensions
///
/// Examples
/// --------
/// >>> import numpy as np
/// >>> import bucket_search
/// >>> points = np.array([[0.0, 0.0, 0.0], [1.0, 1.0, 1.0], [5.0, 5.0, 5.0]])
/// >>> bin_widths = np.array([2.0, 2.0, 2.0])
/// >>> point_bin = bucket_search.PointBin3D(points, bin_widths)
/// >>> point_bin.radius_search(np.array([0.0, 0.0, 0.0]), 1.5)
/// >>> found = point_bin.found_indices()
/// >>> print(f"Found {len(found)} points")
#[pyclass(name = "PointBin3D")]
pub struct PyPointBin3D {
    inner: RustPointBin3D,
}

#[pymethods]
impl PyPointBin3D {
    /// Create a new PointBin3D structure
    ///
    /// Parameters
    /// ----------
    /// points : numpy.ndarray
    ///     2D array of shape (n_points, 3) with point coordinates
    /// bin_widths : numpy.ndarray
    ///     1D array of shape (3,) with bin widths for x, y, z
    ///
    /// Returns
    /// -------
    /// PointBin3D
    ///     New spatial indexing structure
    #[new]
    pub fn new(
        points: PyReadonlyArray2<f64>,
        bin_widths: PyReadonlyArray1<f64>,
    ) -> PyResult<Self> {
        let points_array = points.as_array();
        let bin_widths_array = bin_widths.as_array();

        // Validate dimensions
        if points_array.ncols() != 3 {
            return Err(pyo3::exceptions::PyValueError::new_err(
                "Points must have exactly 3 columns (x, y, z)",
            ));
        }

        if bin_widths_array.len() != 3 {
            return Err(pyo3::exceptions::PyValueError::new_err(
                "Bin widths must have exactly 3 elements",
            ));
        }

        // Convert to owned arrays
        let points_owned: Array2<f64> = points_array.to_owned();
        let bin_widths_owned: Array1<f64> = bin_widths_array.to_owned();

        let inner = RustPointBin3D::new(points_owned, bin_widths_owned);

        Ok(PyPointBin3D { inner })
    }

    /// Perform a radius search around a query point
    ///
    /// Finds all points within the specified radius and removes them from the structure.
    /// Results accumulate across multiple calls and can be retrieved with `found_indices()`.
    ///
    /// Parameters
    /// ----------
    /// query_point : numpy.ndarray
    ///     1D array of shape (3,) with query point coordinates
    /// radius : float
    ///     Search radius
    pub fn radius_search(&mut self, query_point: PyReadonlyArray1<f64>, radius: f64) {
        let query_array = query_point.as_array();
        self.inner.radius_search(&query_array, radius);
    }

    /// Get the original indices of all found points
    ///
    /// Returns the indices into the original points array that were found
    /// across all radius searches since the last reset.
    ///
    /// Returns
    /// -------
    /// numpy.ndarray
    ///     1D array of original point indices (int64)
    pub fn found_indices<'py>(&self, py: Python<'py>) -> &'py PyArray1<i64> {
        let indices = self.inner.found_indices();
        indices.into_pyarray(py)
    }

    /// Reset the structure for a fresh search
    ///
    /// Restores all points and clears the found indices buffer.
    pub fn reset(&mut self) {
        self.inner.reset();
    }

    /// Get the number of points found so far
    ///
    /// Returns
    /// -------
    /// int
    ///     Number of points found since last reset
    pub fn found_count(&self) -> usize {
        self.inner.found_count()
    }

    /// Get the original points array
    ///
    /// Returns
    /// -------
    /// numpy.ndarray
    ///     2D array of shape (n_points, 3) with original points
    pub fn original_points<'py>(&self, py: Python<'py>) -> &'py PyArray2<f64> {
        self.inner.original_points().clone().into_pyarray(py)
    }

    /// Get the bin shape
    ///
    /// Returns
    /// -------
    /// numpy.ndarray
    ///     1D array of shape (3,) with number of bins in each dimension
    pub fn bin_shape<'py>(&self, py: Python<'py>) -> &'py PyArray1<i64> {
        self.inner.bin_shape().clone().into_pyarray(py)
    }

    /// Get the origin point
    ///
    /// Returns
    /// -------
    /// numpy.ndarray
    ///     1D array of shape (3,) with origin coordinates
    pub fn origin<'py>(&self, py: Python<'py>) -> &'py PyArray1<f64> {
        self.inner.origin().clone().into_pyarray(py)
    }

    fn __repr__(&self) -> String {
        format!(
            "PointBin3D(n_points={}, found_count={})",
            self.inner.original_points().nrows(),
            self.inner.found_count()
        )
    }

    fn __str__(&self) -> String {
        self.__repr__()
    }
}

/// Python module for bucket-search
#[pymodule]
fn _bucket_search(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyPointBin3D>()?;
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    Ok(())
}
