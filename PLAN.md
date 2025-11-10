# Rust + Python Bindings Migration Plan

## Overview
Transform the `bucket_search.py` NumPy/Numba implementation into a high-performance Rust package with Python bindings, installable via pip.

## Project Structure
```
bucket-search/
├── Cargo.toml                 # Rust package manifest
├── pyproject.toml            # Python package configuration (maturin)
├── src/
│   ├── lib.rs                # Rust library entry point + Python bindings
│   ├── pointbin.rs           # Core PointBin3D implementation
│   └── utils.rs              # Helper functions (min/max along axis, etc.)
├── tests/
│   ├── test_rust.rs          # Rust unit tests
│   └── test_python.py        # Python integration tests
├── benches/
│   └── benchmark.rs          # Rust benchmarks
├── .github/
│   └── workflows/
│       ├── ci.yml            # CI/CD workflow
│       └── release.yml       # Release workflow for PyPI
├── bucket_search.py          # Original (keep for reference/comparison)
└── PLAN.md                   # This file
```

---

## Phase 1: Rust Core Implementation ✅
**Goal:** Implement the spatial indexing data structure in pure Rust

### 1.1 Project Setup
- [x] Initialize Rust project with `cargo init --lib`
- [x] Configure `Cargo.toml` with dependencies (ndarray for array operations)
- [x] Set up workspace structure

### 1.2 Core Data Structures
- [x] Implement `PointBin3D` struct with required fields:
  - `original_points`: 2D array of f64
  - `points`: Cache-friendly sorted copy
  - `bin_widths`: 1D array of f64
  - `origin`: 1D array of f64
  - `original_indices`: Index mapping
  - `bin_indices`: 2D array of i64
  - `bin_shape`: 1D array of i64
  - `first_member`: 3D array (linked list heads)
  - `next_member`: 1D array (linked list)
  - Internal state for reset functionality
  - `found_indices` and `found_count`

### 1.3 Utility Functions
- [x] Implement `min_along_axis0` (column-wise minimum)
- [x] Implement `max_along_axis0` (column-wise maximum)
- [x] Helper functions for spatial operations

### 1.4 Core Methods
- [x] Implement `new()` constructor:
  - Compute bin indices from points
  - Sort points by bin for cache efficiency
  - Build linked list structure
  - Store original state for reset
- [x] Implement `radius_search()`:
  - Calculate bounding box in bin coordinates
  - Iterate over intersecting bins
  - Check distances using vectorized operations
  - Remove found points from linked list
  - Track found indices
- [x] Implement `found_indices()`:
  - Map sorted indices back to original indices
- [x] Implement `reset()`:
  - Restore original linked list structure
  - Clear found indices

### 1.5 Rust Testing
- [x] Write unit tests for utility functions
- [x] Write integration tests matching Python test cases
- [x] Test edge cases (empty bins, single point, boundary conditions)

### 1.6 Performance Optimization
- [x] Profile Rust implementation
- [x] Use `#[inline]` for hot paths
- [ ] Consider SIMD optimizations for distance calculations (future enhancement)
- [x] Optimize memory layout for cache efficiency

---

## Phase 2: Python Bindings with PyO3/Maturin ✅
**Goal:** Expose Rust functionality to Python with zero-copy NumPy integration

### 2.1 Maturin Setup
- [x] Add `pyo3` dependency to `Cargo.toml` with `extension-module` feature
- [x] Create `pyproject.toml` with maturin build backend
- [x] Configure package metadata (name, version, authors, description)

### 2.2 Python Bindings
- [x] Implement `#[pyclass]` wrapper for `PointBin3D`:
  - Use `numpy` crate for array conversions
  - Handle `PyArray` for NumPy compatibility
  - Implement zero-copy views where possible
- [x] Expose constructor accepting NumPy arrays:
  - `__init__(points: np.ndarray, bin_widths: np.ndarray)`
- [x] Expose methods as `#[pymethods]`:
  - `radius_search(query_point: np.ndarray, radius: float)`
  - `found_indices() -> np.ndarray`
  - `reset()`
- [x] Add Python docstrings to all exposed functions
- [x] Implement `__repr__` and `__str__` for debugging

### 2.3 Type Safety & Error Handling
- [x] Validate input dimensions (3D points)
- [x] Handle dimension mismatches with clear error messages
- [x] Use `PyResult` for fallible operations
- [x] Add runtime checks for array shapes

### 2.4 Python Testing
- [x] Port existing `run_pointbin_test()` to pytest
- [x] Test NumPy array compatibility
- [x] Test error handling and edge cases
- [x] Compare results with original Python implementation
- [x] Test memory safety (no segfaults, proper cleanup)

---

## Phase 3: Integration Testing & Validation
**Goal:** Ensure feature parity and correctness

### 3.1 Functional Testing
- [ ] Create test data generators (random point clouds)
- [ ] Test with various point cloud sizes (10, 100, 1000, 10000+ points)
- [ ] Test with different bin widths
- [ ] Test multiple searches without reset
- [ ] Test reset functionality
- [ ] Verify found indices match original implementation

### 3.2 Numerical Accuracy
- [ ] Compare results between Rust and Python implementations
- [ ] Test floating-point edge cases
- [ ] Verify distance calculations are identical

### 3.3 Performance Benchmarking
- [ ] Create Rust criterion benchmarks
- [ ] Create Python benchmarks (timeit)
- [ ] Compare against original Numba implementation
- [ ] Document performance improvements
- [ ] Test with large datasets (100K+ points)

### 3.4 Memory Testing
- [ ] Verify no memory leaks using valgrind or miri
- [ ] Test with large datasets
- [ ] Monitor memory usage vs Python implementation

---

## Phase 4: Build & Distribution Workflows
**Goal:** Automate building, testing, and releasing

### 4.1 Local Development Setup
- [ ] Create `.gitignore` for Rust/Python artifacts
- [ ] Document development setup in README
- [ ] Create development scripts:
  - Build: `maturin develop` for local testing
  - Test: run both Rust and Python tests
  - Benchmark: run performance tests

### 4.2 GitHub Actions CI Workflow
- [ ] Set up matrix testing:
  - Python versions: 3.8, 3.9, 3.10, 3.11, 3.12
  - OS: Ubuntu, macOS, Windows
  - Architectures: x86_64, aarch64 (ARM)
- [ ] Jobs:
  - Lint (clippy, rustfmt)
  - Rust tests
  - Build Python wheels with maturin
  - Python integration tests
  - Benchmarks (optional, on main branch)

### 4.3 Release Workflow
- [ ] Automated wheel building for multiple platforms
- [ ] Use `maturin build --release --strip` for production builds
- [ ] Build manylinux wheels for Linux compatibility
- [ ] Build macOS universal2 wheels
- [ ] Build Windows wheels
- [ ] Upload to PyPI using trusted publishing or API tokens
- [ ] Create GitHub releases with artifacts

### 4.4 PyPI Package Configuration
- [ ] Configure `pyproject.toml` with:
  - Package classifiers
  - Python version requirements (>=3.8)
  - Project URLs (repository, issues)
  - License information
  - Keywords for discoverability
- [ ] Create `README.md` with installation and usage examples
- [ ] Add `LICENSE` file

---

## Phase 5: Documentation & Polish
**Goal:** Production-ready package

### 5.1 Documentation
- [ ] Write comprehensive README.md:
  - Installation instructions (`pip install bucket-search`)
  - Quick start example
  - API reference
  - Performance benchmarks
  - Migration guide from old Python version
- [ ] Add inline documentation (Rust doc comments + Python docstrings)
- [ ] Create examples directory with usage patterns
- [ ] Document build process for contributors

### 5.2 Package Quality
- [ ] Add package metadata badges (CI status, PyPI version, license)
- [ ] Set up code coverage tracking
- [ ] Enable dependabot for dependency updates
- [ ] Add contributing guidelines

### 5.3 Migration & Compatibility
- [ ] Create compatibility layer if API changes
- [ ] Document breaking changes from original
- [ ] Provide deprecation warnings if needed
- [ ] Keep original `bucket_search.py` for reference

---

## Phase 6: Finalization
**Goal:** Deploy and validate

### 6.1 Pre-release Testing
- [ ] Test installation from TestPyPI
- [ ] Test on clean environments (Docker containers)
- [ ] Validate all platforms (Linux, macOS, Windows)
- [ ] Run full test suite on all target Python versions

### 6.2 Release
- [ ] Tag version in git (e.g., v0.1.0)
- [ ] Build and publish to PyPI
- [ ] Create GitHub release with changelog
- [ ] Announce release

### 6.3 Post-release
- [ ] Monitor for installation issues
- [ ] Address any platform-specific bugs
- [ ] Gather performance feedback
- [ ] Plan future improvements

---

## Success Criteria
- ✓ Rust implementation passes all original Python tests
- ✓ Python bindings provide identical API
- ✓ Performance matches or exceeds Numba implementation
- ✓ Package installable via `pip install bucket-search`
- ✓ Works on Linux, macOS, and Windows
- ✓ Supports Python 3.8+
- ✓ CI/CD pipeline automatically builds and tests
- ✓ Documentation is clear and complete

---

## Technical Stack
- **Rust:** Latest stable (1.70+)
- **PyO3:** 0.20+ (Python bindings)
- **Maturin:** 1.0+ (build system)
- **ndarray:** Array operations in Rust
- **numpy:** Python NumPy compatibility
- **pytest:** Python testing
- **criterion:** Rust benchmarking

---

## Timeline Estimate
- Phase 1 (Rust Core): 2-3 days
- Phase 2 (Python Bindings): 1-2 days
- Phase 3 (Integration Testing): 1 day
- Phase 4 (Build Workflows): 1 day
- Phase 5 (Documentation): 1 day
- Phase 6 (Release): 0.5 days
- **Total:** ~7-9 days for complete implementation

---

## Notes
- Prioritize correctness over premature optimization
- Maintain feature parity with original implementation
- Zero-copy NumPy integration is critical for performance
- Test thoroughly across all target platforms
- Keep original Python implementation for comparison testing
