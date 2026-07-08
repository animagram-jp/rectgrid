use core::array::from_fn;
use core::primitive::f64;
use alloc::boxed::Box;
use alloc::rc::Rc;
use alloc::vec::Vec;

// --- rectgrid ---
//
// When treating the rectgrid module of x and y as 2D coordinates,
// x represents the axis that becomes the width in the viewport.
// y represents the height direction in the viewport.
// The origin (0,0) is assumed to be the top-left corner.

type Px   = f64;
type Unit = f64;

#[derive(Clone, Copy)]
pub enum Length {
    Px(Px),
    Unit(Unit),
}
pub type Point<const D: usize> = [Length; D];
#[derive(Clone, Copy)]
pub struct Region<const D: usize> {
    pub base:   Point<D>,
    pub offset: Point<D>,
}
#[derive(Debug)]
pub struct OutOfIndex;

pub enum DefiningExpression {
    // Fn(i) = points[i+1] - points[i]
    ForwardDifference(Rc<dyn Fn(u32) -> Result<Px, OutOfIndex>>), // OutOfIndex can mean boundary; 引数は差分のindex(整数)
    VectorList(Vec<Px>), // boundary
    Scale(f64), // unboundary
}

impl DefiningExpression {
    /// 式から評価クロージャを生成する。x(端数可) -> px座標(originとの相対距離)。
    /// 端数は線形補間でpxに変換する。
    pub fn build(&self) -> Box<dyn Fn(f64) -> Result<Px, OutOfIndex>> {
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

pub struct Rectgrid<const D: usize> {
    pub origin: [Px; D],
    expressions: [DefiningExpression; D],
    cache: [Option<Box<dyn Fn(f64) -> Result<Px, OutOfIndex>>>; D],
}

impl<const D: usize> Rectgrid<D> {
    /// let mut grid = Grid::new(...);
    pub fn new(origin: [Px; D], expressions: [DefiningExpression; D]) -> Self {
        Self {
            origin,
            expressions,
            cache: from_fn(|_| None),
        }
    }

    /// Region(Px/Unit混在)を受け、共通単位pxへ解決した(base, offset)の列を返す。
    /// Unit成分はその軸の式でevalし、Px成分はそのまま使う。
    pub fn update(&mut self, regions: Vec<Region<D>>) -> Result<Vec<([Px; D], [Px; D])>, OutOfIndex> {
        let mut out = Vec::with_capacity(regions.len());
        for region in regions {
            let base   = self.resolve(&region.base)?;
            let offset = self.resolve(&region.offset)?;
            out.push((base, offset));
        }
        Ok(out)
    }

    /// i軸のLengthをpxに変換する。
    pub fn unit_length(&mut self, i: usize, l: &Length) -> Result<Px, OutOfIndex> {
        match l {
            Length::Px(p)   => Ok(*p),
            Length::Unit(u) => self.eval(i, *u),
        }
    }

    /// PointをPx列に変換する。
    pub fn unit_point(&mut self, point: &Point<D>) -> Result<[Px; D], OutOfIndex> {
        let mut px = [0.0; D];
        for i in 0..D {
            px[i] = self.unit_length(i, &point[i])?;
        }
        Ok(px)
    }

    fn resolve(&mut self, point: &Point<D>) -> Result<[Px; D], OutOfIndex> {
        self.unit_point(point)
    }

    /// viewport等の外部px座標をoriginで補正し、グリッド原点基準のローカル座標に変換する。
    /// origin補正はRectgrid内部で完結させ、呼び出し側(event.rs等)に意識させないためのprivateヘルパー。
    fn to_local(&self, coord: [Px; D]) -> [Px; D] {
        from_fn(|i| coord[i] - self.origin[i])
    }

    /// origin(グリッド原点のviewport上位置)を更新する。resize等でsectionの位置が変わった際に呼ぶ。
    pub fn set_origin(&mut self, origin: [Px; D]) {
        self.origin = origin;
    }

    /// viewport座標pointerと、既知のsection基準px位置base_pxから、ドラッグ開始オフセットを求める。
    /// pointerはviewport座標のまま渡してよい(内部でorigin補正する)。
    pub fn drag_offset(&self, pointer: [Px; D], base_px: [Px; D]) -> [Px; D] {
        let local = self.to_local(pointer);
        core::array::from_fn(|i| local[i] - base_px[i])
    }

    /// viewport座標pointerと、drag_offset()で得たオフセットから、現在のsection基準px位置を求める。
    /// pointerはviewport座標のまま渡してよい(内部でorigin補正する)。
    pub fn drag_move(&self, pointer: [Px; D], offset: [Px; D]) -> [Px; D] {
        let local = self.to_local(pointer);
        core::array::from_fn(|i| local[i] - offset[i])
    }

    /// viewport座標pointerのi軸成分を、origin補正した上でunit座標に変換する(端数floorはしない)。
    /// 角ハンドルドラッグ等、ポインタ位置からunit値を直接求めたい場合に使う。
    pub fn axis_unit_from_pointer(&mut self, i: usize, pointer: [Px; D]) -> Result<f64, OutOfIndex> {
        let local = self.to_local(pointer);
        let unit_px = self.eval(i, 1.0)?;
        Ok(local[i] / unit_px)
    }

    /// callerによる式の置換口。該当iのキャッシュのみ破棄し、次のevalで作り直す。
    pub fn set_expression(&mut self, i: usize, def: DefiningExpression) {
        self.expressions[i] = def;
        self.cache[i] = None;
    }

    /// i軸の位置x(端数可)をpx評価する。キャッシュがあれば実行、なければ式から作って保存してから実行。
    pub fn eval(&mut self, i: usize, x: f64) -> Result<Px, OutOfIndex> {
        if self.cache[i].is_none() {
            self.cache[i] = Some(self.expressions[i].build());
        }
        (self.cache[i].as_ref().unwrap())(x)
    }

    /// px座標coordが各Regionに含まれるか判定する。
    /// coordはviewport座標のまま渡してよい(内部でorigin補正する)。
    /// 判定時、extendのUnit値をbase, offsetにそれぞれ加算する。
    /// 各RegionはUnit/Px混在なのでPxへ解決してから内包判定する。
    pub fn judge(&mut self, coord: [Px; D], regions: &[Region<D>], extend: Option<([Unit; D], [Unit; D])>, detail: bool) -> Result<Vec<Option<JudgeResult<D>>>, OutOfIndex> {
        let coord = self.to_local(coord);
        // extendのUnit値をpxへ変換しておく
        let ext: Option<([Px; D], [Px; D])> = if let Some((eb, eo)) = extend {
            let eb_px: [Px; D] = core::array::from_fn(|i| self.eval(i, eb[i]).unwrap_or(0.0));
            let eo_px: [Px; D] = core::array::from_fn(|i| self.eval(i, eo[i]).unwrap_or(0.0));
            Some((eb_px, eo_px))
        } else {
            None
        };
        let mut hits = Vec::with_capacity(regions.len());
        for region in regions {
            let base_px   = self.resolve(&region.base)?;
            let offset_px = self.resolve(&region.offset)?;
            let (lo, hi): ([Px; D], [Px; D]) = if let Some((eb, eo)) = &ext {
                (core::array::from_fn(|i| base_px[i] + eb[i]),
                 core::array::from_fn(|i| base_px[i] + offset_px[i] + eo[i]))
            } else {
                (base_px, core::array::from_fn(|i| base_px[i] + offset_px[i]))
            };
            let inside = (0..D).all(|i| coord[i] >= lo[i] && coord[i] <= hi[i]);
            if !inside {
                hits.push(None);
                continue;
            }
            if detail {
                // 各軸: Regionの辺長に対する割合(base側=0.0, offset側=1.0)。extendの作用は無視する。
                let hi_noext: [Px; D] = core::array::from_fn(|i| base_px[i] + offset_px[i]);
                let ratio = core::array::from_fn(|i| (coord[i] - base_px[i]) / (hi_noext[i] - base_px[i]));
                hits.push(Some(JudgeResult::Ratio(ratio)));
            } else {
                hits.push(Some(JudgeResult::Hit));
            }
        }
        Ok(hits)
    }
}

#[derive(Clone, Copy)]
pub enum JudgeResult<const D: usize> {
    Hit,
    Ratio([f64; D]), // 各軸: Regionの辺長に対する割合(base側=0.0, offset側=1.0)
}

impl<const D: usize> Region<D> {
    /// Unit成分をfloorして整数格子にスナップする。Px成分はそのまま。
    /// Unit成分をfloorして整数格子にスナップする。extendはbaseにのみ適用し、floor前に加算する。
    /// (例: extend=-0.05 → 0.05以上食い込んでいれば繰り上げ、未満なら切り捨て)
    ///
    /// ```
    /// use rectgrid::rectgrid::{Region, Length};
    /// let mut r = Region {
    ///     base:   [Length::Unit(2.6), Length::Unit(0.6)],
    ///     offset: [Length::Unit(1.0), Length::Unit(3.0)],
    /// };
    /// r.snap_floor(Some([-0.5, -0.5]));
    /// // base: 2.6 + (-0.5) = 2.1 → floor = 2.0
    /// assert!(matches!(r.base[0], Length::Unit(v) if v == 2.0));
    /// // base: 0.6 + (-0.5) = 0.1 → floor = 0.0
    /// assert!(matches!(r.base[1], Length::Unit(v) if v == 0.0));
    /// // offset: extendなし、そのままfloor
    /// assert!(matches!(r.offset[0], Length::Unit(v) if v == 1.0));
    /// assert!(matches!(r.offset[1], Length::Unit(v) if v == 3.0));
    /// ```
    pub fn snap_floor(&mut self, extend: Option<[Unit; D]>) -> &mut Self {
        for (i, length) in self.base.iter_mut().enumerate() {
            if let Length::Unit(u) = length {
                *u = if let Some(ext) = extend { (*u + ext[i]).floor() } else { u.floor() };
            }
        }
        for length in self.offset.iter_mut() {
            if let Length::Unit(u) = length {
                *u = u.floor();
            }
        }
        self
    }

    /// offsetのいずれかの軸が非ゼロUnitか(=面積を持つRegionか)を返す。
    /// Px成分は常に「面積あり」とみなす(0.0のPxは意図的な点として扱わないため)。
    ///
    /// ```
    /// use rectgrid::rectgrid::{Region, Length};
    /// let point = Region {
    ///     base:   [Length::Unit(0.0), Length::Unit(0.0)],
    ///     offset: [Length::Unit(0.0), Length::Unit(0.0)],
    /// };
    /// assert!(!point.has_size());
    /// let area = Region {
    ///     base:   [Length::Unit(0.0), Length::Unit(0.0)],
    ///     offset: [Length::Unit(1.0), Length::Unit(3.0)],
    /// };
    /// assert!(area.has_size());
    /// ```
    pub fn has_size(&self) -> bool {
        self.offset.iter().any(|l| !matches!(l, Length::Unit(v) if *v == 0.0))
    }
}

// ============================================================
// pointer / drag interaction (stateless helpers)
//
// crate内の他モジュール(js_client等)に依存せず、Rectgrid<D>とRegion<D>のみで
// 完結するイベント処理ロジック。呼び出し側(event.rs等)はDOM/Command生成に専念する。
// ============================================================

/// PointerDown時、末尾(最前面)から走査してcoordにhitする最初のRegionのindexを返す。
/// extendはjudgeにそのまま渡す判定余白。
///
/// ```
/// use rectgrid::rectgrid::{Rectgrid, DefiningExpression, Region, Length, hit_test};
/// let mut rectgrid = Rectgrid::<2>::new([0.0, 0.0], [DefiningExpression::Scale(200.0), DefiningExpression::Scale(64.0)]);
/// let regions = [
///     Region { base: [Length::Unit(0.0), Length::Unit(0.0)], offset: [Length::Unit(1.0), Length::Unit(1.0)] },
///     Region { base: [Length::Unit(2.0), Length::Unit(0.0)], offset: [Length::Unit(1.0), Length::Unit(1.0)] },
/// ];
/// // (250, 10)pxはregions[1](x: 400..600px)の内側
/// assert_eq!(hit_test(&mut rectgrid, [450.0, 10.0], &regions, None), Some(1));
/// // どのRegionにもhitしない座標
/// assert_eq!(hit_test(&mut rectgrid, [-100.0, -100.0], &regions, None), None);
/// ```
pub fn hit_test<const D: usize>(
    rectgrid:   &mut Rectgrid<D>,
    coord:   [Px; D],
    regions: &[Region<D>],
    extend:  Option<([Unit; D], [Unit; D])>,
) -> Option<usize> {
    rectgrid.judge(coord, regions, extend, false)
        .unwrap_or_default()
        .into_iter()
        .enumerate()
        .rev()
        .find_map(|(i, hit)| hit.map(|_| i))
}

/// 面積を持つRegionに対し、coordが辺付近(閾値threshold未満)にあるかを軸ごとに判定する。
/// 戻り値は(各軸のRatio(取得できた場合), 角判定結果)のペア。
/// 角判定結果の各要素: Some(true)=base側の辺付近([0, threshold]), Some(false)=offset側の辺付近([1-threshold, 1]), None=非該当。
/// 全軸がSomeの場合のみ角ハンドルとして扱う(呼び出し側でNoneを許容するかは呼び出し側の判断)。
/// RatioがNoneになるのは、has_sizeでないか、coordがextend込みのRegion範囲外(inside判定に落ちた)場合。
///
/// judgeのinside判定にはextendをそのまま使う(Region外側の判定余白)が、
/// 辺付近かどうかの判定はJudgeResult::Ratio(extendの影響を受けない、Region内側基準の比率)のみで行う。
/// これにより、extendの大きさがRegionのサイズに対して相対的に小さい軸があっても、
/// 角判定の閾値の意味がRegion間・軸間で常に一定になる。
///
/// ```
/// use rectgrid::rectgrid::{Rectgrid, DefiningExpression, Region, Length, corner_test};
/// let mut rectgrid = Rectgrid::<2>::new([0.0, 0.0], [DefiningExpression::Scale(200.0), DefiningExpression::Scale(64.0)]);
/// let region = Region { base: [Length::Unit(2.0), Length::Unit(0.0)], offset: [Length::Unit(1.0), Length::Unit(3.0)] };
/// // region左上隅(400, 0)px付近をクリック → base側同士の角
/// let (ratio, corner) = corner_test(&mut rectgrid, [400.0, 0.0], &region, None, 0.1);
/// assert_eq!(ratio, Some([0.0, 0.0]));
/// assert_eq!(corner, Some([Some(true), Some(true)]));
/// // region中央付近は角に該当しないが、Ratio自体は取得できる
/// let (ratio, corner) = corner_test(&mut rectgrid, [500.0, 96.0], &region, None, 0.1);
/// assert_eq!(ratio, Some([0.5, 0.5]));
/// assert_eq!(corner, None);
/// // extendの範囲内(0.05unit)だが、y軸のRatio換算では十分辺に近い(1unit中0.05 < threshold 0.1)ので角判定される
/// let extend = Some(([-0.05, -0.05], [0.05, 0.05]));
/// let (ratio, corner) = corner_test(&mut rectgrid, [400.0, -3.0], &region, extend, 0.1);
/// assert_eq!(corner, Some([Some(true), Some(true)]));
/// // Region範囲外(extendを超える)はRatioも取得できない
/// let (ratio, corner) = corner_test(&mut rectgrid, [400.0, -10.0], &region, extend, 0.1);
/// assert_eq!(ratio, None);
/// assert_eq!(corner, None);
/// ```
pub fn corner_test<const D: usize>(
    rectgrid:     &mut Rectgrid<D>,
    coord:     [Px; D],
    region:    &Region<D>,
    extend:    Option<([Unit; D], [Unit; D])>,
    threshold: f64,
) -> (Option<[f64; D]>, Option<[Option<bool>; D]>) {
    if !region.has_size() { return (None, None); }
    let Some(result) = rectgrid
        .judge(coord, core::slice::from_ref(region), extend, true)
        .ok().and_then(|mut v| v.pop().flatten()) else {
        return (None, None);
    };
    let JudgeResult::Ratio(ratio) = result else { return (None, None); };
    let corner: [Option<bool>; D] = core::array::from_fn(|i| {
        if ratio[i] <= threshold { Some(true) }
        else if ratio[i] >= 1.0 - threshold { Some(false) }
        else { None }
    });
    let corner = if corner.iter().all(Option::is_some) { Some(corner) } else { None };
    (Some(ratio), corner)
}

/// PointerDown時、pointer(viewport座標)とhitしたRegionのbaseから、ドラッグオフセットを求める。
///
/// ```
/// use rectgrid::rectgrid::{Rectgrid, DefiningExpression, Region, Length, pointer_down_offset};
/// let mut rectgrid = Rectgrid::<2>::new([10.0, 20.0], [DefiningExpression::Scale(200.0), DefiningExpression::Scale(64.0)]);
/// let region = Region { base: [Length::Unit(1.0), Length::Unit(0.0)], offset: [Length::Unit(1.0), Length::Unit(1.0)] };
/// // region.base のsection基準pxは(200, 0)。originを補正した上で(220, 30)を掴んだ場合のオフセット。
/// let offset = pointer_down_offset(&mut rectgrid, [230.0, 50.0], &region).unwrap();
/// assert_eq!(offset, [20.0, 30.0]);
/// ```
pub fn pointer_down_offset<const D: usize>(
    rectgrid:  &mut Rectgrid<D>,
    coord:  [Px; D],
    region: &Region<D>,
) -> Result<[Px; D], OutOfIndex> {
    let base_px = rectgrid.unit_point(&region.base)?;
    Ok(rectgrid.drag_offset(coord, base_px))
}

/// Drag中、角ハンドルドラッグによってRegionのbase/offsetを更新し、更新後のRegionとpxを返す。
/// corner[i] = Some(base_side): base_side==trueならbase側の辺を、falseならoffset側の辺を動かす。
///
/// ```
/// use rectgrid::rectgrid::{Rectgrid, DefiningExpression, Region, Length, drag_resize};
/// let mut rectgrid = Rectgrid::<2>::new([0.0, 0.0], [DefiningExpression::Scale(200.0), DefiningExpression::Scale(64.0)]);
/// let region = Region { base: [Length::Unit(2.0), Length::Unit(0.0)], offset: [Length::Unit(1.0), Length::Unit(3.0)] };
/// // 左上(base側)の角を(150, 0)pxへドラッグ → x軸のbaseが0unitへ縮む
/// let (resized, base_px, offset_px) = drag_resize(&mut rectgrid, [150.0, 0.0], &region, [Some(true), None]).unwrap();
/// assert!(matches!(resized.base[0], Length::Unit(v) if v == 0.0));
/// assert_eq!(base_px[0], 0.0);
/// assert_eq!(offset_px[1], 192.0); // y軸(offset=3.0unit)は未変更
/// ```
pub fn drag_resize<const D: usize>(
    rectgrid:  &mut Rectgrid<D>,
    pointer: [Px; D],
    region:  &Region<D>,
    corner:  [Option<bool>; D],
) -> Result<(Region<D>, [Px; D], [Px; D]), OutOfIndex> {
    let mut region = *region;
    for i in 0..D {
        let Some(base_side) = corner[i] else { continue };
        let new_u = rectgrid.axis_unit_from_pointer(i, pointer)?.floor();
        let base_u   = if let Length::Unit(v) = region.base[i]   { v } else { 0.0 };
        let offset_u = if let Length::Unit(v) = region.offset[i] { v } else { 1.0 };
        if base_side {
            let new_offset = (base_u + offset_u - new_u).max(1.0);
            region.base[i]   = Length::Unit(new_u);
            region.offset[i] = Length::Unit(new_offset);
        } else {
            let new_offset = (new_u - base_u).max(1.0);
            region.offset[i] = Length::Unit(new_offset);
        }
    }
    let base_px   = rectgrid.unit_point(&region.base)?;
    let offset_px = rectgrid.unit_point(&region.offset)?;
    Ok((region, base_px, offset_px))
}

/// Drag中、移動ドラッグ(角ハンドルでない)によってRegionのbaseをPxで更新し、更新後のRegionとpxを返す。
///
/// ```
/// use rectgrid::rectgrid::{Rectgrid, DefiningExpression, Region, Length, drag_translate};
/// let mut rectgrid = Rectgrid::<2>::new([10.0, 20.0], [DefiningExpression::Scale(200.0), DefiningExpression::Scale(64.0)]);
/// let region = Region { base: [Length::Unit(0.0), Length::Unit(0.0)], offset: [Length::Unit(1.0), Length::Unit(1.0)] };
/// // pointerをviewport座標(230, 50)、drag_offsetを(20, 30)として移動
/// let (moved, px) = drag_translate(&mut rectgrid, [230.0, 50.0], [20.0, 30.0], &region);
/// assert_eq!(px, [200.0, 0.0]);
/// assert!(matches!(moved.base[0], Length::Px(v) if v == 200.0));
/// assert!(matches!(moved.base[1], Length::Px(v) if v == 0.0));
/// ```
pub fn drag_translate<const D: usize>(
    rectgrid:       &mut Rectgrid<D>,
    pointer:     [Px; D],
    drag_offset: [Px; D],
    region:      &Region<D>,
) -> (Region<D>, [Px; D]) {
    let px = rectgrid.drag_move(pointer, drag_offset);
    let mut region = *region;
    region.base = core::array::from_fn(|i| Length::Px(px[i]));
    (region, px)
}

/// DragEnd時、面積持ちRegionの移動ドラッグ結果(base=PxのRegion)をUnit格子にスナップする。
/// extendはsnap_floorにそのまま渡す。
///
/// ```
/// use rectgrid::rectgrid::{Rectgrid, DefiningExpression, Region, Length, snap_region_to_unit};
/// let mut rectgrid = Rectgrid::<2>::new([0.0, 0.0], [DefiningExpression::Scale(200.0), DefiningExpression::Scale(64.0)]);
/// let region = Region { base: [Length::Px(430.0), Length::Px(70.0)], offset: [Length::Unit(1.0), Length::Unit(1.0)] };
/// // 430px / 200 = 2.15unit, 70px / 64 = 1.09375unit。extend +0.25してfloor。
/// let (snapped, base_px) = snap_region_to_unit(&mut rectgrid, &region, Some([0.25, 0.25])).unwrap();
/// assert!(matches!(snapped.base[0], Length::Unit(v) if v == 2.0));
/// assert!(matches!(snapped.base[1], Length::Unit(v) if v == 1.0));
/// assert_eq!(base_px, [400.0, 64.0]);
/// ```
pub fn snap_region_to_unit<const D: usize>(
    rectgrid:  &mut Rectgrid<D>,
    region: &Region<D>,
    extend: Option<[Unit; D]>,
) -> Result<(Region<D>, [Px; D]), OutOfIndex> {
    let mut region = *region;
    for i in 0..D {
        if let Length::Px(px) = region.base[i] {
            let unit_px = rectgrid.eval(i, 1.0)?;
            region.base[i] = Length::Unit(px / unit_px);
        }
    }
    region.snap_floor(extend);
    let base_px = rectgrid.unit_point(&region.base)?;
    Ok((region, base_px))
}

/// DragEnd時、点Region(面積なし)のdrag_px(section基準px、既にdrag_offset適用済み)から
/// Unit格子にスナップしたRegionを求める。
///
/// ```
/// use rectgrid::rectgrid::{Rectgrid, DefiningExpression, Region, Length, snap_point_to_unit};
/// let mut rectgrid = Rectgrid::<2>::new([0.0, 0.0], [DefiningExpression::Scale(200.0), DefiningExpression::Scale(64.0)]);
/// // 430px / 200 = 2.15unit, 70px / 64 = 1.09375unit。snap +0.25してfloor。
/// let (region, base_px) = snap_point_to_unit(&mut rectgrid, [430.0, 70.0], [0.25, 0.25]).unwrap();
/// assert!(matches!(region.base[0], Length::Unit(v) if v == 2.0));
/// assert!(matches!(region.base[1], Length::Unit(v) if v == 1.0));
/// assert!(!region.has_size());
/// assert_eq!(base_px, [400.0, 64.0]);
/// ```
pub fn snap_point_to_unit<const D: usize>(
    rectgrid:   &mut Rectgrid<D>,
    drag_px: [Px; D],
    snap:    [Unit; D],
) -> Result<(Region<D>, [Px; D]), OutOfIndex> {
    let mut base: Point<D> = [Length::Unit(0.0); D];
    for i in 0..D {
        let unit_px = rectgrid.eval(i, 1.0)?;
        base[i] = Length::Unit((drag_px[i] / unit_px + snap[i]).floor());
    }
    let region = Region { base, offset: core::array::from_fn(|_| Length::Unit(0.0)) };
    let base_px = rectgrid.unit_point(&region.base)?;
    Ok((region, base_px))
}
