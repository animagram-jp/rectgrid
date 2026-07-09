# rectgrid

[![Crates.io](https://img.shields.io/crates/v/rectgrid.svg)](https://crates.io/crates/rectgrid)

Region operations on rectilinear grids with arbitrary unit systems.

- A geometry module for operating on a two-point coordinate region (a box) and its collections (a region), defined over the grid of an arbitrary unit system — a rectilinear grid whose axes each have an independent increment function. Such a unit system is defined by an intrinsic origin and per-axis difference functions, both expressed in a base unit (unit: a general-purpose unit). It further provides, for any single box, a conversion function into a local unit system (parameter) in which each axis has unit vector length, making it possible to implement boundary tests against arbitrary geometry.
- The base unit system is defined as the unit system whose origin lies at (0, ..., 0) and whose per-axis difference functions all return the constant 1. This base unit is named Px (pixel: picture element).

[English](#rectgrid) | [日本語](#ja)

---

## Version

| Version | Status    | Date       | Description |
|---------|-----------|------------|-------------|
| 0.1.0   | Released  | 2026-07-10 | 1st release |

This project adheres to [Semantic Versioning](https://semver.org/).

---

## Commands

```bash
# unit test
cargo test

# wasm build
cd examples && wasm-pack build --target web --out-dir app --out-name app
```

---

## Coordinate system

- When treating the rectgrid module's x and y as 2D coordinates, x is the axis that becomes the width in the viewport, y is the height direction, and the origin (0,0) is the top-left corner.
- Px passed into a function from outside this module is global (an external coordinate not yet corrected for origin, e.g. a viewport coordinate); each function subtracts origin internally to make it local. Px derived from a box (base/offset) — the return value of `unit_to_px`, and anything built on it such as `hit_test`/`*_as_px` results — is always local (origin=0 as the reference). If such a value is passed back across a `RectGrid` boundary, treat it as local px.

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
| `BBox<D>` | `base` | - | `Point<D>` | Start point |
|           | `offset` | - | `Point<D>` | Vector distance to the end point |
|           | `snap_floor` | `extend: Option<[Unit; D]>` | `&mut Self` | Snaps base/offset to the integer grid via floor. extend applies to base only, added before flooring |
|           | `has_size` | - | `bool` | Whether offset is nonzero on every axis (i.e., the BBox has area/volume) |
| `RectgridError` | `OutOfIndex` | `u32` | - | Out-of-range access. The last valid index within range |
|                 | `InvalidDefinition` | - | - | The definition is invalid and an evaluation closure cannot be built |
| `IncrementFunction` | `ForwardDifference` | `Rc<dyn Fn(u32) -> Result<Px, RectgridError>>` | - | - |
|                     | `VectorList` | `Vec<Px>` | - | An empty `Vec<Px>` is invalid |
|                     | `Scale` | `f64` | - | - |
|                     | `accumulate` | - | `Result<Box<dyn Fn(f64) -> Result<Px, RectgridError>>, RectgridError>` | - |
| `RectGrid<D>` | `origin` | - | `[Px; D]` | Start point |
|               | `new`                 | `origin: [Px; D], definitions: [IncrementFunction; D]` | `Result<Self, RectgridError>` | - |
|               | `set_definition`      | `definition: IncrementFunction, d: usize` | `Result<(), RectgridError>` | Replaces the definition for axis d |
|               | `point_to_unit`       | `point: [Px; D]` | `[Result<Unit, RectgridError>; D]` | Numerically inverts px to unit (origin is subtracted before conversion; assumes the accumulator is monotonically non-decreasing over Unit >= 0) |
|               | `unit_to_px`          | `d: usize, unit: &Unit` | `Result<Px, RectgridError>` | Converts a unit coordinate to px (evaluates the accumulator directly) |
|               | `point_as_px`         | `points: &Vec<Point<D>>` | `Vec<Result<[Px; D], RectgridError>>` | Converts multiple unit coordinate points to px. A point with an unevaluable axis returns Err; other points are unaffected |
|               | `box_as_px`           | `boxes: &Vec<BBox<D>>` | `Vec<Result<([Px; D], [Px; D]), RectgridError>>` | Converts multiple BBox to (base_px, offset_px). offset_px is the actual side length accounting for base position (unit_to_px(base+offset) − unit_to_px(base)), correct even under a nonlinear accumulator. A box with an unevaluable axis returns Err; other boxes are unaffected |
|               | `hit_test`            | `point: [Px; D], boxes: &Vec<BBox<D>>, extend: Option<([Unit; D], [Unit; D])>` | `Option<usize>` | Returns the highest index among the boxes point hits |
|               | `hit_test_with_parameter` | `point: [Px; D], boxes: &Vec<BBox<D>>, extend: Option<([Unit; D], [Unit; D])>` | `Option<(usize, [Parameter; D])>` | Like hit_test, returns the highest-index hit along with the get_parameter-equivalent value |
|               | `hit_tests`           | `point: [Px; D], boxes: &Vec<BBox<D>>, extend: Option<([Unit; D], [Unit; D])>` | `Vec<bool>` | Scans every box point hits and returns hit/no-hit for each, in a Vec the same length as boxes |
|               | `get_parameter`           | `point: [Px; D], bx: BBox<D>` | `[Parameter; D]` | Signed local coordinate for a single box, with each side length normalized to 1 |
|               | `offset`              | `pointer: [Px; D], z: [Px; D]` | `[Px; D]` | pointer's local coordinate (after origin correction) with z subtracted |
| - | `corner_test<D>` | `grid: &RectGrid<D>, point: [Px; D], bx: &BBox<D>, threshold: f64` | `(Option<[Parameter; D]>, Option<[Option<bool>; D]>)` | For a BBox with area, determines whether point is near an edge (within threshold) |
| - | `drag_resize<D>` | `grid: &RectGrid<D>, pointer: [Px; D], bx: &BBox<D>, corner: [Option<bool>; D]` | `Result<BBox<D>, RectgridError>` | Updates BBox's base/offset via a corner-handle drag |
| - | `drag_translate<D>` | `grid: &RectGrid<D>, pointer: [Px; D], drag_offset: [Px; D]` | `[Px; D]` | Computes base's px position during a move drag |
| - | `snap_region_to_unit<D>` | `grid: &RectGrid<D>, pointer: [Px; D], drag_offset: [Px; D], bx: &BBox<D>, extend: Option<[Unit; D]>` | `Result<BBox<D>, RectgridError>` | At DragEnd, snaps the move-drag result of a BBox with area to the Unit grid |
| - | `snap_point_to_unit<D>` | `grid: &RectGrid<D>, pointer: [Px; D], drag_offset: [Px; D], snap: [Unit; D]` | `Result<BBox<D>, RectgridError>` | At DragEnd, computes a BBox snapped to the Unit grid from the move-drag result of a point BBox |

## Internal ports

| Item | Port | Parameter | Return | Description |
|-|-|-|-|-|
| `RectGrid<D>` | `accumulator` | - | `[Box<dyn Fn(f64) -> Result<Px, RectgridError>>; D]` | f(Unit) -> Result<Px> per axis |
|               | `px_to_unit_axis` | `i: usize, target: Px` | `Result<Unit, RectgridError>` | Binary search inverting accumulator[i] |
|               | `contains` | `point: [Px; D], bx: &BBox<D>, extend: Option<([Unit; D], [Unit; D])>` | `(bool, [Px; D], [Px; D])` | Hit test backing hit_test/hit_tests/hit_test_with_parameter |
|               | `parameter_from_px` | `point: [Px; D], base_px: [Px; D], offset_px: [Px; D]` | `[Parameter; D]` | Shared by get_parameter/hit_test_with_parameter |

---

# Ja

- 各軸が独立した階差関数を持つ直交座標系(rectilinear grid)の、固有の原点座標と各軸の階差関数を与単位で定義した任意単位系(unit: 一般単位)の格子上で、2点間座標領域(box)とその集合(region)を操作するための幾何計算モジュール。さらに、単一のboxの、各軸のベクトル長を1とした局所単位系(parameter)への変換関数により、任意の幾何による境界判定を実装可能にする。

- 与単位系とは、原点の座標が(0,...,0), 全ての軸の階差関数が定数1を返す単位系を指す。単位名をPx(pixel: picture element)とする。

## 座標系

- rectgridモジュールのx, yを2D座標として扱う場合、xはviewportで幅になる軸、yは高さ方向、原点(0,0)は左上隅とする。
- モジュール外部から関数引数として渡されるpxはglobal(origin未補正の外部座標、例えばviewport座標)として受け取り、各関数の内部でoriginを差し引いてlocal化する。一方、box(base/offset)由来のpx(`unit_to_px`の戻り値や、それを使う`hit_test`系・`*_as_px`系の戻り値)は常にlocal(origin=0を基準とした座標)を返す。呼び出し側が`RectGrid`を跨いで再度渡す場合はlocal pxとして扱う。

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
| `BBox<D>` | `base` | - | `Point<D>` | 始点 |
|           | `offset` | - | `Point<D>` | 終点までのベクトル距離 |
|           | `snap_floor` | `extend: Option<[Unit; D]>` | `&mut Self` | base/offsetをfloor整数格子にスナップ。extendはbaseにのみfloor前に加算 |
|           | `has_size` | - | `bool` | 全軸のoffsetが非ゼロか(面積/体積を持つBBoxか) |
| `RectgridError` | `OutOfIndex` | `u32` | - | 範囲外アクセス。範囲内に収まる最後の有効index |
|                 | `InvalidDefinition` | - | - | 定義が不正で評価クロージャを構築できない |
| `IncrementFunction` | `ForwardDifference` | `Rc<dyn Fn(u32) -> Result<Px, RectgridError>>` | - | - |
|                     | `VectorList` | `Vec<Px>` | - | 空のVec<Px>は不正 |
|                     | `Scale` | `f64` | - | - |
|                     | `accumulate` | - | `Result<Box<dyn Fn(f64) -> Result<Px, RectgridError>>, RectgridError>` | - |
| `RectGrid<D>` | `origin` | - | `[Px; D]` | 始点 |
|               | `new`                 | `origin: [Px; D], definitions: [IncrementFunction; D]` | `Result<Self, RectgridError>` | - |
|               | `set_definition`      | `definition: IncrementFunction, d: usize` | `Result<(), RectgridError>` | d軸の定義を差し替える |
|               | `point_to_unit`       | `point: [Px; D]` | `[Result<Unit, RectgridError>; D]` | pxをunitへ数値的に逆変換(originを差し引いてから変換。accumulatorがUnit>=0で単調非減少である前提) |
|               | `unit_to_px`          | `d: usize, unit: &Unit` | `Result<Px, RectgridError>` | unit座標をpxへ変換(accumulatorをそのまま評価) |
|               | `point_as_px`         | `points: &Vec<Point<D>>` | `Vec<Result<[Px; D], RectgridError>>` | 複数のunit座標点をpxへ変換。評価不能な軸がある点はError |
|               | `box_as_px`           | `boxes: &Vec<BBox<D>>` | `Vec<Result<([Px; D], [Px; D]), RectgridError>>` | 複数のBBoxを(base_px, offset_px)へ変換。offset_pxはbase位置を踏まえた実際の辺の長さ(unit_to_px(base+offset) − unit_to_px(base))で、非線形なaccumulatorでも正しい長さになる。評価不能な軸があるboxはError |
|               | `hit_test`            | `point: [Px; D], boxes: &Vec<BBox<D>>, extend: Option<([Unit; D], [Unit; D])>` | `Option<usize>` | pointにhitするboxesのうちindex最大のものを返す |
|               | `hit_test_with_parameter` | `point: [Px; D], boxes: &Vec<BBox<D>>, extend: Option<([Unit; D], [Unit; D])>` | `Option<(usize, [Parameter; D])>` | hit_testと同様にindex最大のhitとget_parameter相当の値を返す |
|               | `hit_tests`           | `point: [Px; D], boxes: &Vec<BBox<D>>, extend: Option<([Unit; D], [Unit; D])>` | `Vec<bool>` | pointがhitするboxを全て走査し、boxesと同じ長さのhit有無を返す |
|               | `get_parameter`           | `point: [Px; D], bx: BBox<D>` | `[Parameter; D]` | 単一boxの各辺長を1とした符号付き局所座標 |
|               | `offset`              | `pointer: [Px; D], z: [Px; D]` | `[Px; D]` | pointerのlocal座標(origin補正後)からzを差し引いた値 |
| - | `corner_test<D>` | `grid: &RectGrid<D>, point: [Px; D], bx: &BBox<D>, threshold: f64` | `(Option<[Parameter; D]>, Option<[Option<bool>; D]>)` | 面積を持つBBoxに対しpointが辺付近(threshold未満)にあるかを判定 |
| - | `drag_resize<D>` | `grid: &RectGrid<D>, pointer: [Px; D], bx: &BBox<D>, corner: [Option<bool>; D]` | `Result<BBox<D>, RectgridError>` | 角ハンドルドラッグによってBBoxのbase/offsetを更新する |
| - | `drag_translate<D>` | `grid: &RectGrid<D>, pointer: [Px; D], drag_offset: [Px; D]` | `[Px; D]` | 移動ドラッグ中のbaseのpx位置を求める |
| - | `snap_region_to_unit<D>` | `grid: &RectGrid<D>, pointer: [Px; D], drag_offset: [Px; D], bx: &BBox<D>, extend: Option<[Unit; D]>` | `Result<BBox<D>, RectgridError>` | DragEnd時、面積を持つBBoxの移動ドラッグ結果をUnit格子にスナップする |
| - | `snap_point_to_unit<D>` | `grid: &RectGrid<D>, pointer: [Px; D], drag_offset: [Px; D], snap: [Unit; D]` | `Result<BBox<D>, RectgridError>` | DragEnd時、点BBoxの移動ドラッグ結果をUnit格子にスナップしたBBoxを求める |

## 内部ポート

| アイテム | ポート | 引数 | 戻り値 | 説明 |
|-|-|-|-|-|
| `RectGrid<D>` | `accumulator` | - | `[Box<dyn Fn(f64) -> Result<Px, RectgridError>>; D]` | 各軸のf(Unit) -> Result<Px> |
|               | `px_to_unit_axis` | `i: usize, target: Px` | `Result<Unit, RectgridError>` | accumulator[i]を二分探索で逆変換 |
|               | `contains` | `point: [Px; D], bx: &BBox<D>, extend: Option<([Unit; D], [Unit; D])>` | `(bool, [Px; D], [Px; D])` | hit_test/hit_tests/hit_test_with_parameterの共通判定 |
|               | `parameter_from_px` | `point: [Px; D], base_px: [Px; D], offset_px: [Px; D]` | `[Parameter; D]` | get_parameter/hit_test_with_parameterで共有 |