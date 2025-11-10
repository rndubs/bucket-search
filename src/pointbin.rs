//! Core PointBin3D data structure for efficient spatial indexing

use ndarray::{Array1, Array2, Array3, ArrayView1};
use crate::utils::{max_along_axis0_i64, min_along_axis0};

/// A 3D spatial indexing structure using binning/bucketing for efficient radius searches
///
/// This structure bins points into a 3D grid and maintains a linked list structure
/// for cache-efficient spatial queries. Points are sorted by bin for optimal memory access.
pub struct PointBin3D {
    /// Original input points (n_points, 3)
    original_points: Array2<f64>,
    /// Cache-friendly sorted copy of points (n_points, 3)
    points: Array2<f64>,
    /// Width of each bin in x, y, z dimensions (3,)
    bin_widths: Array1<f64>,
    /// Origin point (minimum corner) of the binning grid (3,)
    origin: Array1<f64>,
    /// Maps sorted index back to original index (n_points,)
    original_indices: Array1<i64>,
    /// Shape of the bin grid (3,)
    bin_shape: Array1<i64>,
    /// Head of linked list for each bin (bin_shape[0], bin_shape[1], bin_shape[2])
    first_member: Array3<i64>,
    /// Next pointer in linked list (n_points,)
    next_member: Array1<i64>,
    /// Backup of first_member for reset (bin_shape[0], bin_shape[1], bin_shape[2])
    original_first_member: Array3<i64>,
    /// Backup of next_member for reset (n_points,)
    original_next_member: Array1<i64>,
    /// Buffer for storing found indices during search (n_points,)
    found_indices_buffer: Array1<i64>,
    /// Count of found points in current search
    found_count: usize,
}

impl PointBin3D {
    /// Create a new PointBin3D structure
    ///
    /// # Arguments
    /// * `original_points` - 2D array of shape (n_points, 3) with point coordinates
    /// * `bin_widths` - 1D array of shape (3,) with bin widths for x, y, z
    ///
    /// # Returns
    /// A new PointBin3D instance with points organized into bins
    ///
    /// # Panics
    /// Panics if points don't have exactly 3 columns or bin_widths doesn't have length 3
    pub fn new(original_points: Array2<f64>, bin_widths: Array1<f64>) -> Self {
        assert_eq!(original_points.ncols(), 3, "Points must have 3 dimensions");
        assert_eq!(bin_widths.len(), 3, "Bin widths must have 3 dimensions");

        let n_points = original_points.nrows();

        // 1. Compute origin and bin indices
        let origin = min_along_axis0(&original_points.view());

        let mut bin_indices = Array2::<i64>::zeros((n_points, 3));
        for i in 0..n_points {
            for j in 0..3 {
                bin_indices[[i, j]] = ((original_points[[i, j]] - origin[j]) / bin_widths[j]).floor() as i64;
            }
        }

        let bin_shape = max_along_axis0_i64(&bin_indices.view()) + 1;

        // 2. Sort points by bin for cache efficiency
        // Create sorting keys based on bin indices
        let mut keys: Vec<(i64, usize)> = Vec::with_capacity(n_points);
        for i in 0..n_points {
            let key = bin_indices[[i, 0]] * bin_shape[1] * bin_shape[2]
                    + bin_indices[[i, 1]] * bin_shape[2]
                    + bin_indices[[i, 2]];
            keys.push((key, i));
        }
        keys.sort_by_key(|&(k, _)| k);

        // Extract sort order
        let sort_order: Vec<usize> = keys.iter().map(|&(_, idx)| idx).collect();

        // Create sorted points array
        let mut points = Array2::<f64>::zeros((n_points, 3));
        let mut original_indices = Array1::<i64>::zeros(n_points);
        for (new_idx, &orig_idx) in sort_order.iter().enumerate() {
            for j in 0..3 {
                points[[new_idx, j]] = original_points[[orig_idx, j]];
            }
            original_indices[new_idx] = orig_idx as i64;
        }

        // 3. Build linked list structure
        let size = (
            bin_shape[0] as usize,
            bin_shape[1] as usize,
            bin_shape[2] as usize
        );
        let mut first_member = Array3::<i64>::from_elem(size, -1);
        let mut next_member = Array1::<i64>::from_elem(n_points, -1);

        // Build linked lists using sorted indices
        for i_sorted in 0..n_points {
            let i_original = sort_order[i_sorted];
            let ix = bin_indices[[i_original, 0]] as usize;
            let iy = bin_indices[[i_original, 1]] as usize;
            let iz = bin_indices[[i_original, 2]] as usize;

            next_member[i_sorted] = first_member[[ix, iy, iz]];
            first_member[[ix, iy, iz]] = i_sorted as i64;
        }

        // Store backups for reset functionality
        let original_first_member = first_member.clone();
        let original_next_member = next_member.clone();

        // Initialize search buffers
        let found_indices_buffer = Array1::<i64>::from_elem(n_points, -1);

        Self {
            original_points,
            points,
            bin_widths,
            origin,
            original_indices,
            bin_shape,
            first_member,
            next_member,
            original_first_member,
            original_next_member,
            found_indices_buffer,
            found_count: 0,
        }
    }

    /// Perform a radius search around a query point
    ///
    /// Finds all points within the specified radius and removes them from the structure.
    /// Results are accumulated and can be retrieved with `found_indices()`.
    ///
    /// # Arguments
    /// * `query_point` - 3D point to search around
    /// * `radius` - Search radius
    ///
    /// # Panics
    /// Panics if query_point doesn't have exactly 3 elements
    pub fn radius_search(&mut self, query_point: &ArrayView1<f64>, radius: f64) {
        assert_eq!(query_point.len(), 3, "Query point must have 3 dimensions");

        // Compute bounding box in bin coordinates
        let min_corner = query_point - radius;
        let max_corner = query_point + radius;

        let mut min_bin = Array1::<i64>::zeros(3);
        let mut max_bin = Array1::<i64>::zeros(3);

        for j in 0..3 {
            min_bin[j] = ((min_corner[j] - self.origin[j]) / self.bin_widths[j]).floor() as i64;
            max_bin[j] = ((max_corner[j] - self.origin[j]) / self.bin_widths[j]).floor() as i64;
        }

        // Clamp to valid range
        for j in 0..3 {
            min_bin[j] = min_bin[j].max(0);
            max_bin[j] = max_bin[j].min(self.bin_shape[j] - 1);
        }

        let radius_sq = radius * radius;

        // Iterate over intersecting bins
        for ix in min_bin[0]..=max_bin[0] {
            for iy in min_bin[1]..=max_bin[1] {
                for iz in min_bin[2]..=max_bin[2] {
                    let mut prev: i64 = -1;
                    let mut i = self.first_member[[ix as usize, iy as usize, iz as usize]];

                    // Traverse linked list
                    while i != -1 {
                        let next_i = self.next_member[i as usize];

                        // Compute distance squared
                        let mut dist_sq = 0.0;
                        for j in 0..3 {
                            let diff = self.points[[i as usize, j]] - query_point[j];
                            dist_sq += diff * diff;
                        }

                        if dist_sq <= radius_sq {
                            // Point found - remove from linked list
                            if prev == -1 {
                                self.first_member[[ix as usize, iy as usize, iz as usize]] = next_i;
                            } else {
                                self.next_member[prev as usize] = next_i;
                            }

                            self.next_member[i as usize] = -2; // Mark as removed
                            self.found_indices_buffer[self.found_count] = i;
                            self.found_count += 1;
                        } else {
                            prev = i;
                        }
                        i = next_i;
                    }
                }
            }
        }
    }

    /// Get the original indices of all found points
    ///
    /// Returns the indices into the original points array that were found
    /// across all radius searches since the last reset.
    ///
    /// # Returns
    /// 1D array of original point indices
    pub fn found_indices(&self) -> Array1<i64> {
        let mut result = Array1::<i64>::zeros(self.found_count);
        for i in 0..self.found_count {
            let sorted_idx = self.found_indices_buffer[i] as usize;
            result[i] = self.original_indices[sorted_idx];
        }
        result
    }

    /// Reset the structure for a fresh search
    ///
    /// Restores all points and clears the found indices buffer.
    pub fn reset(&mut self) {
        self.first_member.assign(&self.original_first_member);
        self.next_member.assign(&self.original_next_member);
        self.found_count = 0;
    }

    /// Get the number of points found so far
    pub fn found_count(&self) -> usize {
        self.found_count
    }

    /// Get a reference to the original points
    pub fn original_points(&self) -> &Array2<f64> {
        &self.original_points
    }

    /// Get the bin shape
    pub fn bin_shape(&self) -> &Array1<i64> {
        &self.bin_shape
    }

    /// Get the origin
    pub fn origin(&self) -> &Array1<f64> {
        &self.origin
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ndarray::array;
    use approx::assert_abs_diff_eq;

    #[test]
    fn test_pointbin_creation() {
        let points = array![
            [0.5, 0.5, 0.5],
            [3.0, 3.0, 3.0],
            [6.0, 5.0, 5.0],
        ];
        let bin_widths = array![5.0, 5.0, 5.0];

        let point_bin = PointBin3D::new(points, bin_widths);

        assert_eq!(point_bin.original_points().nrows(), 3);
        assert_abs_diff_eq!(point_bin.origin()[0], 0.5, epsilon = 1e-10);
    }

    #[test]
    fn test_radius_search_basic() {
        let points = array![
            [0.5, 0.5, 0.5],  // Index 0
            [3.0, 3.0, 3.0],  // Index 1
            [6.0, 5.0, 5.0],  // Index 2
        ];
        let bin_widths = array![5.0, 5.0, 5.0];

        let mut point_bin = PointBin3D::new(points, bin_widths);

        // Search near point 2
        let query = array![5.0, 5.0, 5.0];
        point_bin.radius_search(&query.view(), 1.5);

        let results = point_bin.found_indices();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0], 2);
    }

    #[test]
    fn test_reset() {
        let points = array![
            [0.5, 0.5, 0.5],
            [6.0, 5.0, 5.0],
        ];
        let bin_widths = array![5.0, 5.0, 5.0];

        let mut point_bin = PointBin3D::new(points, bin_widths);

        // First search
        let query = array![5.0, 5.0, 5.0];
        point_bin.radius_search(&query.view(), 1.5);
        assert_eq!(point_bin.found_count(), 1);

        // Reset
        point_bin.reset();
        assert_eq!(point_bin.found_count(), 0);

        // Search again - should find the same point
        point_bin.radius_search(&query.view(), 1.5);
        assert_eq!(point_bin.found_count(), 1);
    }
}
