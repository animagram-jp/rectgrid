use core::{
	primitive::{u32, usize, f64},
	result::Result,
	array::from_fn
};
use alloc::{
    vec::Vec,
    boxed::Box, 
    rc::Rc
};

pub type Px    = f64;
pub type Unit  = f64;
pub type Ratio = f64;

pub type Point<const D: usize>  = [Unit; D];
pub type Box<const D: usize>    = (Point<D>, Point<D>); // (base, offset)

// todo: Option式は、幾何定義実装部
pub type Region<const D: usize> = (Vec<Box<D>>, Option(Box<dyn Fn(u32) -> Result<Px, OutOfIndex>>));

#[derive(Debug)]
pub struct OutOfIndex;

pub enum IncrementFunction {
    /// Fn(i) = points[i+1] - points[i]
    /// OutOfIndex means boundary; 引数は差分のindex(整数)
    /// todo: OutOfIndexの分散分布が許されるか検証
    ForwardDifference(Rc<dyn Fn(u32) -> Result<Px, OutOfIndex>>),
    /// boundary
    /// 原点から正方向に間隔の与単位値を列挙した配列。
    VectorList(Vec<Px>),
    /// unboundary
    Scale(f64),
}

impl IncrementFunction {
    /// 式から評価クロージャを生成する。f64 -> px座標(originとの相対距離)。
    /// 端数は線形補間でpxに変換する。
    pub fn accumulate(&self) -> Box<dyn Fn(f64) -> Result<Px, OutOfIndex>> {
        match self {
            // 差分を0..floor(x)で累積し、端数ぶんは次の差分を線形補間で加算。
            Self::ForwardDifference(f) => {
                let f = f.clone();
                Box::new(move |x| {
                    let n = x.floor() as u32;
                    let frac = x - n as f64;
                    let mut acc: Px = 0.0;
                    for k in 0..n {
                        acc += f(k)?;
                    }
                    if frac != 0.0 {
                        acc += f(n)? * frac;
                    }
                    Ok(acc)
                })
            }
            // 累積座標の配列。整数indexで引き、端数は隣との線形補間。範囲外は境界。
            Self::VectorList(pxs) => {
                let pxs = pxs.clone();
                Box::new(move |x| {
                    let n = x.floor() as usize;
                    let frac = x - n as f64;
                    let lo = *pxs.get(n).ok_or(OutOfIndex)?;
                    if frac == 0.0 {
                        return Ok(lo);
                    }
                    let hi = *pxs.get(n + 1).ok_or(OutOfIndex)?;
                    Ok(lo + (hi - lo) * frac)
                })
            }
            // 等間隔スケール。端数もそのまま乗る。境界なし。
            Self::Scale(s) => {
                let s = *s;
                Box::new(move |x| Ok(s * x))
            }
        }
    }
}

pub struct RectGrid<const D: usize> {
    pub origin: [Px; D],
    accumulator: [Option<Box<dyn Fn(Unit) -> Result<Px, OutOfIndex>>>; D]
}

impl<const D: usize> RectGrid<D> {
    pub fn new(&mut self, origin: [Px; D], definitions: [IncrementFunction; D]) -> Self {
			Self {
                origin,
                accumulator = todo!("iterate definitions[d].accumulate() for d: D")
			}
    }

    // todo: structにdefinitionsを追加し、accumulatorをOptionにすることで、originを含めたaffineアルゴリズムへaccumulator最適化可能にするか検討
	pub fn set_definition(&mut self, definition: IncrementFunction, d: D) -> Self {
            self.accumulator[d] = definition.accumulate();
		}

    pub fn as_px(&self, boxes: &Vec<Box<D>>) -> Vec<Result<([Px; D], [Px; D]), OutOfIndex>> {
        let mut out = Vec::with_capacity(boxes.len());
        for box in boxes {
            let base   = self.accumulator(&box.0) + self.origin;
            let offset = self.accumulator(&box.1) + self.origin;
            out.push((base, offset));
        }
        Ok(out)
    }
}

pub fn snap_floor(&mut self, extend: Option<[Unit; D]>) -> &mut Self