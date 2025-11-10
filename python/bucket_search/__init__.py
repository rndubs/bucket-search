"""
Bucket Search - High-performance 3D spatial indexing library

This library provides efficient radius-based spatial queries for 3D point clouds
using a bucket/binning algorithm implemented in Rust.
"""

from bucket_search._bucket_search import PointBin3D, __version__

__all__ = ["PointBin3D", "__version__"]
