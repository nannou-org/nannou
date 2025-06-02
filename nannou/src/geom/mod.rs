//! Types, functions and other items related to geometry. This module is the source of all graphics
//! and lazer primitives and aids work in 2D and 3D space.
//!
//! Each module provides a set of general tools for working with the named geometry including:
//!
//! - A typed, object representation.
//! - Functions for producing vertices, triangles and triangulation indices.
//! - Functions for checking whether or not the geometry contains a point.
//! - Functions for determining the bounding rectangle or cuboid.
//! - A function for finding the centroid.

pub use nannou_core::geom::*;

pub use self::path::{Path, path};

pub mod path;
