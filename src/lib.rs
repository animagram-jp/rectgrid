#![no_std]
extern crate core;
extern crate alloc;

use core::fmt::{Display, Formatter, Result};

mod rectgrid;
pub use rectgrid::*;

/// rectgridモジュール内のエラー。
/// OutOfIndex: accumulator評価時の範囲外アクセス。範囲内に収まる最後の有効indexを持つ。
/// InvalidDefinition: IncrementFunctionの定義が不正(VectorList(Vec::new())など)で評価クロージャを構築できない。
#[derive(Debug, Clone)]
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