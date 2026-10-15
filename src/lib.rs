//! Boundary box operations on rectilinear grids with arbitrary unit systems.
//!
//! A rectilinear grid is an orthogonal grid whose axes each have an independent increment
//! function. [`RectGrid`] defines such a grid by an origin and one [`IncrementFunction`] per
//! axis, both expressed in the base unit [`Px`]. Coordinates on the grid are [`Unit`] values,
//! and a [`BBox`] is a boundary box between two points in unit coordinates. For a single
//! boundary box, [`RectGrid::get_parameter`] converts a point into the local unit system
//! [`Parameter`], in which each side of the boundary box has length 1; this makes boundary
//! tests against arbitrary geometry possible (see the `geometry` module).
//!
//! The base unit system is the unit system whose origin lies at (0, ..., 0) and whose
//! per-axis increment functions all return the constant 1. Its unit is named Px
//! (pixel: picture element).
//!
//! # Coordinate system
//!
//! - When treating x and y as 2D coordinates, x is the axis that becomes the width in the
//!   viewport, y is the height direction, and the origin (0, 0) is the top-left corner.
//! - Px passed into a function from outside this crate is global: an external coordinate not
//!   yet corrected for the origin, e.g. a viewport coordinate. Each function subtracts the
//!   origin internally to make it local. Px derived from a boundary box, such as the return
//!   value of [`RectGrid::unit_to_px`] and anything built on it (`hit_test`, `*_as_px`), is
//!   always local: the origin is at 0. If such a value is passed back across a [`RectGrid`]
//!   boundary, treat it as local Px.
//!
//! # Feature flags
//!
//! - `geometry`: enables the `geometry` module, which gives the signed distance from a point
//!   to a line, circle, ellipse or polygon in unit coordinates.

#![no_std]
#![warn(missing_docs)]
extern crate alloc;
extern crate core;

use core::fmt::{Display, Formatter, Result};

mod rectgrid;
#[cfg(feature = "geometry")]
pub mod geometry;
pub use rectgrid::*;

/// Error of this crate. Every fallible function returns one of these variants.
#[derive(Debug, Clone, Copy)]
pub enum Error {
    /// An access outside the domain of an axis. Carries the nearest valid index:
    /// 0 when the access is below the domain, the last valid index when it is above.
    OutOfIndex(u32),
    /// The definition cannot be evaluated, so no accumulator is built from it
    /// (e.g. an empty `VectorList`).
    InvalidDefinition,
    /// An argument cannot be evaluated (e.g. NaN).
    InvalidInput,
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match self {
            Error::OutOfIndex(last) => {
                write!(f, "out of index: last valid index is {}", last)
            }
            Error::InvalidDefinition => write!(f, "invalid definition"),
            Error::InvalidInput => write!(f, "invalid input"),
        }
    }
}
