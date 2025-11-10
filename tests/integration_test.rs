//! Integration tests matching the original Python test suite

use bucket_search::PointBin3D;
use ndarray::array;

#[test]
fn test_pointbin_full_workflow() {
    println!("--- Running PointBin3D Integration Test ---");

    // 1. Setup Test Data (Original Indices: 0, 1, 2, 3, 4)
    let original_points = array![
        [0.5, 0.5, 0.5],    // Index 0: Center of bin (0,0,0)
        [3.0, 3.0, 3.0],    // Index 1: Far from (5,5,5)
        [6.0, 5.0, 5.0],    // Index 2: Close to (5,5,5) - dist=1.0
        [10.0, 10.0, 10.0], // Index 3: Far away, in a different large bin
        [0.0, 0.0, 0.0],    // Index 4: In bin (0,0,0)
    ];

    let bin_widths = array![5.0, 5.0, 5.0];

    // 2. Initialize the structure
    let mut point_bin = PointBin3D::new(original_points, bin_widths);
    println!("Total points: {}", point_bin.original_points().nrows());
    println!("Bin shape: {:?}", point_bin.bin_shape());

    // Verify origin
    assert!((point_bin.origin()[0] - 0.0).abs() < 1e-10);
    assert!((point_bin.origin()[1] - 0.0).abs() < 1e-10);
    assert!((point_bin.origin()[2] - 0.0).abs() < 1e-10);

    // 3. Search 1: Remove points near the center of bin (1,1,1)
    let query_point_1 = array![5.0, 5.0, 5.0];
    let radius_1 = 1.5;

    point_bin.radius_search(&query_point_1.view(), radius_1);
    let results_1 = point_bin.found_indices();

    println!("\n--- Search 1 ---");
    println!("Query: {:?}, Radius: {}", query_point_1, radius_1);
    println!("Found Original Indices: {:?}", results_1);

    assert_eq!(results_1.len(), 1, "Expected 1 point found");
    assert_eq!(results_1[0], 2, "Expected to find point index 2");
    assert_eq!(point_bin.found_count(), 1, "Expected found_count to be 1");

    // 4. Search 2: Attempt to find point 2 again (should fail) and find point 0
    let query_point_2 = array![0.5, 0.5, 0.5];
    let radius_2 = 0.6;

    point_bin.radius_search(&query_point_2.view(), radius_2);
    let results_2 = point_bin.found_indices();

    println!("\n--- Search 2 ---");
    println!("Query: {:?}, Radius: {}", query_point_2, radius_2);
    println!("Found Original Indices: {:?}", results_2);

    // Results should include both search 1 and search 2
    assert_eq!(results_2.len(), 2, "Expected 2 points found total");
    let mut sorted_results = results_2.to_vec();
    sorted_results.sort();
    assert_eq!(sorted_results[0], 0, "Expected to find point index 0");
    assert_eq!(sorted_results[1], 2, "Expected to find point index 2");
    assert_eq!(point_bin.found_count(), 2, "Expected found_count to be 2");

    // 5. Full Reset and Re-test (Point 2 should be back)
    point_bin.reset();

    println!("\n--- Reset and Re-Search ---");
    println!("Structure reset. Re-running Search 1 (should find point 2 again).");

    point_bin.radius_search(&query_point_1.view(), radius_1);
    let results_3 = point_bin.found_indices();

    assert_eq!(results_3.len(), 1, "Reset failed - expected 1 point");
    assert_eq!(results_3[0], 2, "Reset failed - expected point index 2");
    println!("Reset successful. Found Original Indices: {:?}", results_3);

    println!("\n*** ALL TESTS PASSED ***");
}

#[test]
fn test_empty_search() {
    let points = array![
        [0.0, 0.0, 0.0],
        [10.0, 10.0, 10.0],
    ];
    let bin_widths = array![5.0, 5.0, 5.0];

    let mut point_bin = PointBin3D::new(points, bin_widths);

    // Search in an area with no points
    let query = array![5.0, 5.0, 5.0];
    point_bin.radius_search(&query.view(), 0.1);

    let results = point_bin.found_indices();
    assert_eq!(results.len(), 0, "Expected no points found");
}

#[test]
fn test_multiple_points_in_radius() {
    let points = array![
        [0.0, 0.0, 0.0],
        [0.5, 0.0, 0.0],
        [0.0, 0.5, 0.0],
        [0.0, 0.0, 0.5],
        [10.0, 10.0, 10.0],
    ];
    let bin_widths = array![2.0, 2.0, 2.0];

    let mut point_bin = PointBin3D::new(points, bin_widths);

    // Search near origin - should find first 4 points
    let query = array![0.0, 0.0, 0.0];
    point_bin.radius_search(&query.view(), 1.0);

    let results = point_bin.found_indices();
    assert_eq!(results.len(), 4, "Expected 4 points found");

    let mut sorted_results = results.to_vec();
    sorted_results.sort();
    assert_eq!(sorted_results, vec![0, 1, 2, 3]);
}

#[test]
fn test_cumulative_searches() {
    let points = array![
        [0.0, 0.0, 0.0],  // 0
        [1.0, 0.0, 0.0],  // 1
        [2.0, 0.0, 0.0],  // 2
        [3.0, 0.0, 0.0],  // 3
    ];
    let bin_widths = array![1.5, 1.5, 1.5];

    let mut point_bin = PointBin3D::new(points, bin_widths);

    // First search
    let query1 = array![0.0, 0.0, 0.0];
    point_bin.radius_search(&query1.view(), 0.5);
    assert_eq!(point_bin.found_count(), 1);

    // Second search (cumulative)
    let query2 = array![2.0, 0.0, 0.0];
    point_bin.radius_search(&query2.view(), 0.5);
    assert_eq!(point_bin.found_count(), 2);

    let results = point_bin.found_indices();
    let mut sorted_results = results.to_vec();
    sorted_results.sort();
    assert_eq!(sorted_results, vec![0, 2]);
}
