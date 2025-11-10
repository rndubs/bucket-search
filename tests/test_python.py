"""
Python integration tests for bucket_search

These tests verify that the Rust implementation provides the same
functionality as the original Python/Numba implementation.
"""

import numpy as np
import pytest


def test_pointbin_basic_workflow():
    """Test basic PointBin3D workflow matching original Python test"""
    # Import here so the test is skipped if the package isn't built
    try:
        from bucket_search import PointBin3D
    except ImportError:
        pytest.skip("bucket_search not built - run 'maturin develop' first")

    print("--- Running PointBin3D Python Test ---")

    # 1. Setup Test Data (Original Indices: 0, 1, 2, 3, 4)
    original_points = np.array([
        [0.5, 0.5, 0.5],    # Index 0: Center of bin (0,0,0)
        [3.0, 3.0, 3.0],    # Index 1: Far from (5,5,5)
        [6.0, 5.0, 5.0],    # Index 2: Close to (5,5,5) - dist=1.0
        [10.0, 10.0, 10.0], # Index 3: Far away, in a different large bin
        [0.0, 0.0, 0.0],    # Index 4: In bin (0,0,0)
    ], dtype=np.float64)

    bin_widths = np.array([5.0, 5.0, 5.0], dtype=np.float64)

    # 2. Initialize the structure
    point_bin = PointBin3D(original_points, bin_widths)
    print(f"Total points: {point_bin.original_points().shape[0]}")
    print(f"Bin shape: {point_bin.bin_shape()}")

    # Verify origin
    assert np.allclose(point_bin.origin(), np.array([0.0, 0.0, 0.0]))

    # 3. Search 1: Remove points near the center of bin (1,1,1)
    query_point_1 = np.array([5.0, 5.0, 5.0], dtype=np.float64)
    radius_1 = 1.5

    point_bin.radius_search(query_point_1, radius_1)
    results_1 = point_bin.found_indices()

    print("\n--- Search 1 ---")
    print(f"Query: {query_point_1}, Radius: {radius_1}")
    print(f"Found Original Indices: {results_1}")

    expected_1 = np.array([2])
    assert np.array_equal(np.sort(results_1), expected_1)
    assert point_bin.found_count() == 1

    # 4. Search 2: Attempt to find point 2 again (should fail) and find point 0
    query_point_2 = np.array([0.5, 0.5, 0.5], dtype=np.float64)
    radius_2 = 0.6

    point_bin.radius_search(query_point_2, radius_2)
    results_2 = point_bin.found_indices()

    print("\n--- Search 2 ---")
    print(f"Query: {query_point_2}, Radius: {radius_2}")
    print(f"Found Original Indices: {results_2}")

    expected_2 = np.array([0, 2])
    assert np.array_equal(np.sort(results_2), expected_2)
    assert point_bin.found_count() == 2

    # 5. Full Reset and Re-test (Point 2 should be back)
    point_bin.reset()

    print("\n--- Reset and Re-Search ---")
    print("Structure reset. Re-running Search 1 (should find point 2 again).")

    point_bin.radius_search(query_point_1, radius_1)
    results_3 = point_bin.found_indices()

    assert np.array_equal(np.sort(results_3), expected_1)
    print(f"Reset successful. Found Original Indices: {results_3}")

    print("\n*** ALL TESTS PASSED ***")


def test_empty_search():
    """Test search with no points in radius"""
    try:
        from bucket_search import PointBin3D
    except ImportError:
        pytest.skip("bucket_search not built")

    points = np.array([
        [0.0, 0.0, 0.0],
        [10.0, 10.0, 10.0],
    ], dtype=np.float64)
    bin_widths = np.array([5.0, 5.0, 5.0], dtype=np.float64)

    point_bin = PointBin3D(points, bin_widths)

    # Search in an area with no points
    query = np.array([5.0, 5.0, 5.0], dtype=np.float64)
    point_bin.radius_search(query, 0.1)

    results = point_bin.found_indices()
    assert len(results) == 0


def test_multiple_points_in_radius():
    """Test finding multiple points within radius"""
    try:
        from bucket_search import PointBin3D
    except ImportError:
        pytest.skip("bucket_search not built")

    points = np.array([
        [0.0, 0.0, 0.0],
        [0.5, 0.0, 0.0],
        [0.0, 0.5, 0.0],
        [0.0, 0.0, 0.5],
        [10.0, 10.0, 10.0],
    ], dtype=np.float64)
    bin_widths = np.array([2.0, 2.0, 2.0], dtype=np.float64)

    point_bin = PointBin3D(points, bin_widths)

    # Search near origin - should find first 4 points
    query = np.array([0.0, 0.0, 0.0], dtype=np.float64)
    point_bin.radius_search(query, 1.0)

    results = point_bin.found_indices()
    assert len(results) == 4

    sorted_results = np.sort(results)
    assert np.array_equal(sorted_results, np.array([0, 1, 2, 3]))


def test_error_handling():
    """Test that appropriate errors are raised for invalid inputs"""
    try:
        from bucket_search import PointBin3D
    except ImportError:
        pytest.skip("bucket_search not built")

    # Test wrong number of dimensions for points
    with pytest.raises(ValueError, match="3 columns"):
        points = np.array([[0.0, 0.0], [1.0, 1.0]], dtype=np.float64)
        bin_widths = np.array([1.0, 1.0, 1.0], dtype=np.float64)
        PointBin3D(points, bin_widths)

    # Test wrong number of dimensions for bin_widths
    with pytest.raises(ValueError, match="3 elements"):
        points = np.array([[0.0, 0.0, 0.0], [1.0, 1.0, 1.0]], dtype=np.float64)
        bin_widths = np.array([1.0, 1.0], dtype=np.float64)
        PointBin3D(points, bin_widths)


def test_repr():
    """Test string representation"""
    try:
        from bucket_search import PointBin3D
    except ImportError:
        pytest.skip("bucket_search not built")

    points = np.array([
        [0.0, 0.0, 0.0],
        [1.0, 1.0, 1.0],
    ], dtype=np.float64)
    bin_widths = np.array([1.0, 1.0, 1.0], dtype=np.float64)

    point_bin = PointBin3D(points, bin_widths)
    repr_str = repr(point_bin)

    assert "PointBin3D" in repr_str
    assert "n_points=2" in repr_str
    assert "found_count=0" in repr_str


if __name__ == "__main__":
    # Run tests directly
    test_pointbin_basic_workflow()
    test_empty_search()
    test_multiple_points_in_radius()
    test_error_handling()
    test_repr()
    print("\n=== All Python tests passed ===")
