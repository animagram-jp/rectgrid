// This file includes untranslated text (ja).

use core::{primitive::{u32, usize, f64}, result::Result, array::from_fn, marker::PhantomData, ops::{Add, AddAssign, Sub, Mul, Div}};
use alloc::{vec::Vec, boxed::Box, rc::Rc};

use crate::RectgridError;

// 前提: RectGrid自体は各次元D間のPxの関係性を扱わないが、幾何実装の上では、D間で各Pxは等しく出力される。

// When treating the rectgrid module of x and y as 2D coordinates,
// x represents the axis that becomes the width in the viewport.
// y represents the height direction in the viewport.
// The origin (0,0) is assumed to be the top-left corner.

/// 単位系タグ付きのf64値。タグはゼロサイズで実行時表現には影響しない。
pub struct Value<Tag>(f64, PhantomData<Tag>);

impl<Tag> Clone for Value<Tag> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<Tag> Copy for Value<Tag> {}

impl<Tag> Value<Tag> {
    pub fn new(v: f64) -> Self {
        Self(v, PhantomData)
    }

    pub fn get(self) -> f64 {
        self.0
    }
}

impl<Tag> From<f64> for Value<Tag> {
    fn from(v: f64) -> Self {
        Self::new(v)
    }
}

impl<Tag> Add for Value<Tag> {
    type Output = Value<Tag>;
    fn add(self, rhs: Self) -> Self::Output {
        Value::new(self.0 + rhs.0)
    }
}

impl<Tag> AddAssign for Value<Tag> {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

impl<Tag> Sub for Value<Tag> {
    type Output = Value<Tag>;
    fn sub(self, rhs: Self) -> Self::Output {
        Value::new(self.0 - rhs.0)
    }
}

impl<Tag> Mul<f64> for Value<Tag> {
    type Output = Value<Tag>;
    fn mul(self, rhs: f64) -> Self::Output {
        Value::new(self.0 * rhs)
    }
}

impl<Tag> Div for Value<Tag> {
    type Output = f64;
    fn div(self, rhs: Self) -> f64 {
        self.0 / rhs.0
    }
}

pub struct PxTag;
pub struct UnitTag;
pub struct ParameterTag;

/// 原点から正方向へ無限に広がる、直交独立単位系。
pub type Px = Value<PxTag>;

/// 固有原点から正方向への序数を値として各軸で無限または有限に広がる、任意の直交単位系。
pub type Unit = Value<UnitTag>;

/// Bondary Boxの1つに対して、各辺長を1とし、符号をunit座標に従った、無界な局所座標の単位系。
pub type Parameter = Value<ParameterTag>;

pub type Point<const D: usize>  = [Unit; D];
pub type BBox<const D: usize>   = (Point<D>, Point<D>); // (base, offset)

// todo: Option式は、幾何定義実装部
// pub type Region<const D: usize> = (Vec<BBox<D>>, Option<Box<dyn Fn(u32) -> Result<Px, RectgridError>>>);

pub enum IncrementFunction {
    /// Fn(i) = points[i+1] - points[i]
    /// OutOfIndex means boundary; 引数は差分のindex(整数)
    /// クロージャは範囲外の場合、範囲内に収まる最後の有効indexをOutOfIndexに詰めて返すこと。
    /// todo: OutOfIndexの分散分布が許されるか検証
    ForwardDifference(Rc<dyn Fn(u32) -> Result<Px, RectgridError>>),
    /// boundary
    /// 原点から正方向に間隔の与単位値を列挙した配列。
    VectorList(Vec<Px>),
    /// unboundary
    Scale(f64),
}

impl IncrementFunction {
    /// 式から評価クロージャを生成する。Unit座標(f64) -> px座標(originとの相対距離)。
    /// 端数は線形補間でpxに変換する。
    /// VectorListが空など、定義が評価不能な場合はInvalidDefinitionを返す。
    pub fn accumulate(&self) -> Result<Box<dyn Fn(f64) -> Result<Px, RectgridError>>, RectgridError> {
        match self {
            // 差分を0..floor(x)で累積し、端数ぶんは次の差分を線形補間で加算。
            Self::ForwardDifference(f) => {
                let f = f.clone();
                Ok(Box::new(move |x| {
                    let n = libm::floor(x) as u32;
                    let frac = x - n as f64;
                    let mut acc = Px::new(0.0);
                    for k in 0..n {
                        acc += f(k)?;
                    }
                    if frac != 0.0 {
                        acc += f(n)? * frac;
                    }
                    Ok(acc)
                }))
            }
            // 累積座標の配列。整数indexで引き、端数は隣との線形補間。範囲外は境界。
            Self::VectorList(pxs) => {
                if pxs.is_empty() {
                    return Err(RectgridError::InvalidDefinition);
                }
                let pxs = pxs.clone();
                Ok(Box::new(move |x| {
                    let last = (pxs.len() - 1) as u32;
                    let n = libm::floor(x) as usize;
                    let frac = x - n as f64;
                    let lo = *pxs.get(n).ok_or(RectgridError::OutOfIndex(last))?;
                    if frac == 0.0 {
                        return Ok(lo);
                    }
                    let hi = *pxs.get(n + 1).ok_or(RectgridError::OutOfIndex(last))?;
                    Ok(lo + (hi - lo) * frac)
                }))
            }
            Self::Scale(s) => {
                let s = *s;
                Ok(Box::new(move |x| Ok(Px::new(s * x))))
            }
        }
    }
}

pub struct RectGrid<const D: usize> {
    pub origin: [Px; D],
    /// f([Unit; D]) -> f([Px; D])
    accumulator: [Box<dyn Fn(f64) -> Result<Px, RectgridError>>; D],
}

impl<const D: usize> RectGrid<D> {
    pub fn new(origin: [Px; D], definitions: [IncrementFunction; D]) -> Result<Self, RectgridError> {
        let accumulator: Vec<_> = definitions.into_iter()
            .map(|d| d.accumulate())
            .collect::<Result<_, _>>()?;
        let accumulator = match accumulator.try_into() {
            Ok(a) => a,
            Err(_) => unreachable!("definitions and accumulator share length D"),
        };
        Ok(Self { origin, accumulator })
    }

    pub fn set_definition(&mut self, definition: IncrementFunction, d: usize) -> Result<(), RectgridError> {
        self.accumulator[d] = definition.accumulate()?;
        Ok(())
    }

    pub fn point_to_unit(&self, point: [Px; D]) -> [Result<Unit, RectgridError>; D] {
        todo!()
    }

    fn unit_to_px(&self, i: usize, unit: &Unit) -> Result<Px, RectgridError> {
        self.accumulator[i](unit.get())
    }

    pub fn point_as_px(&self, points: &Vec<Point<D>>) -> Vec<[Px; D]> {
        points.iter().map(|pt| {
            from_fn(|i| self.unit_to_px(i, &pt[i]).unwrap_or(Px::new(0.0)))
        }).collect()
    }

    pub fn box_as_px(&self, boxes: &Vec<BBox<D>>) -> Vec<([Px; D], [Px; D])> {
        boxes.iter().map(|(base, offset)| {
            let base_px   = from_fn(|i| self.unit_to_px(i, &base[i]).unwrap_or(Px::new(0.0)));
            let offset_px = from_fn(|i| self.unit_to_px(i, &offset[i]).unwrap_or(Px::new(0.0)));
            (base_px, offset_px)
        }).collect()
    }

    /// `ξ_d = (point_d − base_d) / offset_d`
    /// 単一のboxの各辺長(offset)を1とした、符号付き局所座標(ratio)
    pub fn get_ratio(&self, point: [Px; D], bx: BBox<D>) -> [Parameter; D] {
        let (base, offset) = bx;
        from_fn(|i| {
            let base_px   = self.unit_to_px(i, &base[i]).unwrap_or(Px::new(0.0));
            let offset_px = self.unit_to_px(i, &offset[i]).unwrap_or(Px::new(1.0));
            if offset_px.get() == 0.0 { Parameter::new(0.0) } else { Parameter::new((point[i] - base_px) / offset_px) }
        })
    }

    pub fn as_px(&self, boxes: &Vec<BBox<D>>) -> Vec<Result<([Px; D], [Px; D]), RectgridError>> {
        boxes.iter().map(|(base, offset)| -> Result<([Px; D], [Px; D]), RectgridError> {
            let mut base_px   = [Px::new(0.0); D];
            let mut offset_px = [Px::new(0.0); D];
            for i in 0..D {
                base_px[i]   = self.unit_to_px(i, &base[i])?;
                offset_px[i] = self.unit_to_px(i, &offset[i])?;
            }
            Ok((base_px, offset_px))
        }).collect()
    }
}
