use bucket_search::PointBin3D;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use ndarray::{array, Array1, Array2};

fn create_random_points(n: usize) -> Array2<f64> {
    // Create a simple deterministic pattern instead of random for consistency
    let mut points = Array2::<f64>::zeros((n, 3));
    for i in 0..n {
        let fi = i as f64;
        points[[i, 0]] = (fi * 1.234) % 100.0;
        points[[i, 1]] = (fi * 2.345) % 100.0;
        points[[i, 2]] = (fi * 3.456) % 100.0;
    }
    points
}

fn bench_construction(c: &mut Criterion) {
    let mut group = c.benchmark_group("construction");

    for size in [100, 1000, 10000].iter() {
        let points = create_random_points(*size);
        let bin_widths = array![5.0, 5.0, 5.0];

        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| {
                PointBin3D::new(black_box(points.clone()), black_box(bin_widths.clone()))
            });
        });
    }

    group.finish();
}

fn bench_radius_search(c: &mut Criterion) {
    let mut group = c.benchmark_group("radius_search");

    for size in [100, 1000, 10000].iter() {
        let points = create_random_points(*size);
        let bin_widths = array![5.0, 5.0, 5.0];

        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            let mut point_bin = PointBin3D::new(points.clone(), bin_widths.clone());
            let query = array![50.0, 50.0, 50.0];

            b.iter(|| {
                point_bin.reset();
                point_bin.radius_search(black_box(&query.view()), black_box(2.0));
            });
        });
    }

    group.finish();
}

fn bench_multiple_searches(c: &mut Criterion) {
    let points = create_random_points(1000);
    let bin_widths = array![5.0, 5.0, 5.0];
    let mut point_bin = PointBin3D::new(points, bin_widths);

    let queries = vec![
        array![25.0, 25.0, 25.0],
        array![50.0, 50.0, 50.0],
        array![75.0, 75.0, 75.0],
    ];

    c.bench_function("multiple_searches", |b| {
        b.iter(|| {
            point_bin.reset();
            for query in &queries {
                point_bin.radius_search(black_box(&query.view()), black_box(2.0));
            }
            point_bin.found_indices()
        });
    });
}

criterion_group!(benches, bench_construction, bench_radius_search, bench_multiple_searches);
criterion_main!(benches);
