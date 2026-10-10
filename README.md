# rectgrid

[![Crates.io](https://img.shields.io/crates/v/rectgrid.svg)](https://crates.io/crates/rectgrid)

Boundary box operations on rectilinear grids with arbitrary unit systems.

- A crate for operating on a two-point coordinate boundary box and its collections, defined over the grid of an arbitrary unit system — a rectilinear grid whose axes each have an independent increment function. Such a unit system is defined by an intrinsic origin and per-axis increment functions, both expressed in a base unit (unit: a general-purpose unit). It further provides, for any single boundary box, a conversion function into a local unit system (parameter) in which each axis has unit vector length, making it possible to implement boundary tests against arbitrary geometry.
- The base unit system is defined as the unit system whose origin lies at (0, ..., 0) and whose per-axis increment functions all return the constant 1. This base unit is named Px (pixel: picture element).

To enable the `geometry` module:

```toml
[dependencies]
rectgrid = { version = "0.4", features = ["geometry"] }
```

[English](#rectgrid) | [日本語](#ja)

---

## Version

| Version | Status    | Date       | Description |
|---------|-----------|------------|-------------|
| 0.1.0   | Released  | 2026-07-10 | 1st release |
| 0.1.1   | Released  | 2026-07-13 | improve performance(#7) |
| 0.2.0   | Released  | 2026-10-01 | add BBox::new() |
| 0.3.0   | Released  | 2026-10-02 | improve algorithm and tests |
| 0.4.0   | Released  | 2026-10-10 | Breaking: `Error` rename, NaN is rejected, `geometry` returns `Result` |

This project adheres to [Semantic Versioning](https://semver.org/).

---

## Coordinate system

- When treating rectgrid's x and y as 2D coordinates, x is the axis that becomes the width in the viewport, y is the height direction, and the origin (0,0) is the top-left corner.
- Px passed into a function from outside this crate is global (an external coordinate not yet corrected for origin, e.g. a viewport coordinate); each function subtracts origin internally to make it local. Px derived from a boundary box (base/offset) — the return value of `unit_to_px`, and anything built on it such as `hit_test`/`*_as_px` results — is always local (origin=0 as the reference). If such a value is passed back across a `RectGrid` boundary, treat it as local px.

---

## Public ports

| Item | Port | Parameter | Return | Description |
|-|-|-|-|-|
| `Value<Tag>` | `new` | `v: f64` | `Self` | - |
|              | `get` | - | `f64` | - |
| `PxTag` | - | - | - | - |
| `UnitTag` | - | - | - | - |
| `ParameterTag` | - | - | - | - |
| `Px` | - | - | - | `Value<PxTag>` |
| `Unit` | - | - | - | `Value<UnitTag>` |
| `Parameter` | - | - | - | `Value<ParameterTag>` |
| `Point<D>` | - | - | - | `[Unit; D]` |
| `BBox<D>` | `new` | `base: Point<D>, offset: Point<D>` | `Self` | Constructs a boundary box. A negative offset axis is normalized by advancing base to the far corner and negating offset, so offset is always non-negative |
|           | `base` | - | `Point<D>` | Start point |
|           | `offset` | - | `Point<D>` | Vector distance to the end point (each axis is guaranteed non-negative) |
|           | `snap_floor` | `extend: Option<[Unit; D]>` | `&mut Self` | Snaps base/offset to the integer grid via floor. extend applies to base only, added before flooring |
|           | `has_size` | - | `bool` | Whether offset is nonzero on every axis (i.e., the boundary box has area/volume) |
| `Error` | `OutOfIndex` | `u32` | - | Out-of-range access. Carries the largest valid index not above the access (0 if negative) |
|                 | `InvalidDefinition` | - | - | The definition is invalid and an evaluation closure cannot be built |
|                 | `InvalidInput` | - | - | An argument cannot be evaluated (e.g. NaN) |
| `StepFn` | `step` | `i: u32` | `Result<Px, Error>` | Implemented for every `Deref` to `Fn(u32) -> Result<Px, Error>` (`Rc`, `Arc`, `Box`, `&F`) |
| `DefaultSteps` | - | - | - | `Rc<dyn Fn(u32) -> Result<Px, Error>>`; the `P` of a grid that does not name one |
| `IncrementFunction<P>` | `ForwardDifference` | `P` | - | `P` points to `Fn(u32) -> Result<Px, Error>` (see `StepFn`); defaults to `Rc<dyn Fn(u32) -> Result<Px, Error>>` |
|                     | `VectorList` | `Vec<Px>` | - | An empty `Vec<Px>` is invalid |
|                     | `Scale` | `f64` | - | NaN is invalid |
|                     | `accumulate` | `self` | `Result<Accumulator<P>, Error>` | Builds a forward/inverse `Accumulator` from the definition |
| `Accumulator<P>` | `Scale` | `f64` | - | Inverse resolves analytically (`target / s`), no search needed |
|               | `VectorList` | `Vec<Px>` | - | Inverse resolves via `partition_point` + O(1) linear-interpolation solve |
|               | `ForwardDifference` | `P` | - | Inverse scans the segments |
|               | `forward` | `x: f64` | `Result<Px, Error>` | unit coordinate -> px |
|               | `inverse` | `target: Px` | `Result<Unit, Error>` | px -> unit coordinate |
| `RectGrid<D, P>` | `origin` | - | `[Px; D]` | Start point |
|               | `new`                 | `origin: [Px; D], definitions: [IncrementFunction<P>; D]` | `Result<Self, Error>` | - |
|               | `set_definition`      | `definition: IncrementFunction<P>, d: usize` | `Result<(), Error>` | Replaces the definition for axis d |
|               | `point_to_unit`       | `point: [Px; D]` | `[Result<Unit, Error>; D]` | Inverts px to unit (origin subtracted first); the accumulator must be non-decreasing over Unit >= 0 |
|               | `unit_to_px`          | `d: usize, unit: &Unit` | `Result<Px, Error>` | Converts a unit coordinate to px (evaluates the accumulator directly) |
|               | `point_as_px`         | `points: &[Point<D>]` | `Vec<[Result<Px, Error>; D]>` | Converts unit points to px, one Result per axis |
|               | `box_as_px`           | `boxes: &[BBox<D>]` | `Vec<[Result<(Px, Px), Error>; D]>` | Converts boundary boxes to (base_px, offset_px), one Result per axis |
|               | `hit_test`            | `point: [Px; D], boxes: &[BBox<D>], extend: Option<([Unit; D], [Unit; D])>` | `Option<usize>` | Returns the highest index among the boundary boxes point hits; an unevaluable boundary box never hits |
|               | `hit_test_with_parameter` | `point: [Px; D], boxes: &[BBox<D>], extend: Option<([Unit; D], [Unit; D])>` | `Option<(usize, [Parameter; D])>` | Like hit_test, returns the highest-index hit along with the get_parameter-equivalent value |
|               | `hit_tests`           | `point: [Px; D], boxes: &[BBox<D>], extend: Option<([Unit; D], [Unit; D])>` | `Vec<bool>` | Returns hit/no-hit for every boundary box, in a Vec of the same length |
|               | `get_parameter`           | `point: [Px; D], bx: BBox<D>` | `[Result<Parameter, Error>; D]` | Signed local coordinate (side length normalized to 1), one Result per axis |
|               | `offset`              | `pointer: [Px; D], z: [Px; D]` | `[Px; D]` | pointer's local coordinate (after origin correction) with z subtracted |
| - | `corner_test<D, P>` | `grid: &RectGrid<D, P>, point: [Px; D], bx: &BBox<D>, threshold: f64, extend: Option<([Unit; D], [Unit; D])>` | `(Option<[Parameter; D]>, Option<[Option<bool>; D]>)` | For a boundary box with area, determines whether point is near an edge (within threshold) |
| - | `drag_resize<D, P>` | `grid: &RectGrid<D, P>, pointer: [Px; D], bx: &BBox<D>, corner: [Option<bool>; D]` | `Result<BBox<D>, Error>` | Updates the boundary box's base/offset via a corner-handle drag |
| - | `drag_translate<D, P>` | `grid: &RectGrid<D, P>, pointer: [Px; D], drag_offset: [Px; D]` | `[Px; D]` | Computes base's px position during a move drag |
| - | `snap_bbox_to_unit<D, P>` | `grid: &RectGrid<D, P>, pointer: [Px; D], drag_offset: [Px; D], bx: &BBox<D>, extend: Option<[Unit; D]>` | `Result<BBox<D>, Error>` | At DragEnd, snaps the move-drag result of a boundary box with area to the Unit grid |
| - | `snap_point_to_unit<D, P>` | `grid: &RectGrid<D, P>, pointer: [Px; D], drag_offset: [Px; D], snap: [Unit; D]` | `Result<BBox<D>, Error>` | At DragEnd, computes a boundary box snapped to the Unit grid from the move-drag result of a boundary box without area (a point) |
| `Line<D>` | `start` | - | `Point<D>` | Start point |
|           | `end` | - | `Point<D>` | End point |
| `Circle<D>` | `center` | - | `Point<D>` | Center |
|             | `radius` | - | `Unit` | Radius |
|             | `from_three_points` | `a: Point<2>, b: Point<2>, c: Point<2>` | `Result<Option<Self>, Error>` | Circle through three points (`Circle<2>`); `Ok(None)` when they are collinear (relative threshold 1e-12) |
| `Ellipse<D>` | `center` | - | `Point<D>` | Center |
|              | `rx` | - | `Unit` | Semi-axis along x (axis-aligned) |
|              | `ry` | - | `Unit` | Semi-axis along y (axis-aligned) |
| `Polygon<D>` | `vertices` | - | `Vec<Point<D>>` | Vertices in order; the last connects back to the first |
| `PointOnGeometry<D>` | `t` | - | `Parameter` | Position of `projected` on the geometry; its meaning is given per `as_on_*` |
|                      | `projected` | - | `Point<D>` | Closest point on the geometry |
|                      | `signed_distance` | - | `Unit` | Distance to `projected`; negative inside, positive outside |
| - | `as_on_line` | `point: [Unit; 2], line: Line<2>` | `Result<PointOnGeometry<2>, Error>` | t is unclamped (0 at start, 1 at end); signed_distance is positive when `cross(end - start, point - start) > 0` |
| - | `as_on_circle` | `point: [Unit; 2], circle: Circle<2>` | `Result<PointOnGeometry<2>, Error>` | t is the angle (radians, `atan2`) of point around the center |
| - | `as_on_ellipse` | `point: [Unit; 2], ellipse: Ellipse<2>` | `Result<PointOnGeometry<2>, Error>` | t is the parametric angle of the closest point; a zero `rx` or `ry` gives the distance to the center |
| - | `as_on_polygon` | `point: [Unit; 2], polygon: Polygon<2>` | `Result<(PointOnGeometry<2>, usize), Error>` | t (0 to 1) and index of the nearest edge (edge `i` runs `vertices[i]` to `vertices[i + 1]`); outside is positive for a counter-clockwise polygon; `Err(InvalidInput)` below 3 vertices |

---

# Ja

- 各軸が独立した階差関数を持つ直交座標系(rectilinear grid)の、固有の原点座標と各軸の階差関数を与単位で定義した任意単位系(unit: 一般単位)の格子上で、2点間座標のboundary boxとその集合を操作するための幾何計算クレート。さらに、単一のboundary boxの、各軸のベクトル長を1とした局所単位系(parameter)への変換関数により、任意の幾何による境界判定を実装可能にする。

- 与単位系とは、原点の座標が(0,...,0), 全ての軸の階差関数が定数1を返す単位系を指す。単位名をPx(pixel: picture element)とする。

`geometry`モジュールはfeature機能で有効化できる:

```toml
[dependencies]
rectgrid = { version = "0.4", features = ["geometry"] }
```

## 座標系

- rectgridのx, yを2D座標として扱う場合、xはviewportで幅になる軸、yは高さ方向、原点(0,0)は左上隅とする。
- クレート外部から関数引数として渡されるpxはglobal(origin未補正の外部座標、例えばviewport座標)として受け取り、各関数の内部でoriginを差し引いてlocal化する。一方、boundary box(base/offset)由来のpx(`unit_to_px`の戻り値や、それを使う`hit_test`系・`*_as_px`系の戻り値)は常にlocal(origin=0を基準とした座標)を返す。呼び出し側が`RectGrid`を跨いで再度渡す場合はlocal pxとして扱う。

## 公開ポート

| アイテム | ポート | 引数 | 戻り値 | 説明 |
|-|-|-|-|-|
| `Value<Tag>` | `new` | `v: f64` | `Self` | - |
|              | `get` | - | `f64` | - |
| `PxTag` | - | - | - | - |
| `UnitTag` | - | - | - | - |
| `ParameterTag` | - | - | - | - |
| `Px` | - | - | - | `Value<PxTag>` |
| `Unit` | - | - | - | `Value<UnitTag>` |
| `Parameter` | - | - | - | `Value<ParameterTag>` |
| `Point<D>` | - | - | - | `[Unit; D]` |
| `BBox<D>` | `new` | `base: Point<D>, offset: Point<D>` | `Self` | boundary boxを構築する。offsetのいずれかの軸が負の場合はbaseをその軸の終点側へ進め、offsetを反転して非負に正規化する |
|           | `base` | - | `Point<D>` | 始点 |
|           | `offset` | - | `Point<D>` | 終点までのベクトル距離(各軸は非負であることが保証される) |
|           | `snap_floor` | `extend: Option<[Unit; D]>` | `&mut Self` | base/offsetをfloor整数格子にスナップ。extendはbaseにのみfloor前に加算 |
|           | `has_size` | - | `bool` | 全軸のoffsetが非ゼロか(面積/体積を持つboundary boxか) |
| `Error` | `OutOfIndex` | `u32` | - | 範囲外アクセス。アクセス位置以下で最大の有効index(負なら0) |
|                 | `InvalidDefinition` | - | - | 定義が不正で評価クロージャを構築できない |
|                 | `InvalidInput` | - | - | 引数を評価できない(例: NaN) |
| `StepFn` | `step` | `i: u32` | `Result<Px, Error>` | `Fn(u32) -> Result<Px, Error>`への`Deref`すべてに実装(`Rc`, `Arc`, `Box`, `&F`) |
| `DefaultSteps` | - | - | - | `Rc<dyn Fn(u32) -> Result<Px, Error>>`。`P`を指定しない格子の`P` |
| `IncrementFunction<P>` | `ForwardDifference` | `P` | - | `P`は`Fn(u32) -> Result<Px, Error>`へのポインタ(`StepFn`)。既定は`Rc<dyn Fn(u32) -> Result<Px, Error>>` |
|                     | `VectorList` | `Vec<Px>` | - | 空のVec<Px>は不正 |
|                     | `Scale` | `f64` | - | NaNは不正 |
|                     | `accumulate` | `self` | `Result<Accumulator<P>, Error>` | 定義から順変換・逆変換を持つ`Accumulator`を構築する |
| `Accumulator<P>` | `Scale` | `f64` | - | 逆変換は解析的(`target / s`)に即決、探索不要 |
|               | `VectorList` | `Vec<Px>` | - | 逆変換は`partition_point`と線形補間の逆算(O(1))で解決 |
|               | `ForwardDifference` | `P` | - | 逆変換は区間を走査して解く |
|               | `forward` | `x: f64` | `Result<Px, Error>` | unit座標 -> px |
|               | `inverse` | `target: Px` | `Result<Unit, Error>` | px -> unit座標 |
| `RectGrid<D, P>` | `origin` | - | `[Px; D]` | 始点 |
|               | `new`                 | `origin: [Px; D], definitions: [IncrementFunction<P>; D]` | `Result<Self, Error>` | - |
|               | `set_definition`      | `definition: IncrementFunction<P>, d: usize` | `Result<(), Error>` | d軸の定義を差し替える |
|               | `point_to_unit`       | `point: [Px; D]` | `[Result<Unit, Error>; D]` | pxをunitへ逆変換(originを差し引いてから変換。accumulatorがUnit>=0で単調非減少である前提) |
|               | `unit_to_px`          | `d: usize, unit: &Unit` | `Result<Px, Error>` | unit座標をpxへ変換(accumulatorをそのまま評価) |
|               | `point_as_px`         | `points: &[Point<D>]` | `Vec<[Result<Px, Error>; D]>` | 複数のunit座標点をpxへ変換。軸ごとのResultを返す |
|               | `box_as_px`           | `boxes: &[BBox<D>]` | `Vec<[Result<(Px, Px), Error>; D]>` | 複数のboundary boxを(base_px, offset_px)へ変換。軸ごとのResultを返す |
|               | `hit_test`            | `point: [Px; D], boxes: &[BBox<D>], extend: Option<([Unit; D], [Unit; D])>` | `Option<usize>` | pointにhitするboundary boxのうちindex最大のものを返す。評価不能なboundary boxはhitしない |
|               | `hit_test_with_parameter` | `point: [Px; D], boxes: &[BBox<D>], extend: Option<([Unit; D], [Unit; D])>` | `Option<(usize, [Parameter; D])>` | hit_testと同様にindex最大のhitとget_parameter相当の値を返す |
|               | `hit_tests`           | `point: [Px; D], boxes: &[BBox<D>], extend: Option<([Unit; D], [Unit; D])>` | `Vec<bool>` | 全てのboundary boxについてhit有無を、同じ長さのVecで返す |
|               | `get_parameter`           | `point: [Px; D], bx: BBox<D>` | `[Result<Parameter, Error>; D]` | 単一のboundary boxの各辺長を1とした符号付き局所座標。軸ごとのResultを返す |
|               | `offset`              | `pointer: [Px; D], z: [Px; D]` | `[Px; D]` | pointerのlocal座標(origin補正後)からzを差し引いた値 |
| - | `corner_test<D, P>` | `grid: &RectGrid<D, P>, point: [Px; D], bx: &BBox<D>, threshold: f64, extend: Option<([Unit; D], [Unit; D])>` | `(Option<[Parameter; D]>, Option<[Option<bool>; D]>)` | 面積を持つboundary boxに対しpointが辺付近(threshold未満)にあるかを判定 |
| - | `drag_resize<D, P>` | `grid: &RectGrid<D, P>, pointer: [Px; D], bx: &BBox<D>, corner: [Option<bool>; D]` | `Result<BBox<D>, Error>` | 角ハンドルドラッグによってboundary boxのbase/offsetを更新する |
| - | `drag_translate<D, P>` | `grid: &RectGrid<D, P>, pointer: [Px; D], drag_offset: [Px; D]` | `[Px; D]` | 移動ドラッグ中のbaseのpx位置を求める |
| - | `snap_bbox_to_unit<D, P>` | `grid: &RectGrid<D, P>, pointer: [Px; D], drag_offset: [Px; D], bx: &BBox<D>, extend: Option<[Unit; D]>` | `Result<BBox<D>, Error>` | DragEnd時、面積を持つboundary boxの移動ドラッグ結果をUnit格子にスナップする |
| - | `snap_point_to_unit<D, P>` | `grid: &RectGrid<D, P>, pointer: [Px; D], drag_offset: [Px; D], snap: [Unit; D]` | `Result<BBox<D>, Error>` | DragEnd時、面積を持たない(点の)boundary boxの移動ドラッグ結果をUnit格子にスナップしたboundary boxを求める |
| `Line<D>` | `start` | - | `Point<D>` | 始点 |
|           | `end` | - | `Point<D>` | 終点 |
| `Circle<D>` | `center` | - | `Point<D>` | 中心 |
|             | `radius` | - | `Unit` | 半径 |
|             | `from_three_points` | `a: Point<2>, b: Point<2>, c: Point<2>` | `Result<Option<Self>, Error>` | 3点を通る円(`Circle<2>`)。3点が一直線上(相対閾値1e-12)なら`Ok(None)` |
| `Ellipse<D>` | `center` | - | `Point<D>` | 中心 |
|              | `rx` | - | `Unit` | x方向の半径(軸平行) |
|              | `ry` | - | `Unit` | y方向の半径(軸平行) |
| `Polygon<D>` | `vertices` | - | `Vec<Point<D>>` | 頂点を順に並べたもの。最後の頂点は最初の頂点へ戻る |
| `PointOnGeometry<D>` | `t` | - | `Parameter` | `projected`の幾何上の位置。意味は`as_on_*`ごと |
|                      | `projected` | - | `Point<D>` | 幾何上の最近点 |
|                      | `signed_distance` | - | `Unit` | `projected`までの距離。内側が負、外側が正 |
| - | `as_on_line` | `point: [Unit; 2], line: Line<2>` | `Result<PointOnGeometry<2>, Error>` | tは切り詰めない(startで0、endで1)。`cross(end - start, point - start) > 0`のときsigned_distanceは正 |
| - | `as_on_circle` | `point: [Unit; 2], circle: Circle<2>` | `Result<PointOnGeometry<2>, Error>` | tは中心まわりのpointの角度(ラジアン、`atan2`) |
| - | `as_on_ellipse` | `point: [Unit; 2], ellipse: Ellipse<2>` | `Result<PointOnGeometry<2>, Error>` | tは最近点のパラメトリック角。`rx`か`ry`が0なら中心までの距離 |
| - | `as_on_polygon` | `point: [Unit; 2], polygon: Polygon<2>` | `Result<(PointOnGeometry<2>, usize), Error>` | 最近辺のt(0〜1)とindex(辺`i`は`vertices[i]`から`vertices[i + 1]`)。反時計回りの多角形では外側が正。3頂点未満なら`Err(InvalidInput)` |
