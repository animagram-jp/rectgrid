#![no_std]
extern crate alloc;
extern crate core;

use core::fmt::{Display, Formatter, Result};

mod rectgrid;
#[cfg(feature = "geometry")]
pub mod geometry;
pub use rectgrid::*;

#[derive(Debug, Clone, Copy)]
pub enum Error {
    OutOfIndex(u32),
    InvalidDefinition,
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
