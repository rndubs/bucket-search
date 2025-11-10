# Bucket Search

[![CI](https://github.com/rndubs/bucket-search/actions/workflows/ci.yml/badge.svg)](https://github.com/rndubs/bucket-search/actions/workflows/ci.yml)
[![PyPI](https://img.shields.io/pypi/v/bucket-search)](https://pypi.org/project/bucket-search/)
[![Python Versions](https://img.shields.io/pypi/pyversions/bucket-search)](https://pypi.org/project/bucket-search/)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue)](https://github.com/rndubs/bucket-search)

A high-performance 3D spatial indexing library for efficient radius-based searches, implemented in Rust with Python bindings.

## Building the Python Wheel

### Prerequisites

- Rust 1.70+ ([Install](https://rustup.rs/))
- Python 3.8+
- `uv` for Python package management ([Install](https://docs.astral.sh/uv/getting-started/installation/))

### Build Instructions

```bash
# Clone the repository
git clone https://github.com/rndubs/bucket-search.git
cd bucket-search

# Install uv (if not already installed)
curl -LsSf https://astral.sh/uv/install.sh | sh

# Install dependencies
uv sync

# Build the wheel
uv build

# The wheel will be created in the dist/ directory
# Install it with: pip install dist/bucket_search-*.whl
```

### Development Build

For development with editable installation:

```bash
# Build and install in development mode
uv run maturin develop --release

# Run tests
cargo test          # Rust tests
uv run pytest       # Python tests

# Run benchmarks
cargo bench
```

## Features

- **⚡ Blazingly Fast**: Rust implementation with cache-optimized data structures
- **🐍 Python-Friendly**: Seamless NumPy integration with zero-copy operations
- **🔍 Efficient Search**: O(1) average-case radius searches using spatial binning
- **💾 Memory Efficient**: Cache-friendly point sorting and linked-list structure
- **🧪 Well-Tested**: Comprehensive test suite for both Rust and Python
- **📦 Easy Install**: Simple `pip install bucket-search`

## Installation

### From PyPI (Recommended)

```bash
# Using pip
pip install bucket-search

# Or using uv
uv add bucket-search
```

### From Source

Requirements: Rust 1.70+ and Python 3.8+

```bash
# Install uv (if not already installed)
curl -LsSf https://astral.sh/uv/install.sh | sh

# Build and install
uv run maturin develop --release
```

## Quick Start

```python
import numpy as np
from bucket_search import PointBin3D

# Create a point cloud
points = np.array([
    [0.0, 0.0, 0.0],
    [1.0, 1.0, 1.0],
    [5.0, 5.0, 5.0],
    [10.0, 10.0, 10.0],
], dtype=np.float64)

# Define bin widths for spatial partitioning
bin_widths = np.array([2.0, 2.0, 2.0], dtype=np.float64)

# Create the spatial index
point_bin = PointBin3D(points, bin_widths)

# Search for points within radius 1.5 of the origin
query_point = np.array([0.0, 0.0, 0.0], dtype=np.float64)
point_bin.radius_search(query_point, radius=1.5)

# Get indices of found points
found_indices = point_bin.found_indices()
print(f"Found {len(found_indices)} points: {found_indices}")
# Output: Found 2 points: [0 1]

# Retrieve found points
found_points = points[found_indices]
print(f"Found points:\n{found_points}")

# Reset for a new search
point_bin.reset()
```

## API Reference

### `PointBin3D`

A 3D spatial indexing structure using binning for efficient radius searches.

#### Constructor

```python
PointBin3D(points: np.ndarray, bin_widths: np.ndarray)
```

**Parameters:**
- `points` (np.ndarray): 2D array of shape `(n_points, 3)` with point coordinates
- `bin_widths` (np.ndarray): 1D array of shape `(3,)` with bin widths for x, y, z dimensions

**Raises:**
- `ValueError`: If `points` doesn't have exactly 3 columns or `bin_widths` doesn't have length 3

#### Methods

##### `radius_search(query_point, radius)`

Find all points within a specified radius of a query point.

**Parameters:**
- `query_point` (np.ndarray): 1D array of shape `(3,)` with query coordinates
- `radius` (float): Search radius

**Notes:**
- Found points are removed from the search structure
- Results accumulate across multiple calls
- Use `reset()` to restore all points

##### `found_indices()`

Get the original indices of all found points.

**Returns:**
- `np.ndarray`: 1D array of int64 indices

##### `reset()`

Reset the structure to restore all points for a fresh search.

##### `found_count()`

Get the number of points found since the last reset.

**Returns:**
- `int`: Count of found points

##### `original_points()`

Get a copy of the original points array.

**Returns:**
- `np.ndarray`: 2D array of shape `(n_points, 3)`

##### `bin_shape()`

Get the shape of the bin grid.

**Returns:**
- `np.ndarray`: 1D array of shape `(3,)` with number of bins in each dimension

##### `origin()`

Get the origin point (minimum corner of the grid).

**Returns:**
- `np.ndarray`: 1D array of shape `(3,)`

## How It Works

The library uses a spatial binning algorithm to accelerate nearest-neighbor searches:

1. **Binning**: Points are partitioned into a 3D grid of bins based on `bin_widths`
2. **Cache Optimization**: Points are sorted by bin for contiguous memory access
3. **Linked Lists**: Each bin maintains a linked list of its points for O(1) removal
4. **Efficient Search**: Only bins intersecting the search sphere are checked
5. **Dynamic Removal**: Found points are removed from the structure during search

This approach provides:
- **Construction**: O(n log n) due to sorting
- **Radius Search**: O(k) where k is the number of points in intersecting bins
- **Memory**: O(n) with excellent cache locality

## Performance

Benchmarks on various point cloud sizes (Apple M1, single-threaded):

| Operation | 100 points | 1,000 points | 10,000 points |
|-----------|------------|--------------|---------------|
| Construction | 7 µs | 46 µs | 518 µs |
| Radius Search | 1.7 µs | 2.0 µs | 3.7 µs |

The Rust implementation provides significant speedups over pure Python implementations, especially for large point clouds and multiple queries.

## Use Cases

- **Point Cloud Processing**: LiDAR data, 3D scanning, photogrammetry
- **Particle Simulations**: Physics engines, molecular dynamics
- **Spatial Queries**: GIS applications, geographic data analysis
- **Robotics**: Path planning, obstacle detection
- **Computer Graphics**: Collision detection, proximity queries

## Development

See the [Building the Python Wheel](#building-the-python-wheel) section above for setup instructions.

### Project Structure

```
bucket-search/
├── src/
│   ├── lib.rs              # Library entry point
│   ├── pointbin.rs         # Core PointBin3D implementation
│   ├── utils.rs            # Utility functions
│   └── python_bindings.rs  # PyO3 bindings
├── tests/
│   ├── integration_test.rs # Rust integration tests
│   └── test_python.py      # Python integration tests
├── benches/
│   └── pointbin_bench.rs   # Performance benchmarks
├── Cargo.toml              # Rust package manifest
├── pyproject.toml          # Python package configuration
└── PLAN.md                 # Implementation plan
```

## Migration from NumPy/Numba

If you're migrating from the original Python/Numba implementation:

1. Replace `from bucket_search import PointBin3D_JIT` with `from bucket_search import PointBin3D`
2. The API is identical - no code changes needed!
3. Remove `numba` dependency from your requirements
4. Enjoy better performance and easier deployment

## Contributing

Contributions are welcome! Please:

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes
4. Run tests (`cargo test && pytest`)
5. Commit your changes (`git commit -m 'Add amazing feature'`)
6. Push to the branch (`git push origin feature/amazing-feature`)
7. Open a Pull Request

## License

This project is dual-licensed under:

- MIT License ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)

You may choose either license for your use.

## Acknowledgments

- Built with [PyO3](https://github.com/PyO3/pyo3) for Python bindings
- Uses [ndarray](https://github.com/rust-ndarray/ndarray) for array operations
- Packaged with [Maturin](https://github.com/PyO3/maturin)

## Citation

If you use this library in your research, please cite:

```bibtex
@software{bucket_search,
  title = {Bucket Search: High-Performance 3D Spatial Indexing},
  author = {bucket-search contributors},
  year = {2025},
  url = {https://github.com/rndubs/bucket-search}
}
```
