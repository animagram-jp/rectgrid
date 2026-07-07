#![no_std]
extern crate core;
extern crate alloc;

use core::{
    fmt::{self, Display},
    result::Result,
};

#[derive(Debug, Clone)]
pub enum Error {
    Rectgrid,
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Rectgrid => write!(f, "invalid : ") // todo
        }
    }
}