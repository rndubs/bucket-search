from numba import njit,  float64, int64
from numba.experimental import jitclass
import numpy as np


@njit
def min_along_axis0(arr):
    out = arr[0].copy()
    for i in range(arr.shape[0]):
        for j in range(arr.shape[1]):
            if arr[i, j] < out[j]:
                out[j] = arr[i, j]
    return out

@njit
def max_along_axis0(arr):
    out = arr[0].copy()
    for i in range(arr.shape[0]):
        for j in range(arr.shape[1]):
            if arr[i, j] > out[j]:
                out[j] = arr[i, j]
    return out


spec = [
    # New: The original data, for reference/resetting the structure
    ('original_points', float64[:, :]),
    # New: The cache-friendly, bin-sorted copy of points
    ('points', float64[:, :]),
    ('bin_widths', float64[:]),
    ('origin', float64[:]),
    # New: Maps the index in 'points' back to the index in 'original_points'
    ('original_indices', int64[:]),
    ('bin_indices', int64[:, :]),
    ('bin_shape', int64[:]),
    ('first_member', int64[:, :, :]),
    ('next_member', int64[:]),
    ('original_first_member', int64[:, :, :]),
    ('original_next_member', int64[:]),
    ('original_next_member_copy', int64[:]), # Used for reset
    ('_found_indices', int64[:]),
    ('found_count', int64),
]

@jitclass(spec)
class PointBin3D_JIT:
    def __init__(self, original_points, bin_widths):
        self.original_points = original_points
        self.bin_widths = bin_widths
        self.origin = min_along_axis0(original_points)
        n_points = original_points.shape[0]

        # 1. Compute bin indices
        self.bin_indices = np.floor((original_points - self.origin) / bin_widths).astype(np.int64)
        self.bin_shape = max_along_axis0(self.bin_indices) + 1
        size_tuple = (int(self.bin_shape[0]), int(self.bin_shape[1]), int(self.bin_shape[2]))

        # 2. Sort/Reorder Data (The Cache Boost)
        # Create a combined key for stable sorting by bin indices (ix, iy, iz)
        # This determines the final contiguous order.
        keys = (self.bin_indices[:, 0] * self.bin_shape[1] * self.bin_shape[2] +
                self.bin_indices[:, 1] * self.bin_shape[2] +
                self.bin_indices[:, 2])

        # Get the new sorted order (indices into original_points)
        sort_order = np.argsort(keys)

        # Create the cache-friendly, sorted points and original index map
        self.points = original_points[sort_order].copy()
        self.original_indices = sort_order.copy()

        # Now, bin_indices needs to refer to the NEW (sorted) index
        # This is essentially the same as doing: self.bin_indices = self.bin_indices[sort_order]
        # But we don't strictly need to update self.bin_indices if we rebuild pointers

        # 3. Initialize/Build Linked Lists (using NEW indices 0 to N-1)
        self.first_member = np.full(size_tuple, -1, dtype=np.int64)
        self.next_member = np.full(n_points, -1, dtype=np.int64)

        # Iterate over the NEW sorted indices (i_sorted)
        for i_sorted in range(n_points):
            # i_sorted is the new index.
            # We look up the original index to find its bin coords.
            i_original = sort_order[i_sorted]
            ix, iy, iz = self.bin_indices[i_original]

            # The linked list connects the NEW indices.
            self.next_member[i_sorted] = self.first_member[ix, iy, iz]
            self.first_member[ix, iy, iz] = i_sorted

        # Store originals for reset
        self.original_first_member = self.first_member.copy()
        self.original_next_member = self.next_member.copy()
        # Numba requires a direct copy assignment for reset function
        self.original_next_member_copy = self.original_next_member.copy()

        # Running list of found indices
        # We store the *new, sorted* indices first, then map to original indices on output
        self._found_indices = np.full(n_points, -1, dtype=np.int64)
        self.found_count = 0

    def radius_search(self, query_point, radius):
        # Compute the bounding box in bin coordinates (using original_points.shape[1] dimensions)
        min_corner = query_point - radius
        max_corner = query_point + radius

        # Use np.int64 for consistency
        min_bin = np.floor((min_corner - self.origin) / self.bin_widths).astype(np.int64)
        max_bin = np.floor((max_corner - self.origin) / self.bin_widths).astype(np.int64)

        # Clamp bin indices to valid range (Vectorized)
        min_bin = np.maximum(min_bin, 0)
        max_bin = np.minimum(max_bin, self.bin_shape - 1)

        radius_sq = radius ** 2

        # Iterate over all bins that may intersect the search sphere
        for ix in range(min_bin[0], max_bin[0] + 1):
            for iy in range(min_bin[1], max_bin[1] + 1):
                for iz in range(min_bin[2], max_bin[2] + 1):
                    prev = -1
                    i = self.first_member[ix, iy, iz] # i is the NEW, sorted index

                    # Traverse the linked list for this bin
                    while i != -1:
                        next_i = self.next_member[i]

                        # p% VECTORIZED DISTANCE CHECK (Accessing contiguous memory)
                        diff = self.points[i] - query_point
                        d = np.sum(diff * diff)

                        if d <= radius_sq:
                            # Point is found and is in the cache-friendly 'points' array

                            # Remove from linked list
                            if prev == -1:
                                self.first_member[ix, iy, iz] = next_i
                            else:
                                self.next_member[prev] = next_i

                            self.next_member[i] = -2 # Use -2 as a clear 'removed' marker (or use a separate array as discussed)
                            self._found_indices[self.found_count] = i # Store the NEW index
                            self.found_count += 1
                            # prev does not advance
                        else:
                            prev = i
                        i = next_i

    def found_indices(self):
        """Returns the original indices of points found within the radius."""
        # Maps the NEW (sorted) indices to the ORIGINAL indices before returning
        found_sorted_indices = self._found_indices[:self.found_count]
        return self.original_indices[found_sorted_indices].copy()

    def reset(self):
        """Restores the structure for a fresh search on all points."""
        self.first_member[:, :, :] = self.original_first_member
        # Use the stored copy for assignment, as Numba is strict about array aliasing
        self.next_member[:] = self.original_next_member_copy
        self.found_count = 0

# --- Test Setup ---
# (Place this after your PointBin3D_JIT class definition)

def run_pointbin_test():
    print("--- Running PointBin3D_JIT Test ---")

    # 1. Setup Test Data (Original Indices: 0, 1, 2, 3, 4)
    # ----------------------------------------------------
    original_points = np.array([
        [0.5, 0.5, 0.5],  # Index 0: Center of bin (0,0,0)
        [3.0, 3.0, 3.0],  # Index 1: Far from (5,5,5)
        [6.0, 5.0, 5.0],  # Index 2: Close to (5,5,5) - dist=1.0
        [10.0, 10.0, 10.0], # Index 3: Far away, in a different large bin
        [0.0, 0.0, 0.0],  # Index 4: In bin (0,0,0)
    ], dtype=np.float64)

    bin_widths = np.array([5.0, 5.0, 5.0], dtype=np.float64)

    # 2. Initialize the JIT Class
    # ---------------------------
    point_bin = PointBin3D_JIT(original_points, bin_widths)
    print(f"Total points: {point_bin.points.shape[0]}")
    print(f"Bin shape: {point_bin.bin_shape}")

    assert np.allclose(point_bin.origin, np.array([0.0, 0.0, 0.0]))

    # 3. Search 1: Remove points near the center of bin (1,1,1)
    # -----------------------------------------------------------
    query_point_1 = np.array([5.0, 5.0, 5.0], dtype=np.float64)
    radius_1 = 1.5

    point_bin.radius_search(query_point_1, radius_1)
    results_1 = point_bin.found_indices()

    print("\n--- Search 1 ---")
    print(f"Query: {query_point_1}, Radius: {radius_1}")
    print(f"Found Original Indices: {results_1}")

    expected_1 = np.array([2])
    assert np.array_equal(np.sort(results_1), expected_1), f"Expected {expected_1}, got {results_1}"
    assert point_bin.found_count == 1, f"Expected found_count 1, got {point_bin.found_count}"

    assert point_bin.first_member[1, 1, 1] == -1, "Point was not removed from bin head."

    # 4. Search 2: Attempt to find point 2 again (should fail) and find point 0
    # --------------------------------------------------------------------------
    query_point_2 = np.array([0.5, 0.5, 0.5], dtype=np.float64)
    radius_2 = 0.6

    point_bin.radius_search(query_point_2, radius_2)
    results_2 = point_bin.found_indices()

    print("\n--- Search 2 ---")
    print(f"Query: {query_point_2}, Radius: {radius_2}")
    print(f"Found Original Indices: {results_2}")

    expected_2 = np.array([0, 2])
    assert np.array_equal(np.sort(results_2), expected_2), f"Expected {expected_2}, got {results_2}"
    assert point_bin.found_count == 2, f"Expected found_count 2, got {point_bin.found_count}"

    # 5. Full Reset and Re-test (Point 2 should be back)
    # -------------------------------------------------
    point_bin.reset()

    print("\n--- Reset and Re-Search ---")
    print("Structure reset. Re-running Search 1 (should find point 2 again).")

    point_bin.radius_search(query_point_1, radius_1)
    results_3 = point_bin.found_indices()

    assert np.array_equal(np.sort(results_3), expected_1), f"Reset failed. Expected {expected_1}, got {results_3}"
    print(f"Reset successful. Found Original Indices: {results_3}")

    print("\n*** ALL TESTS PASSED ***")

# ---------------------------------------------------------------------
# The main execution block
# ---------------------------------------------------------------------

if __name__ == "__main__":
    run_pointbin_test()

