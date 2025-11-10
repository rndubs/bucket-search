//! # Bucket Search
//!
//! A high-performance 3D spatial indexing library for efficient radius searches.
//!
//! This library implements a bucket/binning spatial data structure that organizes
//! 3D points into a grid for fast radius-based queries. Points are sorted by bin
//! for cache efficiency, and a linked list structure enables dynamic point removal.
//!
//! ## Example
//!
//! ```rust
//! use bucket_search::PointBin3D;
//! use ndarray::{array, Array2};
//!
//! let points = array![
//!     [0.0, 0.0, 0.0],
//!     [1.0, 1.0, 1.0],
//!     [5.0, 5.0, 5.0],
//! ];
//! let bin_widths = array![2.0, 2.0, 2.0];
//!
//! let mut point_bin = PointBin3D::new(points, bin_widths);
//!
//! // Search for points within radius 1.5 of origin
//! let query = array![0.0, 0.0, 0.0];
//! point_bin.radius_search(&query.view(), 1.5);
//!
//! let found = point_bin.found_indices();
//! println!("Found {} points", found.len());
//! ```

mod utils;
mod pointbin;

pub use pointbin::PointBin3D;
pub use utils::{max_along_axis0, min_along_axis0};

// Python bindings
#[cfg(feature = "python")]
mod python_bindings;

#[cfg(feature = "python")]
pub use python_bindings::*;
