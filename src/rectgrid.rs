// This file includes untranslated text (ja).

use core::{primitive::{u32, usize, f64}, result::Result, array::from_fn};
use alloc::{vec::Vec, boxed::Box, rc::Rc};

// 前提: RectGrid自体は各次元D間のPxの関係性を扱わないが、幾何実装の上では、D間で各Pxは等しく出力される。

// When treating the rectgrid module of x and y as 2D coordinates,
// x represents the axis that becomes the width in the viewport.
// y represents the height direction in the viewport.
// The origin (0,0) is assumed to be the top-left corner.

/// 原点から正方向へ無限に広がる、直交独立単位系。
pub type Px = f64;

/// Bondary Boxの1つに対して、各辺長を1とし、符号をunit座標に従った、無界な局所座標の単位系。
pub type Parameter = f64;

#[derive(Clone, Copy)]
pub enum Unit {
    Px(f64),
    Unit(f64),
    Parameter(f64),
}

/// RectGridのaccumulator内でのみ使う、範囲外アクセスを示すエラー型。
/// as_on_line側の有界判定（tの範囲）とは別軸の話であることに注意。
#[derive(Debug)]
pub struct OutOfIndex;

pub type Point<const D: usize>  = [Unit; D];
pub type BBox<const D: usize>    = (Point<D>, Point<D>); // (base, offset)

// todo: Option式は、幾何定義実装部
// pub type Region<const D: usize> = (Vec<BBox<D>>, Option<Box<dyn Fn(u32) -> Result<Px, OutOfIndex>>>);

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
            Self::Scale(s) => {
                let s = *s;
                Box::new(move |x| Ok(s * x))
            }
        }
    }
}

pub struct RectGrid<const D: usize> {
    pub origin: [Px; D],
    accumulator: [Option<Box<dyn Fn(f64) -> Result<Px, OutOfIndex>>>; D],
}

impl<const D: usize> RectGrid<D> {
    pub fn new(origin: [Px; D], definitions: [IncrementFunction; D]) -> Self {
        Self {
            origin,
            accumulator: definitions.map(|d| Some(d.accumulate())),
        }
    }

    pub fn set_definition(&mut self, definition: IncrementFunction, d: usize) {
        self.accumulator[d] = Some(definition.accumulate());
    }

    fn eval(&self, i: usize, v: f64) -> Result<Px, OutOfIndex> {
        self.accumulator[i].as_ref().ok_or(OutOfIndex)?(v)
    }

    fn unit_to_px(&self, i: usize, unit: &Unit) -> Result<Px, OutOfIndex> {
        match unit {
            Unit::Px(p)        => Ok(*p),
            Unit::Unit(v)      => self.eval(i, *v),
            Unit::Parameter(_) => Err(OutOfIndex),
        }
    }

    pub fn point_as_px(&self, points: &Vec<Point<D>>) -> Vec<[Px; D]> {
        points.iter().map(|pt| {
            from_fn(|i| self.unit_to_px(i, &pt[i]).unwrap_or(0.0))
        }).collect()
    }

    pub fn box_as_px(&self, boxes: &Vec<BBox<D>>) -> Vec<([Px; D], [Px; D])> {
        boxes.iter().map(|(base, offset)| {
            let base_px   = from_fn(|i| self.unit_to_px(i, &base[i]).unwrap_or(0.0));
            let offset_px = from_fn(|i| self.unit_to_px(i, &offset[i]).unwrap_or(0.0));
            (base_px, offset_px)
        }).collect()
    }

    /// 単一のboxの各辺長を1とした、符号付き局所座標(ratio)
    /// point/戻り値ともPxバリアント/Parameterバリアントであることは呼び出し規約とし、型上はf64配列。
    pub fn get_ratio(&self, point: [f64; D], bx: BBox<D>) -> [Parameter; D] {
        let (base, offset) = bx;
        from_fn(|i| {
            let base_px   = self.unit_to_px(i, &base[i]).unwrap_or(0.0);
            let offset_px = self.unit_to_px(i, &offset[i]).unwrap_or(1.0);
            if offset_px == 0.0 { 0.0 } else { (point[i] - base_px) / offset_px }
        })
    }

    pub fn as_px(&self, boxes: &Vec<BBox<D>>) -> Vec<Result<([Px; D], [Px; D]), OutOfIndex>> {
        boxes.iter().map(|(base, offset)| -> Result<([Px; D], [Px; D]), OutOfIndex> {
            let mut base_px   = [0.0f64; D];
            let mut offset_px = [0.0f64; D];
            for i in 0..D {
                base_px[i]   = self.unit_to_px(i, &base[i])?;
                offset_px[i] = self.unit_to_px(i, &offset[i])?;
            }
            Ok((base_px, offset_px))
        }).collect()
    }
}
