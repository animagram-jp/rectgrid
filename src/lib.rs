#![no_std]
extern crate core;
extern crate alloc;

use core::fmt::{Display, Formatter, Result};

mod rectgrid;
pub use rectgrid::*;

#[derive(Debug, Clone, Copy)]
pub enum RectgridError {
    OutOfIndex(u32),
    InvalidDefinition,
}

impl Display for RectgridError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match self {
            RectgridError::OutOfIndex(last) => write!(f, "out of index: last valid index is {}", last),
            RectgridError::InvalidDefinition => write!(f, "invalid definition"),
        }
    }
}