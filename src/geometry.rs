// This file includes untranslated text (ja).
// 前提: rectgrid(RectGrid/BBox等)自体は各次元D間の関係性を扱わないが、
// 幾何実装の上ではD間で各座標は等しく扱われる。
// 座標変換基盤はrectgridクレートに委譲し、
// このファイルでは幾何図形(Line/Circle/Ellipse/Polygon)の当たり判定のみを実装する。

use alloc::vec::Vec;
use core::primitive::{usize, f64};
use libm;
use crate::{Point, Unit, Parameter};

/// D次元拡張は、あまり重視せず、難しければ2に固定してもよい。
pub struct Line<const D: usize> {
    pub start: Point<D>,
    pub end: Point<D>,
}

/// 線分`line`に対する`point`の相対情報。
/// 無限直線上の符号付き判定（`nx + my = l`型）ではなく、
/// 線分の始点/終点を1本の物差しとした局所座標（t）と、
/// そこからの垂直距離を返す。
///
/// ここでの t は [0, 1] にクランプしない。クランプすると「線分の外に
/// 射影された」という情報が失われ、有界判定ができなくなるため。
/// 呼び出し側は `t < 0.0 || t > 1.0` で線分の範囲外
/// （投影点が線分の外に出る）と判定できる。
///
/// D次元拡張は未対応（D=2固定）。line/pointは同一単位系（Unit）の
/// 値が渡される前提とし、ここでは単位変換は行わない。
///
/// 3-4-5三角形のデータセット: 水平線分(0,0)-(10,0)に対して点(5,5)は
/// 垂直距離5、tは線分の中央(0.5)に射影される。
///
/// ```
/// use rectgrid::Unit;
/// use rectgrid::geometry::*;
///
/// let line = Line { start: [Unit::new(0.0), Unit::new(0.0)], end: [Unit::new(10.0), Unit::new(0.0)] };
/// let result = as_on_line([Unit::new(5.0), Unit::new(5.0)], line);
/// assert_eq!(result.t.get(), 0.5);
/// assert_eq!(result.signed_distance.get().abs(), 5.0);
/// ```
pub fn as_on_line(point: [Unit; 2], line: Line<2>) -> PointOnGeometry<2> {
    let px = point[0].get();
    let py = point[1].get();
    let x1 = line.start[0].get();
    let y1 = line.start[1].get();
    let x2 = line.end[0].get();
    let y2 = line.end[1].get();

    let vx = x2 - x1;
    let vy = y2 - y1;
    let wx = px - x1;
    let wy = py - y1;

    let length_squared = vx * vx + vy * vy;
    let t = if length_squared == 0.0 {
        0.0
    } else {
        (wx * vx + wy * vy) / length_squared
    };

    let proj_x = x1 + t * vx;
    let proj_y = y1 + t * vy;

    let dx = px - proj_x;
    let dy = py - proj_y;
    let distance = libm::sqrt(dx * dx + dy * dy);

    // 線分の進行方向(vx, vy)に対する外積の符号で左右を判定する。
    let cross = vx * wy - vy * wx;
    let sign = if cross < 0.0 { -1.0 } else { 1.0 };

    PointOnGeometry {
        t: Parameter::new(t),
        projected: [Unit::new(proj_x), Unit::new(proj_y)],
        signed_distance: Unit::new(sign * distance),
    }
}

/// D次元拡張は未対応（D=2固定、Lineと同様の方針）。
pub struct Circle<const D: usize> {
    pub center: Point<D>,
    pub radius: Unit,
}

impl Circle<2> {
    /// 円周上の3点a, b, cから中心・半径を求める外心公式。
    /// 3点円は独立した図形ではなく、Circleの生成方法の一つに過ぎない。
    ///
    /// 3点が一直線上（またはそれに近い）場合はNoneを返す。
    ///
    /// データセット: 単位円上の3点(0,1),(1,0),(0,-1) → 中心(0,0)、半径1。
    ///
    /// ```
    /// use rectgrid::Unit;
    /// use rectgrid::geometry::*;
    ///
    /// let result = Circle::from_three_points(
    ///     [Unit::new(0.0), Unit::new(1.0)],
    ///     [Unit::new(1.0), Unit::new(0.0)],
    ///     [Unit::new(0.0), Unit::new(-1.0)],
    /// ).unwrap();
    /// assert!((result.center[0].get()).abs() < 1e-8);
    /// assert!((result.center[1].get()).abs() < 1e-8);
    /// assert!((result.radius.get() - 1.0).abs() < 1e-8);
    /// ```
    pub fn from_three_points(a: Point<2>, b: Point<2>, c: Point<2>) -> Option<Self> {
        const EPSILON: f64 = 1e-8;

        let ax = a[0].get();
        let ay = a[1].get();
        let bx = b[0].get();
        let by = b[1].get();
        let cx = c[0].get();
        let cy = c[1].get();

        // 特定の頂点ペアのy座標差(dy)で割る形の公式は、そのペアがたまたま同じy座標を
        // 持つ非退化な三角形でも誤ってNoneを返してしまうため使わない。ここでは
        // どの座標成分にも依存しない対称な行列式ベースの外心公式を用いる。
        let denominator = ((bx - cx) * (cy - ay) + (cx - ax) * (cy - by)) * 2.0;
        if denominator.abs() < EPSILON {
            return None;
        }

        let a_sq = ax * ax + ay * ay;
        let b_sq = bx * bx + by * by;
        let c_sq = cx * cx + cy * cy;

        let center_x = (a_sq * (by - cy) + b_sq * (cy - ay) + c_sq * (ay - by)) / denominator;
        let center_y = (a_sq * (cx - bx) + b_sq * (ax - cx) + c_sq * (bx - ax)) / denominator;

        let radius = libm::sqrt((center_x - ax) * (center_x - ax) + (center_y - ay) * (center_y - ay));

        Some(Circle {
            center: [Unit::new(center_x), Unit::new(center_y)],
            radius: Unit::new(radius),
        })
    }
}

/// 円`circle`に対する`point`の相対情報。
///
/// t: +x軸方向を0とし、反時計回りのラジアン（atan2の生値、[-pi, pi]）。
/// 円は閉曲線のため線分のような「範囲外」は存在せず、tは常にこの区間に収まる
/// （as_on_lineのように範囲外か否かの判定には使えない）。
///
/// signed_distance: 円周までの符号付き距離（内側なら負、外側なら正）。
/// as_on_lineの「進行方向に対する左右」とは符号の意味が異なるので、
/// 図形によって signed_distance の解釈が変わることに注意。
///
/// D次元拡張は未対応（D=2固定）。単位変換はここでは行わない。
///
/// データセット: 中心(0,0)半径5の円に対し、中心そのものは円周まで距離5（内側なので負）。
///
/// ```
/// use rectgrid::Unit;
/// use rectgrid::geometry::*;
///
/// let circle = Circle { center: [Unit::new(0.0), Unit::new(0.0)], radius: Unit::new(5.0) };
/// let result = as_on_circle([Unit::new(0.0), Unit::new(0.0)], circle);
/// assert_eq!(result.signed_distance.get(), -5.0);
/// ```
pub fn as_on_circle(point: [Unit; 2], circle: Circle<2>) -> PointOnGeometry<2> {
    let px = point[0].get();
    let py = point[1].get();
    let cx = circle.center[0].get();
    let cy = circle.center[1].get();
    let radius = circle.radius.get();

    let dx = px - cx;
    let dy = py - cy;
    let distance_from_center = libm::sqrt(dx * dx + dy * dy);

    let t = libm::atan2(dy, dx);

    let (proj_x, proj_y) = if distance_from_center == 0.0 {
        // 中心と一致する場合は方向が定まらないため、規約上t=0方向に射影する。
        (cx + radius, cy)
    } else {
        let scale = radius / distance_from_center;
        (cx + dx * scale, cy + dy * scale)
    };

    PointOnGeometry {
        t: Parameter::new(t),
        projected: [Unit::new(proj_x), Unit::new(proj_y)],
        signed_distance: Unit::new(distance_from_center - radius),
    }
}

/// D次元拡張は未対応（D=2固定、Circleと同様の方針）。軸はx/yに整列している前提
/// （回転した楕円は非対応）。
pub struct Ellipse<const D: usize> {
    pub center: Point<D>,
    pub rx: Unit,
    pub ry: Unit,
}

/// 楕円`ellipse`に対する`point`の相対情報。
///
/// 真の最近傍点は解析的に求めにくいため、正規化空間で単位円とみなした距離を
/// 短径でスケールし直す近似。signed_distance として内側/外側の符号を残す
/// （ellipse_value < 1 なら内側=負）。
///
/// t: +x軸方向を0とし、反時計回りのラジアン（正規化空間でのatan2、円と同じ規約）。
/// projected: 中心からt方向へ楕円境界まで伸ばした近似射影点（真の垂線の足ではない）。
///
/// rx/ryのいずれかが0の場合は退化楕円として、中心から点までの距離をそのまま返す。
///
/// データセット: 中心(0,0), rx=4, ry=3の楕円に対し、+x軸上で中心からの距離が
/// rxのちょうど2倍(8,0)の点は、正規化距離が2倍（ellipse_value=4, sqrt=2）となり、
/// 近似距離は (2 - 1) * min(rx, ry) = 3、外側なので符号は正。
///
/// ```
/// use rectgrid::Unit;
/// use rectgrid::geometry::*;
///
/// let ellipse = Ellipse { center: [Unit::new(0.0), Unit::new(0.0)], rx: Unit::new(4.0), ry: Unit::new(3.0) };
/// let result = as_on_ellipse([Unit::new(8.0), Unit::new(0.0)], ellipse);
/// assert_eq!(result.signed_distance.get(), 3.0);
/// ```
pub fn as_on_ellipse(point: [Unit; 2], ellipse: Ellipse<2>) -> PointOnGeometry<2> {
    let px = point[0].get();
    let py = point[1].get();
    let cx = ellipse.center[0].get();
    let cy = ellipse.center[1].get();
    let rx = ellipse.rx.get();
    let ry = ellipse.ry.get();

    let raw_dx = px - cx;
    let raw_dy = py - cy;

    if rx == 0.0 || ry == 0.0 {
        let distance = libm::sqrt(raw_dx * raw_dx + raw_dy * raw_dy);
        return PointOnGeometry {
            t: Parameter::new(libm::atan2(raw_dy, raw_dx)),
            projected: [Unit::new(cx), Unit::new(cy)],
            signed_distance: Unit::new(distance),
        };
    }

    let dx = raw_dx / rx;
    let dy = raw_dy / ry;
    let ellipse_value = dx * dx + dy * dy;
    let t = libm::atan2(dy, dx);

    let (proj_x, proj_y) = if ellipse_value == 0.0 {
        // 中心と一致する場合は方向が定まらないため、規約上t=0方向に射影する。
        (cx + rx, cy)
    } else {
        let scale = 1.0 / libm::sqrt(ellipse_value);
        (cx + raw_dx * scale, cy + raw_dy * scale)
    };

    let approx_distance = (libm::sqrt(ellipse_value) - 1.0) * rx.min(ry);

    PointOnGeometry {
        t: Parameter::new(t),
        projected: [Unit::new(proj_x), Unit::new(proj_y)],
        signed_distance: Unit::new(approx_distance),
    }
}

/// 頂点列は反時計回り（CCW）で与えること。CCW/CWの判定はrectgridの座標系
/// （x=右方向、y=下方向、原点は左上）における数学的な向きで行う。CWで渡した
/// 場合の挙動は未定義（内外判定・符号が意図と逆転する）。
/// 自己交差（星型・8の字等）も禁止・検出しない。
/// 頂点数が3未満の場合は不正な入力として扱う。D次元拡張は未対応（D=2固定）。
pub struct Polygon<const D: usize> {
    pub vertices: Vec<Point<D>>,
}

impl Polygon<2> {
    /// 頂点列から辺（Line）の列を生成する。最後の頂点から最初の頂点への辺を
    /// 含めて多角形を閉じる。頂点順序はCCW前提のため、辺の進行方向
    /// （as_on_lineの左右符号）がそのまま内外判定に使える
    /// （as_on_polygon側で辺の符号をそのまま採用する）。
    fn edges(&self) -> Vec<Line<2>> {
        let n = self.vertices.len();
        (0..n)
            .map(|i| Line {
                start: self.vertices[i],
                end: self.vertices[(i + 1) % n],
            })
            .collect()
    }
}

/// 多角形`polygon`に対する`point`の相対情報。
///
/// polygon.verticesはCCW（反時計回り、Polygon構造体のdocコメント参照）で
/// 与えること。CWで渡した場合、signed_distanceの符号が内外逆転する（未定義動作）。
/// 各辺についてas_on_lineを呼び、線分としての有界距離（tを[0,1]にクランプした上での
/// 距離）が最小の辺を選ぶ（as_on_line自体はクランプしないtを返すため、ここで
/// クランプし直す）。
///
/// t: 最近傍の辺の中でのクランプ済み相対座標（[0, 1]）。多角形全体を通した
/// 弧長パラメータではなく、あくまで「どの辺の、どのあたりか」を示す局所値。
/// どの辺が選ばれたかは戻り値のedge_indexで分かる。
///
/// signed_distance: 最近傍辺のas_on_line符号（進行方向に対する左右）を反転させたもの
/// （内側なら負、外側なら正）。CCW前提かつrectgridの座標系（x=右方向、y=下方向）
/// では、進行方向の右側が多角形の内側にあたるため、as_on_lineの符号をそのままでは
/// 使えず反転が必要（円・楕円のsigned_distanceと同じ意味に揃える）。自己交差
/// ポリゴンでは、最近傍辺の符号を反転しただけの値をそのまま採用し、特別扱いはしない。
///
/// 頂点が3未満の場合はpanicする（不正な入力として扱う）。
///
/// 戻り値は`(PointOnGeometry, edge_index)`のタプル。edge_indexは最近傍と判定された
/// 辺のインデックス（`polygon.edges()[edge_index]`、すなわち
/// `vertices[edge_index]`から`vertices[(edge_index + 1) % n]`への辺）。
/// 辺選択のループで既に距離比較しているため、返却にコストはかからない。
///
/// データセット: 三角形(0,0),(10,0),(10,10)に対し、外部の点(5,-2)は最近傍の辺(底辺、
/// index 0)まで距離2、外側なので符号は正。
///
/// ```
/// use rectgrid::Unit;
/// use rectgrid::geometry::*;
///
/// let polygon = Polygon { vertices: vec![
///     [Unit::new(0.0), Unit::new(0.0)],
///     [Unit::new(10.0), Unit::new(0.0)],
///     [Unit::new(10.0), Unit::new(10.0)],
/// ] };
/// let (result, edge_index) = as_on_polygon([Unit::new(5.0), Unit::new(-2.0)], polygon);
/// assert_eq!(result.signed_distance.get(), 2.0);
/// assert_eq!(edge_index, 0);
/// ```
pub fn as_on_polygon(point: [Unit; 2], polygon: Polygon<2>) -> (PointOnGeometry<2>, usize) {
    assert!(polygon.vertices.len() >= 3, "polygon must have at least 3 vertices");

    let px = point[0].get();
    let py = point[1].get();

    let mut best: Option<(f64, PointOnGeometry<2>, usize)> = None;

    for (i, edge) in polygon.edges().into_iter().enumerate() {
        let x1 = edge.start[0].get();
        let y1 = edge.start[1].get();
        let x2 = edge.end[0].get();
        let y2 = edge.end[1].get();

        let result = as_on_line(
            [Unit::new(px), Unit::new(py)],
            Line {
                start: [Unit::new(x1), Unit::new(y1)],
                end: [Unit::new(x2), Unit::new(y2)],
            },
        );

        // as_on_lineのtはクランプされていないため、ここで[0, 1]にクランプしてから
        // 辺選択用の有界距離を求め直す。
        let raw_t = result.t.get();
        let clamped_t = raw_t.max(0.0).min(1.0);
        let clamped_proj_x = x1 + clamped_t * (x2 - x1);
        let clamped_proj_y = y1 + clamped_t * (y2 - y1);
        let bounded_distance = libm::sqrt(
            (px - clamped_proj_x) * (px - clamped_proj_x) + (py - clamped_proj_y) * (py - clamped_proj_y),
        );

        if best.as_ref().map_or(true, |(d, _, _)| bounded_distance < *d) {
            best = Some((bounded_distance, result, i));
        }
    }

    let (_, nearest, edge_index) = best.expect("polygon must have at least one edge");

    // as_on_lineの符号は「進行方向に対する左右」（cross < 0 で負）。
    // CCW頂点列 + rectgridの座標系（x=右, y=下）では、内側は進行方向の右側に
    // あたるため、そのままでは符号が内外と逆になる。ここで反転させる。
    (
        PointOnGeometry {
            t: nearest.t,
            projected: nearest.projected,
            signed_distance: Unit::new(-nearest.signed_distance.get()),
        },
        edge_index,
    )
}

/// 図形上の点を表す共通の戻り値。
/// - `t`: 図形に沿った相対座標（線分/多角形はクランプ済み[0,1]の局所比率、円/楕円はラジアン角）
/// - `projected`: 図形上に射影した点そのもの
/// - `signed_distance`: 図形からの符号付き距離（内側/外側や左右の判定に使う）
pub struct PointOnGeometry<const D: usize> {
    pub t: Parameter,
    pub projected: Point<D>,
    pub signed_distance: Unit,
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;

    fn p(x: f64, y: f64) -> Point<2> {
        [Unit::new(x), Unit::new(y)]
    }

    // --- as_on_line ---
    // doc testで3-4-5三角形（線分上の垂直距離）は検証済み。ここでは境界値と退化を補う。

    #[test]
    fn as_on_line_returns_zero_when_point_lies_on_segment() {
        let line = Line { start: p(-5.0, -5.0), end: p(5.0, 5.0) };
        let result = as_on_line(p(0.0, 0.0), line);
        assert!(result.signed_distance.get().abs() < 1e-8);
    }

    #[test]
    fn as_on_line_handles_zero_length_segment() {
        let line = Line { start: p(2.0, 2.0), end: p(2.0, 2.0) };
        let result = as_on_line(p(5.0, 6.0), line);
        assert_eq!(result.signed_distance.get().abs(), 5.0);
    }

    #[test]
    fn as_on_line_t_outside_zero_one_means_beyond_segment() {
        // as_on_lineはtをクランプしないため、線分の延長線上の点はt<0またはt>1になる。
        let line = Line { start: p(0.0, 0.0), end: p(10.0, 0.0) };
        let result = as_on_line(p(-5.0, 0.0), line);
        assert!(result.t.get() < 0.0);
    }

    // --- as_on_circle ---
    // doc testで中心上の点（内側、負）は検証済み。ここでは外側・負座標・小数を補う。

    #[test]
    fn as_on_circle_negative_coordinates_and_decimals() {
        // 中心(-4,4)半径5の円周上の点(1,4)は距離0。
        let circle = Circle { center: p(-4.0, 4.0), radius: Unit::new(5.0) };
        let result = as_on_circle(p(1.0, 4.0), circle);
        assert!(result.signed_distance.get().abs() < 1e-8);
    }

    #[test]
    fn as_on_circle_outside_is_positive() {
        let circle = Circle { center: p(0.0, 0.0), radius: Unit::new(5.0) };
        let result = as_on_circle(p(10.0, 0.0), circle);
        assert_eq!(result.signed_distance.get(), 5.0);
    }

    // --- Circle::from_three_points ---
    // doc testで単位円は検証済み。ここでは一直線上（None）を補う。

    #[test]
    fn from_three_points_returns_none_for_colinear_points() {
        let result = Circle::from_three_points(p(0.0, 0.0), p(1.0, 1.0), p(2.0, 2.0));
        assert!(result.is_none());
    }

    #[test]
    fn from_three_points_handles_matching_y_on_first_and_third_point() {
        // a, cのy座標がたまたま一致していても(dy=0)、非退化な三角形なら
        // 外心を求められる（特定ペアのdyに依存する公式だと誤ってNoneになる回帰ケース）。
        let result = Circle::from_three_points(p(0.0, 0.0), p(1.0, 3.0), p(4.0, 0.0)).unwrap();
        assert!((result.center[0].get() - 2.0).abs() < 1e-8);
        assert!((result.center[1].get() - 1.0).abs() < 1e-8);
        assert!((result.radius.get() - 5.0_f64.sqrt()).abs() < 1e-8);
    }

    // --- as_on_ellipse ---
    // doc testで外側(rx*2)は検証済み。ここでは内側・中心一致・退化楕円を補う。

    #[test]
    fn as_on_ellipse_inside_is_negative() {
        let ellipse = Ellipse { center: p(0.0, 0.0), rx: Unit::new(4.0), ry: Unit::new(3.0) };
        let result = as_on_ellipse(p(2.0, 0.0), ellipse);
        assert!(result.signed_distance.get() < 0.0);
    }

    #[test]
    fn as_on_ellipse_degenerate_ry_zero_falls_back_to_center_distance() {
        let ellipse = Ellipse { center: p(0.0, 0.0), rx: Unit::new(4.0), ry: Unit::new(0.0) };
        let result = as_on_ellipse(p(3.0, 4.0), ellipse);
        assert_eq!(result.signed_distance.get(), 5.0);
    }

    // --- as_on_polygon ---
    // doc testで三角形の外部点（正）は検証済み。ここでは内側・複数候補を補う。

    #[test]
    fn as_on_polygon_inside_is_negative() {
        let polygon = Polygon {
            vertices: vec![p(0.0, 0.0), p(10.0, 0.0), p(10.0, 10.0), p(0.0, 10.0)],
        };
        let (result, _) = as_on_polygon(p(5.0, 5.0), polygon);
        assert!(result.signed_distance.get() < 0.0);
    }

    #[test]
    fn as_on_polygon_picks_nearest_among_multiple_shapes_edge() {
        // ここではas_on_polygonは単一Polygonのみを受け取るため、2つの三角形のうち
        // 近い方を個別に呼び出して距離を比較する形で等価性を確認する。
        let near = Polygon { vertices: vec![p(20.0, 0.0), p(30.0, 0.0), p(30.0, 10.0)] };
        let (result, _) = as_on_polygon(p(25.0, -2.0), near);
        assert_eq!(result.signed_distance.get(), 2.0);
    }

    #[test]
    fn as_on_polygon_cw_inverts_sign() {
        // 頂点列はCCW規約。同じ四角形をCW（逆順）で与えると、規約違反により
        // signed_distanceの符号が反転することの確認（内側点なのに正になる）。
        let ccw = Polygon { vertices: vec![p(0.0, 0.0), p(10.0, 0.0), p(10.0, 10.0), p(0.0, 10.0)] };
        let cw = Polygon { vertices: vec![p(0.0, 0.0), p(0.0, 10.0), p(10.0, 10.0), p(10.0, 0.0)] };
        let inside = p(5.0, 5.0);
        let (result_ccw, _) = as_on_polygon(inside, ccw);
        let (result_cw, _) = as_on_polygon(inside, cw);
        assert!(result_ccw.signed_distance.get() < 0.0);
        assert!(result_cw.signed_distance.get() > 0.0);
    }

    #[test]
    fn as_on_polygon_returns_nearest_edge_index() {
        // 三角形(0,0),(10,0),(10,10),(0,10)四角形の底辺(index 0)寄りの外部点は
        // edge_index=0を返すはず。
        let polygon = Polygon {
            vertices: vec![p(0.0, 0.0), p(10.0, 0.0), p(10.0, 10.0), p(0.0, 10.0)],
        };
        let (_, edge_index) = as_on_polygon(p(5.0, -2.0), polygon);
        assert_eq!(edge_index, 0);
    }

    #[test]
    #[should_panic(expected = "at least 3 vertices")]
    fn as_on_polygon_panics_for_less_than_three_vertices() {
        let polygon = Polygon { vertices: vec![p(0.0, 0.0), p(1.0, 1.0)] };
        as_on_polygon(p(0.0, 0.0), polygon);
    }
}
