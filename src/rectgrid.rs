// This file includes untranslated text (ja).

use core::{primitive::{u32, usize, f64}, result::Result, array::from_fn, marker::PhantomData, ops::{Add, AddAssign, Sub, Mul, Div}};
use alloc::{vec::Vec, boxed::Box, rc::Rc};

use crate::RectgridError;

// 前提: RectGrid自体は各次元D間のPxの関係性を扱わないが、幾何実装の上では、D間で各Pxは等しく出力される。

// When treating the rectgrid module of x and y as 2D coordinates,
// x represents the axis that becomes the width in the viewport.
// y represents the height direction in the viewport.
// The origin (0,0) is assumed to be the top-left corner.

// px座標系の仕様: モジュール外部から関数引数として渡されるpx(point/pointer)はglobal(origin未補正の外部座標、
// 例えばviewport座標)として受け取り、各関数の内部でoriginを差し引いてlocal化する。
// 一方、box(base/offset)由来のpx(unit_to_pxの戻り値や、それを使うhit_test系・as_px系の戻り値)は
// 常にlocal(origin=0を基準とした座標)を返す。呼び出し側がRectGridを跨いで再度渡す場合はlocal pxとして扱う。

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

#[derive(Clone, Copy)]
pub struct BBox<const D: usize> {
    pub base:   Point<D>,
    pub offset: Point<D>,
}

impl<const D: usize> BBox<D> {
    /// base/offsetをfloorして整数格子にスナップする。
    /// extendはbaseにのみ適用し、floor前に加算する。
    /// (例: extend=-0.5 → 0.5以上食い込んでいれば繰り上げ、未満なら切り捨て)
    ///
    /// ```
    /// use rectgrid::{BBox, Unit};
    /// let mut bx = BBox {
    ///     base:   [Unit::new(2.6), Unit::new(0.6)],
    ///     offset: [Unit::new(1.9), Unit::new(3.0)],
    /// };
    /// bx.snap_floor(Some([Unit::new(-0.5), Unit::new(-0.5)]));
    /// // base: 2.6 + (-0.5) = 2.1 → floor = 2.0
    /// assert_eq!(bx.base[0].get(), 2.0);
    /// // base: 0.6 + (-0.5) = 0.1 → floor = 0.0
    /// assert_eq!(bx.base[1].get(), 0.0);
    /// // offset: extendなし、そのままfloor
    /// assert_eq!(bx.offset[0].get(), 1.0);
    /// assert_eq!(bx.offset[1].get(), 3.0);
    /// ```
    pub fn snap_floor(&mut self, extend: Option<[Unit; D]>) -> &mut Self {
        for (d, u) in self.base.iter_mut().enumerate() {
            let v = if let Some(ext) = extend { *u + ext[d] } else { *u };
            *u = Unit::new(libm::floor(v.get()));
        }
        for u in self.offset.iter_mut() {
            *u = Unit::new(libm::floor(u.get()));
        }
        self
    }

    /// offsetの全軸が非ゼロか(=幾何的な面積/体積を持つBBoxか)を返す。
    /// いずれかの軸が0の場合、線分や点として面積を持たないとみなしfalseを返す。
    ///
    /// ```
    /// use rectgrid::{BBox, Unit};
    /// let area = BBox { base: [Unit::new(0.0); 2], offset: [Unit::new(1.0), Unit::new(3.0)] };
    /// assert!(area.has_size());
    /// // x軸のoffsetが0(線分) → 面積なし
    /// let segment = BBox { base: [Unit::new(0.0); 2], offset: [Unit::new(0.0), Unit::new(3.0)] };
    /// assert!(!segment.has_size());
    /// let point = BBox { base: [Unit::new(0.0); 2], offset: [Unit::new(0.0); 2] };
    /// assert!(!point.has_size());
    /// ```
    pub fn has_size(&self) -> bool {
        self.offset.iter().all(|u| u.get() != 0.0)
    }
}

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
    ///
    /// ```
    /// extern crate alloc;
    /// use rectgrid::{IncrementFunction, Px};
    ///
    /// // Scale: 端数もそのまま線形倍
    /// let f = IncrementFunction::Scale(10.0).accumulate().unwrap();
    /// assert_eq!(f(2.5).unwrap().get(), 25.0);
    ///
    /// // VectorList: 整数indexは列挙値そのもの、端数は隣接値との線形補間
    /// let f = IncrementFunction::VectorList(alloc::vec![Px::new(0.0), Px::new(10.0), Px::new(30.0)]).accumulate().unwrap();
    /// assert_eq!(f(1.0).unwrap().get(), 10.0);
    /// assert_eq!(f(1.5).unwrap().get(), 20.0);
    ///
    /// // 空のVectorListは評価不能
    /// use rectgrid::RectgridError;
    /// assert!(matches!(IncrementFunction::VectorList(alloc::vec![]).accumulate(), Err(RectgridError::InvalidDefinition)));
    /// ```
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

    /// d軸の定義を差し替える。以降のunit_to_px/point_to_unit等はこの新しい定義で評価される。
    ///
    /// ```
    /// use rectgrid::{RectGrid, IncrementFunction, Px, Unit};
    /// let mut grid = RectGrid::<1>::new([Px::new(0.0)], [IncrementFunction::Scale(10.0)]).unwrap();
    /// assert_eq!(grid.unit_to_px(0, &Unit::new(2.0)).unwrap().get(), 20.0);
    /// grid.set_definition(IncrementFunction::Scale(100.0), 0).unwrap();
    /// assert_eq!(grid.unit_to_px(0, &Unit::new(2.0)).unwrap().get(), 200.0);
    /// ```
    pub fn set_definition(&mut self, definition: IncrementFunction, d: usize) -> Result<(), RectgridError> {
        self.accumulator[d] = definition.accumulate()?;
        Ok(())
    }

    /// pxをunitへ数値的に逆変換する(accumulatorはUnit→Pxの一方向クロージャしか持たないため)。
    /// pointはviewport等の外部px座標のまま渡してよい。originを差し引いたローカル座標に補正してから変換する。
    /// 呼び出し側の契約: 各軸のaccumulatorはUnit>=0の範囲で単調非減少であること。
    /// 単調非減少でない場合(Scaleへ負の値を与えた場合やForwardDifferenceが減少する差分を返す場合)は結果を保証しない。
    ///
    /// ```
    /// extern crate alloc;
    /// use rectgrid::{RectGrid, IncrementFunction, Px};
    /// use rectgrid::RectgridError;
    /// let grid = RectGrid::<1>::new(
    ///     [Px::new(0.0)],
    ///     [IncrementFunction::VectorList(alloc::vec![Px::new(0.0), Px::new(10.0), Px::new(30.0)])],
    /// ).unwrap();
    /// // 定義域(0..=30px)を超えるpxは、最後の有効indexを添えたOutOfIndexを返す
    /// let result = grid.point_to_unit([Px::new(100.0)]);
    /// assert!(matches!(result[0], Err(RectgridError::OutOfIndex(2))));
    /// ```
    pub fn point_to_unit(&self, point: [Px; D]) -> [Result<Unit, RectgridError>; D] {
        from_fn(|i| self.px_to_unit_axis(i, point[i] - self.origin[i]))
    }

    fn px_to_unit_axis(&self, i: usize, target: Px) -> Result<Unit, RectgridError> {
        let target = target.get();
        let f = |x: f64| self.accumulator[i](x);

        let mut lo = 0.0;
        let mut hi = 1.0;
        loop {
            match f(hi) {
                Ok(px) if px.get() >= target => break,
                Ok(_) => {
                    lo = hi;
                    hi *= 2.0;
                }
                // 定義域の終端に達した。targetがそこまでの範囲内で到達可能か確認する。
                Err(RectgridError::OutOfIndex(last)) => {
                    hi = last as f64;
                    if f(hi)?.get() < target {
                        return Err(RectgridError::OutOfIndex(last));
                    }
                    break;
                }
                Err(e) => return Err(e),
            }
        }

        const EPSILON: f64 = 1e-9;
        for _ in 0..64 {
            if hi - lo < EPSILON {
                break;
            }
            let mid = (lo + hi) / 2.0;
            if f(mid)?.get() < target {
                lo = mid;
            } else {
                hi = mid;
            }
        }
        Ok(Unit::new((lo + hi) / 2.0))
    }

    /// unit座標をpxへ変換する(accumulatorをそのまま評価する)。unitは原点からの絶対値であること。
    ///
    /// ```
    /// use rectgrid::{RectGrid, IncrementFunction, Px, Unit};
    /// let grid = RectGrid::<1>::new([Px::new(0.0)], [IncrementFunction::Scale(200.0)]).unwrap();
    /// assert_eq!(grid.unit_to_px(0, &Unit::new(2.25)).unwrap().get(), 450.0);
    /// ```
    pub fn unit_to_px(&self, d: usize, unit: &Unit) -> Result<Px, RectgridError> {
        self.accumulator[d](unit.get())
    }

    /// 複数のunit座標点をpxへ変換する。評価不能な軸がある点はErrを返す(点単位で判定を打ち切る、他の点には影響しない)。
    ///
    /// ```
    /// extern crate alloc;
    /// use rectgrid::{RectGrid, IncrementFunction, Px, Unit};
    /// use rectgrid::RectgridError;
    /// let grid = RectGrid::<1>::new(
    ///     [Px::new(0.0)],
    ///     [IncrementFunction::VectorList(alloc::vec![Px::new(0.0), Px::new(10.0)])],
    /// ).unwrap();
    /// let points = alloc::vec![[Unit::new(0.5)], [Unit::new(5.0)]];
    /// let px = grid.point_as_px(&points);
    /// assert_eq!(px[0].as_ref().unwrap()[0].get(), 5.0);
    /// // 2点目はVectorListの定義域(0..=1)を超えるためOutOfIndex
    /// assert!(matches!(px[1], Err(RectgridError::OutOfIndex(1))));
    /// ```
    pub fn point_as_px(&self, points: &Vec<Point<D>>) -> Vec<Result<[Px; D], RectgridError>> {
        points.iter().map(|pt| -> Result<[Px; D], RectgridError> {
            let mut px = [Px::new(0.0); D];
            for d in 0..D {
                px[d] = self.unit_to_px(d, &pt[d])?;
            }
            Ok(px)
        }).collect()
    }

    /// pointがboxes[i]に含まれるか(extend込み)を軸ごとに判定する。
    /// pointはviewport座標のまま渡してよい(内部でoriginを差し引く)。
    /// extendはunit座標のままbase/offsetに加算してからpx変換する
    /// (accumulatorが非線形な場合、pxを個別に変換してから加算すると境界の位置によって幅がずれるため)。
    /// 戻り値: (hitしたか, extend抜きのbase_px, extend抜きのoffset_px)。
    /// base_px/offset_pxはhit後のratio計算にそのまま使い回せるよう、判定ついでに返す。
    fn contains(&self, point: [Px; D], bx: &BBox<D>, extend: Option<([Unit; D], [Unit; D])>) -> (bool, [Px; D], [Px; D]) {
        let local: [Px; D] = from_fn(|d| point[d] - self.origin[d]);
        let base_px:   [Px; D] = from_fn(|d| self.unit_to_px(d, &bx.base[d]).unwrap_or(Px::new(0.0)));
        let offset_px: [Px; D] = from_fn(|d| self.unit_to_px(d, &(bx.base[d] + bx.offset[d])).unwrap_or(Px::new(0.0)));
        let (lo, hi): ([Px; D], [Px; D]) = if let Some((eb, eo)) = extend {
            (from_fn(|d| self.unit_to_px(d, &(bx.base[d] + eb[d])).unwrap_or(Px::new(0.0))),
             from_fn(|d| self.unit_to_px(d, &(bx.base[d] + bx.offset[d] + eo[d])).unwrap_or(Px::new(0.0))))
        } else {
            (base_px, offset_px)
        };
        let hit = (0..D).all(|d| local[d].get() >= lo[d].get() && local[d].get() <= hi[d].get());
        (hit, base_px, offset_px)
    }

    /// `ξ_d = (point_d − base_d) / offset_d`
    /// 単一のboxの各辺長(offset)を1とした、符号付き局所座標(ratio)。
    /// base_px/offset_pxはunit座標のbase/base+offsetをpx変換した値(containsの戻り値と同じ形)。
    fn ratio_from_px(point: [Px; D], base_px: [Px; D], offset_px: [Px; D]) -> [Parameter; D] {
        from_fn(|d| {
            let width = offset_px[d] - base_px[d];
            if width.get() == 0.0 { Parameter::new(0.0) } else { Parameter::new((point[d] - base_px[d]) / width) }
        })
    }

    /// pointにhitするboxesのうち、indexが最大のものを返す(boxesはindexが大きいほど優先度が高いとみなす)。
    /// 複数hitする場合はindexの大きい方を優先するため、末尾から走査する。
    ///
    /// ```
    /// extern crate alloc;
    /// use rectgrid::{RectGrid, IncrementFunction, BBox, Px, Unit};
    /// let grid = RectGrid::<2>::new(
    ///     [Px::new(0.0), Px::new(0.0)],
    ///     [IncrementFunction::Scale(200.0), IncrementFunction::Scale(64.0)],
    /// ).unwrap();
    /// let boxes = alloc::vec![
    ///     BBox { base: [Unit::new(0.0), Unit::new(0.0)], offset: [Unit::new(1.0), Unit::new(1.0)] },
    ///     BBox { base: [Unit::new(2.0), Unit::new(0.0)], offset: [Unit::new(1.0), Unit::new(1.0)] },
    /// ];
    /// // (450, 10)pxはboxes[1](x: 400..600px)の内側
    /// assert_eq!(grid.hit_test([Px::new(450.0), Px::new(10.0)], &boxes, None), Some(1));
    /// // どのboxにもhitしない座標
    /// assert_eq!(grid.hit_test([Px::new(-100.0), Px::new(-100.0)], &boxes, None), None);
    /// ```
    pub fn hit_test(&self, point: [Px; D], boxes: &Vec<BBox<D>>, extend: Option<([Unit; D], [Unit; D])>) -> Option<usize> {
        boxes.iter()
            .enumerate()
            .rev()
            .find_map(|(i, bx)| self.contains(point, bx, extend).0.then_some(i))
    }

    /// hit_testと同様にindex最大のhitを返しつつ、hitしたboxに対するget_ratio相当の値も併せて返す。
    /// ratioはextendの影響を受けない、box内側基準の比率(base側=0.0, offset側=1.0)。
    ///
    /// ```
    /// extern crate alloc;
    /// use rectgrid::{RectGrid, IncrementFunction, BBox, Px, Unit};
    /// let grid = RectGrid::<2>::new(
    ///     [Px::new(0.0), Px::new(0.0)],
    ///     [IncrementFunction::Scale(200.0), IncrementFunction::Scale(64.0)],
    /// ).unwrap();
    /// let boxes = alloc::vec![BBox { base: [Unit::new(0.0), Unit::new(0.0)], offset: [Unit::new(1.0), Unit::new(1.0)] }];
    /// // box中央(100, 32)pxはratio(0.5, 0.5)
    /// let (i, ratio) = grid.hit_test_with_ratio([Px::new(100.0), Px::new(32.0)], &boxes, None).unwrap();
    /// assert_eq!(i, 0);
    /// assert!((ratio[0].get() - 0.5).abs() < 1e-9);
    /// assert!((ratio[1].get() - 0.5).abs() < 1e-9);
    /// ```
    pub fn hit_test_with_ratio(&self, point: [Px; D], boxes: &Vec<BBox<D>>, extend: Option<([Unit; D], [Unit; D])>) -> Option<(usize, [Parameter; D])> {
        let local: [Px; D] = from_fn(|d| point[d] - self.origin[d]);
        boxes.iter()
            .enumerate()
            .rev()
            .find_map(|(i, bx)| {
                let (hit, base_px, offset_px) = self.contains(point, bx, extend);
                hit.then(|| (i, Self::ratio_from_px(local, base_px, offset_px)))
            })
    }

    /// pointがhitするboxを全て走査し、boxesと同じ長さのhit有無を返す。
    ///
    /// ```
    /// extern crate alloc;
    /// use rectgrid::{RectGrid, IncrementFunction, BBox, Px, Unit};
    /// let grid = RectGrid::<2>::new(
    ///     [Px::new(0.0), Px::new(0.0)],
    ///     [IncrementFunction::Scale(200.0), IncrementFunction::Scale(64.0)],
    /// ).unwrap();
    /// // 2軸とも重なるboxes[0](x: 0..200px, y: 0..64px)とboxes[1](x: 100..300px, y: 0..64px)
    /// let boxes = alloc::vec![
    ///     BBox { base: [Unit::new(0.0), Unit::new(0.0)], offset: [Unit::new(1.0), Unit::new(1.0)] },
    ///     BBox { base: [Unit::new(0.5), Unit::new(0.0)], offset: [Unit::new(1.0), Unit::new(1.0)] },
    /// ];
    /// // (150, 10)pxは両方のboxにhitする
    /// assert_eq!(grid.hit_tests([Px::new(150.0), Px::new(10.0)], &boxes, None), alloc::vec![true, true]);
    /// ```
    pub fn hit_tests(&self, point: [Px; D], boxes: &Vec<BBox<D>>, extend: Option<([Unit; D], [Unit; D])>) -> Vec<bool> {
        boxes.iter()
            .map(|bx| self.contains(point, bx, extend).0)
            .collect()
    }

    /// `ξ_d = (point_d − base_d) / offset_d`
    /// 単一のboxの各辺長(offset)を1とした、符号付き局所座標(ratio)。
    /// pointはviewport等の外部px座標のまま渡してよい(内部でoriginを差し引く)。
    ///
    /// ```
    /// use rectgrid::{RectGrid, IncrementFunction, BBox, Px, Unit};
    /// let grid = RectGrid::<2>::new(
    ///     [Px::new(0.0), Px::new(0.0)],
    ///     [IncrementFunction::Scale(200.0), IncrementFunction::Scale(64.0)],
    /// ).unwrap();
    /// let bx = BBox { base: [Unit::new(1.0), Unit::new(0.0)], offset: [Unit::new(1.0), Unit::new(1.0)] };
    /// // boxはx: 200..400px, y: 0..64px。(300, 16)pxは各軸の1/4地点
    /// let ratio = grid.get_ratio([Px::new(300.0), Px::new(16.0)], bx);
    /// assert!((ratio[0].get() - 0.5).abs() < 1e-9);
    /// assert!((ratio[1].get() - 0.25).abs() < 1e-9);
    /// // box範囲外(base側)はratioが負になる
    /// let ratio = grid.get_ratio([Px::new(100.0), Px::new(0.0)], bx);
    /// assert!((ratio[0].get() - (-0.5)).abs() < 1e-9);
    /// ```
    pub fn get_ratio(&self, point: [Px; D], bx: BBox<D>) -> [Parameter; D] {
        let local: [Px; D] = from_fn(|d| point[d] - self.origin[d]);
        let base_px   = from_fn(|d| self.unit_to_px(d, &bx.base[d]).unwrap_or(Px::new(0.0)));
        let offset_px = from_fn(|d| self.unit_to_px(d, &(bx.base[d] + bx.offset[d])).unwrap_or(Px::new(1.0)));
        Self::ratio_from_px(local, base_px, offset_px)
    }

    /// 複数のBBoxを(base_px, offset_px)へ変換する。offset_pxはbase位置を踏まえた実際の辺の長さ
    /// (unit_to_px(base+offset) - unit_to_px(base))であり、非線形なaccumulator(ForwardDifference/VectorList)でも
    /// base位置に応じた正しい長さになる。評価不能な軸があるboxはErrを返す(box単位で判定を打ち切る、他のboxには影響しない)。
    ///
    /// ```
    /// extern crate alloc;
    /// use rectgrid::{RectGrid, IncrementFunction, BBox, Px, Unit};
    /// use rectgrid::RectgridError;
    /// let grid = RectGrid::<1>::new(
    ///     [Px::new(0.0)],
    ///     [IncrementFunction::VectorList(alloc::vec![Px::new(0.0), Px::new(10.0)])],
    /// ).unwrap();
    /// let boxes = alloc::vec![BBox { base: [Unit::new(0.0)], offset: [Unit::new(5.0)] }];
    /// // offset=5はVectorListの定義域(0..=1)を超えるためOutOfIndex
    /// assert!(matches!(grid.box_as_px(&boxes)[0], Err(RectgridError::OutOfIndex(1))));
    /// ```
    pub fn box_as_px(&self, boxes: &Vec<BBox<D>>) -> Vec<Result<([Px; D], [Px; D]), RectgridError>> {
        boxes.iter().map(|bx| -> Result<([Px; D], [Px; D]), RectgridError> {
            let mut base_px   = [Px::new(0.0); D];
            let mut offset_px = [Px::new(0.0); D];
            for d in 0..D {
                base_px[d]   = self.unit_to_px(d, &bx.base[d])?;
                offset_px[d] = self.unit_to_px(d, &(bx.base[d] + bx.offset[d]))? - base_px[d];
            }
            Ok((base_px, offset_px))
        }).collect()
    }

    /// pointerのlocal座標(origin補正後)からzを差し引いた値を返す。
    /// pointerはviewport等の外部px座標のまま渡してよい(内部でoriginを差し引く)。
    /// drag開始時はzにbase_px(要素基準位置)を渡すとドラッグオフセットが、
    /// drag中はzにそのオフセットを渡すと現在の要素基準位置が求まる。
    ///
    /// ```
    /// use rectgrid::{RectGrid, IncrementFunction, Px};
    /// let grid = RectGrid::<2>::new(
    ///     [Px::new(10.0), Px::new(20.0)],
    ///     [IncrementFunction::Scale(200.0), IncrementFunction::Scale(64.0)],
    /// ).unwrap();
    /// // origin(10, 20)を補正した(220, 30)pxを掴んだ位置から、要素基準px(200, 0)を引くとドラッグオフセット
    /// let drag_offset = grid.offset([Px::new(230.0), Px::new(50.0)], [Px::new(200.0), Px::new(0.0)]);
    /// assert_eq!((drag_offset[0].get(), drag_offset[1].get()), (20.0, 30.0));
    /// ```
    pub fn offset(&self, pointer: [Px; D], z: [Px; D]) -> [Px; D] {
        from_fn(|d| (pointer[d] - self.origin[d]) - z[d])
    }
}

// ============================================================
// pointer / drag interaction (stateless helpers)
//
// RectGrid<D>とBBox<D>のみに依存する、状態を持たない純粋関数群。
// いずれも&RectGridのみ要求する(RectGridのaccumulatorはnew()時点で構築済みで、
// 呼び出しごとに内部状態を変更する必要がないため)。
// BBoxはbase/offsetが常にUnitで、Px/Unitが混在する中間状態を表現できないため、
// drag中のpx位置はBBoxを更新せず戻り値としてのみ返す。呼び出し側がdrag中はpxを保持し、
// DragEnd相当のタイミングでsnap_*関数に通してBBox(Unit)へ確定させる。
// ============================================================

/// 面積を持つBBoxに対し、pointが辺付近(閾値threshold未満)にあるかを軸ごとに判定する。
/// 戻り値は(各軸のratio(取得できた場合), 角判定結果)のペア。
/// 角判定結果の各要素: Some(true)=base側の辺付近([0, threshold]), Some(false)=offset側の辺付近([1-threshold, 1]), None=非該当。
/// 全軸がSomeの場合のみ角ハンドルとして扱う(呼び出し側でNoneを許容するかは呼び出し側の判断)。
/// ratioがNoneになるのは、has_sizeでないか、pointがbx範囲外の場合。
///
/// ```
/// use rectgrid::{RectGrid, IncrementFunction, BBox, Px, Unit, corner_test};
/// let grid = RectGrid::<2>::new(
///     [Px::new(0.0), Px::new(0.0)],
///     [IncrementFunction::Scale(200.0), IncrementFunction::Scale(64.0)],
/// ).unwrap();
/// let bx = BBox { base: [Unit::new(2.0), Unit::new(0.0)], offset: [Unit::new(1.0), Unit::new(3.0)] };
/// // bx左上隅(400, 0)px付近をクリック → base側同士の角
/// let (_, corner) = corner_test(&grid, [Px::new(400.0), Px::new(0.0)], &bx, 0.1);
/// assert_eq!(corner, Some([Some(true), Some(true)]));
/// // bx中央付近は角に該当しないが、ratio自体は取得できる
/// let (ratio, corner) = corner_test(&grid, [Px::new(500.0), Px::new(96.0)], &bx, 0.1);
/// assert!(ratio.is_some());
/// assert_eq!(corner, None);
/// // bx範囲外はratioも取得できない
/// let (ratio, corner) = corner_test(&grid, [Px::new(400.0), Px::new(-10.0)], &bx, 0.1);
/// assert!(ratio.is_none());
/// assert_eq!(corner, None);
/// ```
pub fn corner_test<const D: usize>(
    grid:      &RectGrid<D>,
    point:     [Px; D],
    bx:        &BBox<D>,
    threshold: f64,
) -> (Option<[Parameter; D]>, Option<[Option<bool>; D]>) {
    if !bx.has_size() { return (None, None); }
    let ratio = grid.get_ratio(point, *bx);
    let inside = ratio.iter().all(|r| r.get() >= 0.0 && r.get() <= 1.0);
    if !inside { return (None, None); }
    let corner: [Option<bool>; D] = from_fn(|d| {
        let r = ratio[d].get();
        if r <= threshold { Some(true) }
        else if r >= 1.0 - threshold { Some(false) }
        else { None }
    });
    let corner = if corner.iter().all(Option::is_some) { Some(corner) } else { None };
    (Some(ratio), corner)
}

/// Drag中、角ハンドルドラッグによってBBoxのbase/offsetを更新し、更新後のBBoxを返す。
/// corner[d] = Some(base_side): base_side==trueならbase側の辺を、falseならoffset側の辺を動かす。
/// 新しいoffsetは最小1.0unitを保証する(base/offsetが交差しないように)。
///
/// ```
/// use rectgrid::{RectGrid, IncrementFunction, BBox, Px, Unit, drag_resize};
/// let grid = RectGrid::<2>::new(
///     [Px::new(0.0), Px::new(0.0)],
///     [IncrementFunction::Scale(200.0), IncrementFunction::Scale(64.0)],
/// ).unwrap();
/// let bx = BBox { base: [Unit::new(2.0), Unit::new(0.0)], offset: [Unit::new(1.0), Unit::new(3.0)] };
/// // 左上(base側)の角を(150, 0)pxへドラッグ → x軸のbaseが0unitへ縮む
/// let resized = drag_resize(&grid, [Px::new(150.0), Px::new(0.0)], &bx, [Some(true), None]).unwrap();
/// assert_eq!(resized.base[0].get(), 0.0);
/// assert_eq!(resized.offset[0].get(), 3.0); // base 2.0 + offset 1.0 - new_base 0.0
/// assert_eq!(resized.offset[1].get(), 3.0); // y軸は未変更
/// ```
pub fn drag_resize<const D: usize>(
    grid:    &RectGrid<D>,
    pointer: [Px; D],
    bx:      &BBox<D>,
    corner:  [Option<bool>; D],
) -> Result<BBox<D>, RectgridError> {
    let unit = grid.point_to_unit(pointer);
    let mut resized = *bx;
    for d in 0..D {
        let Some(base_side) = corner[d] else { continue };
        let new_u = Unit::new(libm::floor(unit[d]?.get()));
        let base_u   = bx.base[d];
        let offset_u = bx.offset[d];
        if base_side {
            let new_offset = ((base_u + offset_u) - new_u).get().max(1.0);
            resized.base[d]   = new_u;
            resized.offset[d] = Unit::new(new_offset);
        } else {
            let new_offset = (new_u - base_u).get().max(1.0);
            resized.offset[d] = Unit::new(new_offset);
        }
    }
    Ok(resized)
}

/// Drag中、移動ドラッグ(角ハンドルでない)によってbaseのpx位置を求める。
/// pointerはviewport等の外部px座標のまま渡してよい(内部でoriginを差し引く)。
/// BBoxはbaseが常にUnitのため、drag中のpx位置はBBoxを更新せずここで返すのみに留め、
/// DragEnd相当のタイミングでsnap_region_to_unitに通してBBoxへ反映する。
///
/// ```
/// use rectgrid::{RectGrid, IncrementFunction, Px, drag_translate};
/// let grid = RectGrid::<2>::new(
///     [Px::new(10.0), Px::new(20.0)],
///     [IncrementFunction::Scale(200.0), IncrementFunction::Scale(64.0)],
/// ).unwrap();
/// // pointerをviewport座標(230, 50)、drag_offsetを(20, 30)として移動
/// let px = drag_translate(&grid, [Px::new(230.0), Px::new(50.0)], [Px::new(20.0), Px::new(30.0)]);
/// assert_eq!((px[0].get(), px[1].get()), (200.0, 0.0));
/// ```
pub fn drag_translate<const D: usize>(
    grid:        &RectGrid<D>,
    pointer:     [Px; D],
    drag_offset: [Px; D],
) -> [Px; D] {
    grid.offset(pointer, drag_offset)
}

/// DragEnd時、面積を持つBBoxの移動ドラッグ結果をUnit格子にスナップし、更新後のBBox(base)を返す。
/// pointerはviewport等の外部px座標のまま渡してよい(内部でoriginを差し引く)。drag_offsetはdrag_translateに渡したものと同じ値。
/// extendはunit変換値にfloor前に加算する。
///
/// ```
/// use rectgrid::{RectGrid, IncrementFunction, BBox, Px, Unit, snap_region_to_unit};
/// let grid = RectGrid::<2>::new(
///     [Px::new(0.0), Px::new(0.0)],
///     [IncrementFunction::Scale(200.0), IncrementFunction::Scale(64.0)],
/// ).unwrap();
/// let bx = BBox { base: [Unit::new(0.0), Unit::new(0.0)], offset: [Unit::new(1.0), Unit::new(1.0)] };
/// // pointer(430, 70)px、drag_offset(0, 0) → local(430, 70)px = (2.15, 1.09375)unit。extend +0.25してfloor。
/// let snapped = snap_region_to_unit(
///     &grid, [Px::new(430.0), Px::new(70.0)], [Px::new(0.0), Px::new(0.0)], &bx,
///     Some([Unit::new(0.25), Unit::new(0.25)]),
/// ).unwrap();
/// assert_eq!(snapped.base[0].get(), 2.0);
/// assert_eq!(snapped.base[1].get(), 1.0);
/// assert_eq!(snapped.offset[0].get(), 1.0); // offsetはfloor済みでそのまま
/// ```
pub fn snap_region_to_unit<const D: usize>(
    grid:        &RectGrid<D>,
    pointer:     [Px; D],
    drag_offset: [Px; D],
    bx:          &BBox<D>,
    extend:      Option<[Unit; D]>,
) -> Result<BBox<D>, RectgridError> {
    let mut snapped = *bx;
    for d in 0..D {
        let local = pointer[d] - grid.origin[d] - drag_offset[d];
        snapped.base[d] = grid.px_to_unit_axis(d, local)?;
    }
    snapped.snap_floor(extend);
    Ok(snapped)
}

/// DragEnd時、点BBox(面積なし)の移動ドラッグ結果をUnit格子にスナップしたBBoxを求める。
/// pointerはviewport等の外部px座標のまま渡してよい(内部でoriginを差し引く)。drag_offsetはdrag_translateに渡したものと同じ値。
///
/// ```
/// use rectgrid::{RectGrid, IncrementFunction, Px, Unit, snap_point_to_unit};
/// let grid = RectGrid::<2>::new(
///     [Px::new(0.0), Px::new(0.0)],
///     [IncrementFunction::Scale(200.0), IncrementFunction::Scale(64.0)],
/// ).unwrap();
/// // pointer(430, 70)px、drag_offset(0, 0) → (2.15, 1.09375)unit。snap +0.25してfloor。
/// let snapped = snap_point_to_unit(
///     &grid, [Px::new(430.0), Px::new(70.0)], [Px::new(0.0), Px::new(0.0)], [Unit::new(0.25), Unit::new(0.25)],
/// ).unwrap();
/// assert_eq!(snapped.base[0].get(), 2.0);
/// assert_eq!(snapped.base[1].get(), 1.0);
/// assert!(!snapped.has_size());
/// ```
pub fn snap_point_to_unit<const D: usize>(
    grid:        &RectGrid<D>,
    pointer:     [Px; D],
    drag_offset: [Px; D],
    snap:        [Unit; D],
) -> Result<BBox<D>, RectgridError> {
    let mut base: Point<D> = [Unit::new(0.0); D];
    for d in 0..D {
        let local = pointer[d] - grid.origin[d] - drag_offset[d];
        let u = grid.px_to_unit_axis(d, local)?;
        base[d] = Unit::new(libm::floor((u + snap[d]).get()));
    }
    Ok(BBox { base, offset: from_fn(|_| Unit::new(0.0)) })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn point_to_unit_scale_roundtrip() {
        let grid = RectGrid::<2>::new(
            [Px::new(0.0), Px::new(0.0)],
            [IncrementFunction::Scale(200.0), IncrementFunction::Scale(64.0)],
        ).unwrap();
        let result = grid.point_to_unit([Px::new(450.0), Px::new(10.0)]);
        let x = result[0].as_ref().unwrap().get();
        let y = result[1].as_ref().unwrap().get();
        assert!((x - 2.25).abs() < 1e-6, "x = {}", x);
        assert!((y - 0.15625).abs() < 1e-6, "y = {}", y);
    }

    #[test]
    fn point_to_unit_vector_list_roundtrip() {
        let grid = RectGrid::<1>::new(
            [Px::new(0.0)],
            [IncrementFunction::VectorList(alloc::vec![Px::new(0.0), Px::new(10.0), Px::new(30.0), Px::new(60.0)])],
        ).unwrap();
        let result = grid.point_to_unit([Px::new(45.0)]);
        let x = result[0].as_ref().unwrap().get();
        assert!((x - 2.5).abs() < 1e-6, "x = {}", x);
    }

    #[test]
    fn point_to_unit_vector_list_out_of_range() {
        let grid = RectGrid::<1>::new(
            [Px::new(0.0)],
            [IncrementFunction::VectorList(alloc::vec![Px::new(0.0), Px::new(10.0), Px::new(30.0), Px::new(60.0)])],
        ).unwrap();
        let result = grid.point_to_unit([Px::new(100.0)]);
        assert!(matches!(result[0], Err(RectgridError::OutOfIndex(3))));
    }

    #[test]
    fn point_to_unit_applies_origin_offset() {
        let grid = RectGrid::<2>::new(
            [Px::new(10.0), Px::new(20.0)],
            [IncrementFunction::Scale(200.0), IncrementFunction::Scale(64.0)],
        ).unwrap();
        // viewport座標(230, 50)は、origin(10, 20)を差し引くとローカル座標(220, 30)
        let result = grid.point_to_unit([Px::new(230.0), Px::new(50.0)]);
        let x = result[0].as_ref().unwrap().get();
        let y = result[1].as_ref().unwrap().get();
        assert!((x - 1.1).abs() < 1e-6, "x = {}", x);
        assert!((y - 0.46875).abs() < 1e-6, "y = {}", y);
    }
}
