# rectgrid

[![Crates.io](https://img.shields.io/crates/v/rectgrid.svg)](https://crates.io/crates/rectgrid)

Boundary box operations on rectilinear grids with arbitrary unit systems.

- A crate for operating on a two-point coordinate boundary box and its collections, defined over the grid of an arbitrary unit system — a rectilinear grid whose axes each have an independent increment function. Such a unit system is defined by an intrinsic origin and per-axis increment functions, both expressed in a base unit (unit: a general-purpose unit). It further provides, for any single boundary box, a conversion function into a local unit system (parameter) in which each axis has unit vector length, making it possible to implement boundary tests against arbitrary geometry.
- The base unit system is defined as the unit system whose origin lies at (0, ..., 0) and whose per-axis increment functions all return the constant 1. This base unit is named Px (pixel: picture element).

[English](#rectgrid) | [日本語](#ja)

---

## Version

| Version | Status    | Date       | Description |
|---------|-----------|------------|-------------|
| 0.1.0   | Released  | 2026-07-10 | 1st release |
| 0.1.1   | Released  | 2026-07-13 | improve performance(#7) |
| 0.2.0   | Released  | 2026-10-01 | improve BBox::new() |

This project adheres to [Semantic Versioning](https://semver.org/).

---

## Commands

```bash
# unit test
cargo test --all-features

# unit test (examples)
cd examples && cargo test

# wasm build (examples; main thread, imported memory)
cd examples
RUSTFLAGS="-Clink-arg=--import-memory -Clink-arg=--max-memory=134217728" \
cargo build --release --target wasm32-unknown-unknown
cargo install wasm-bindgen-cli --version "$(grep -A1 '^name = "wasm-bindgen"$' Cargo.lock | sed -n 's/^version = "\(.*\)"$/\1/p')" --locked
wasm-bindgen --target web --out-dir app --out-name app target/wasm32-unknown-unknown/release/app.wasm

# auto formatter
cargo +nightly fmt
```

---

## Features

| Feature | Default | Description |
|-|-|-|
| `geometry` | off | Enables the `rectgrid::geometry` module (`Line`, `Circle`, `Ellipse`, `Polygon`, `as_on_*`) |

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
| `RectgridError` | `OutOfIndex` | `u32` | - | Out-of-range access. The last valid index within range |
|                 | `InvalidDefinition` | - | - | The definition is invalid and an evaluation closure cannot be built |
| `StepFn` | `step` | `i: u32` | `Result<Px, RectgridError>` | Implemented for every `Deref` to `Fn(u32) -> Result<Px, RectgridError>` (`Rc`, `Arc`, `Box`, `&F`) |
| `DefaultSteps` | - | - | - | `Rc<dyn Fn(u32) -> Result<Px, RectgridError>>`; the `P` of a grid that does not name one |
| `IncrementFunction<P>` | `ForwardDifference` | `P` | - | `P` points to `Fn(u32) -> Result<Px, RectgridError>` (see `StepFn`); defaults to `Rc<dyn Fn(u32) -> Result<Px, RectgridError>>` |
|                     | `VectorList` | `Vec<Px>` | - | An empty `Vec<Px>` is invalid |
|                     | `Scale` | `f64` | - | - |
|                     | `accumulate` | `self` | `Result<Accumulator<P>, RectgridError>` | Builds a forward/inverse `Accumulator` from the definition |
| `Accumulator<P>` | `Scale` | `f64` | - | Inverse resolves analytically (`target / s`), no search needed |
|               | `VectorList` | `Vec<Px>` | - | Inverse resolves via `partition_point` + O(1) linear-interpolation solve |
|               | `ForwardDifference` | `P` | - | Inverse scans the segments |
|               | `forward` | `x: f64` | `Result<Px, RectgridError>` | unit coordinate -> px |
|               | `inverse` | `target: Px` | `Result<Unit, RectgridError>` | px -> unit coordinate |
| `RectGrid<D, P>` | `origin` | - | `[Px; D]` | Start point |
|               | `new`                 | `origin: [Px; D], definitions: [IncrementFunction<P>; D]` | `Result<Self, RectgridError>` | - |
|               | `set_definition`      | `definition: IncrementFunction<P>, d: usize` | `Result<(), RectgridError>` | Replaces the definition for axis d |
|               | `point_to_unit`       | `point: [Px; D]` | `[Result<Unit, RectgridError>; D]` | Inverts px to unit (origin subtracted first); the accumulator must be non-decreasing over Unit >= 0 |
|               | `unit_to_px`          | `d: usize, unit: &Unit` | `Result<Px, RectgridError>` | Converts a unit coordinate to px (evaluates the accumulator directly) |
|               | `point_as_px`         | `points: &[Point<D>]` | `Vec<[Result<Px, RectgridError>; D]>` | Converts unit points to px, one Result per axis |
|               | `box_as_px`           | `boxes: &[BBox<D>]` | `Vec<[Result<(Px, Px), RectgridError>; D]>` | Converts boundary boxes to (base_px, offset_px), one Result per axis |
|               | `hit_test`            | `point: [Px; D], boxes: &[BBox<D>], extend: Option<([Unit; D], [Unit; D])>` | `Option<usize>` | Returns the highest index among the boundary boxes point hits; an unevaluable boundary box never hits |
|               | `hit_test_with_parameter` | `point: [Px; D], boxes: &[BBox<D>], extend: Option<([Unit; D], [Unit; D])>` | `Option<(usize, [Parameter; D])>` | Like hit_test, returns the highest-index hit along with the get_parameter-equivalent value |
|               | `hit_tests`           | `point: [Px; D], boxes: &[BBox<D>], extend: Option<([Unit; D], [Unit; D])>` | `Vec<bool>` | Returns hit/no-hit for every boundary box, in a Vec of the same length |
|               | `get_parameter`           | `point: [Px; D], bx: BBox<D>` | `[Result<Parameter, RectgridError>; D]` | Signed local coordinate (side length normalized to 1), one Result per axis |
|               | `offset`              | `pointer: [Px; D], z: [Px; D]` | `[Px; D]` | pointer's local coordinate (after origin correction) with z subtracted |
| - | `corner_test<D, P>` | `grid: &RectGrid<D, P>, point: [Px; D], bx: &BBox<D>, threshold: f64, extend: Option<([Unit; D], [Unit; D])>` | `(Option<[Parameter; D]>, Option<[Option<bool>; D]>)` | For a boundary box with area, determines whether point is near an edge (within threshold) |
| - | `drag_resize<D, P>` | `grid: &RectGrid<D, P>, pointer: [Px; D], bx: &BBox<D>, corner: [Option<bool>; D]` | `Result<BBox<D>, RectgridError>` | Updates the boundary box's base/offset via a corner-handle drag |
| - | `drag_translate<D, P>` | `grid: &RectGrid<D, P>, pointer: [Px; D], drag_offset: [Px; D]` | `[Px; D]` | Computes base's px position during a move drag |
| - | `snap_bbox_to_unit<D, P>` | `grid: &RectGrid<D, P>, pointer: [Px; D], drag_offset: [Px; D], bx: &BBox<D>, extend: Option<[Unit; D]>` | `Result<BBox<D>, RectgridError>` | At DragEnd, snaps the move-drag result of a boundary box with area to the Unit grid |
| - | `snap_point_to_unit<D, P>` | `grid: &RectGrid<D, P>, pointer: [Px; D], drag_offset: [Px; D], snap: [Unit; D]` | `Result<BBox<D>, RectgridError>` | At DragEnd, computes a boundary box snapped to the Unit grid from the move-drag result of a boundary box without area (a point) |

## Internal ports

| Item | Port | Parameter | Return | Description |
|-|-|-|-|-|
| `RectGrid<D, P>` | `accumulator` | - | `[Accumulator<P>; D]` | Forward/inverse conversion per axis |
|               | `px_to_unit_axis` | `i: usize, target: Px` | `Result<Unit, RectgridError>` | Delegates to `accumulator[i].inverse` |
|               | `contains` | `point: [Px; D], bx: &BBox<D>, extend: Option<([Unit; D], [Unit; D])>` | `Option<(bool, [Px; D], [Px; D])>` | Hit test backing hit_test/hit_tests/hit_test_with_parameter; None if the boundary box is unevaluable |
|               | `unit_to_px_clipped` | `d: usize, unit: &Unit` | `Result<Px, RectgridError>` | unit_to_px for an extend edge, clipped at a finite domain end |
|               | `parameter_from_px` | `point: [Px; D], base_px: [Px; D], offset_px: [Px; D]` | `[Parameter; D]` | Shared by hit_test_with_parameter |
|               | `parameter_axis` | `point: Px, base_px: Px, far_px: Px` | `Parameter` | One axis of parameter_from_px, shared by get_parameter |

---

# Ja

- 各軸が独立した階差関数を持つ直交座標系(rectilinear grid)の、固有の原点座標と各軸の階差関数を与単位で定義した任意単位系(unit: 一般単位)の格子上で、2点間座標のboundary boxとその集合を操作するための幾何計算クレート。さらに、単一のboundary boxの、各軸のベクトル長を1とした局所単位系(parameter)への変換関数により、任意の幾何による境界判定を実装可能にする。

- 与単位系とは、原点の座標が(0,...,0), 全ての軸の階差関数が定数1を返す単位系を指す。単位名をPx(pixel: picture element)とする。

## Features

| Feature | Default | 説明 |
|-|-|-|
| `geometry` | off | `rectgrid::geometry`モジュール(`Line`, `Circle`, `Ellipse`, `Polygon`, `as_on_*`)を有効化する |

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
| `RectgridError` | `OutOfIndex` | `u32` | - | 範囲外アクセス。範囲内に収まる最後の有効index |
|                 | `InvalidDefinition` | - | - | 定義が不正で評価クロージャを構築できない |
| `StepFn` | `step` | `i: u32` | `Result<Px, RectgridError>` | `Fn(u32) -> Result<Px, RectgridError>`への`Deref`すべてに実装(`Rc`, `Arc`, `Box`, `&F`) |
| `DefaultSteps` | - | - | - | `Rc<dyn Fn(u32) -> Result<Px, RectgridError>>`。`P`を指定しない格子の`P` |
| `IncrementFunction<P>` | `ForwardDifference` | `P` | - | `P`は`Fn(u32) -> Result<Px, RectgridError>`へのポインタ(`StepFn`)。既定は`Rc<dyn Fn(u32) -> Result<Px, RectgridError>>` |
|                     | `VectorList` | `Vec<Px>` | - | 空のVec<Px>は不正 |
|                     | `Scale` | `f64` | - | - |
|                     | `accumulate` | `self` | `Result<Accumulator<P>, RectgridError>` | 定義から順変換・逆変換を持つ`Accumulator`を構築する |
| `Accumulator<P>` | `Scale` | `f64` | - | 逆変換は解析的(`target / s`)に即決、探索不要 |
|               | `VectorList` | `Vec<Px>` | - | 逆変換は`partition_point`と線形補間の逆算(O(1))で解決 |
|               | `ForwardDifference` | `P` | - | 逆変換は区間を走査して解く |
|               | `forward` | `x: f64` | `Result<Px, RectgridError>` | unit座標 -> px |
|               | `inverse` | `target: Px` | `Result<Unit, RectgridError>` | px -> unit座標 |
| `RectGrid<D, P>` | `origin` | - | `[Px; D]` | 始点 |
|               | `new`                 | `origin: [Px; D], definitions: [IncrementFunction<P>; D]` | `Result<Self, RectgridError>` | - |
|               | `set_definition`      | `definition: IncrementFunction<P>, d: usize` | `Result<(), RectgridError>` | d軸の定義を差し替える |
|               | `point_to_unit`       | `point: [Px; D]` | `[Result<Unit, RectgridError>; D]` | pxをunitへ逆変換(originを差し引いてから変換。accumulatorがUnit>=0で単調非減少である前提) |
|               | `unit_to_px`          | `d: usize, unit: &Unit` | `Result<Px, RectgridError>` | unit座標をpxへ変換(accumulatorをそのまま評価) |
|               | `point_as_px`         | `points: &[Point<D>]` | `Vec<[Result<Px, RectgridError>; D]>` | 複数のunit座標点をpxへ変換。軸ごとのResultを返す |
|               | `box_as_px`           | `boxes: &[BBox<D>]` | `Vec<[Result<(Px, Px), RectgridError>; D]>` | 複数のboundary boxを(base_px, offset_px)へ変換。軸ごとのResultを返す |
|               | `hit_test`            | `point: [Px; D], boxes: &[BBox<D>], extend: Option<([Unit; D], [Unit; D])>` | `Option<usize>` | pointにhitするboundary boxのうちindex最大のものを返す。評価不能なboundary boxはhitしない |
|               | `hit_test_with_parameter` | `point: [Px; D], boxes: &[BBox<D>], extend: Option<([Unit; D], [Unit; D])>` | `Option<(usize, [Parameter; D])>` | hit_testと同様にindex最大のhitとget_parameter相当の値を返す |
|               | `hit_tests`           | `point: [Px; D], boxes: &[BBox<D>], extend: Option<([Unit; D], [Unit; D])>` | `Vec<bool>` | 全てのboundary boxについてhit有無を、同じ長さのVecで返す |
|               | `get_parameter`           | `point: [Px; D], bx: BBox<D>` | `[Result<Parameter, RectgridError>; D]` | 単一のboundary boxの各辺長を1とした符号付き局所座標。軸ごとのResultを返す |
|               | `offset`              | `pointer: [Px; D], z: [Px; D]` | `[Px; D]` | pointerのlocal座標(origin補正後)からzを差し引いた値 |
| - | `corner_test<D, P>` | `grid: &RectGrid<D, P>, point: [Px; D], bx: &BBox<D>, threshold: f64, extend: Option<([Unit; D], [Unit; D])>` | `(Option<[Parameter; D]>, Option<[Option<bool>; D]>)` | 面積を持つboundary boxに対しpointが辺付近(threshold未満)にあるかを判定 |
| - | `drag_resize<D, P>` | `grid: &RectGrid<D, P>, pointer: [Px; D], bx: &BBox<D>, corner: [Option<bool>; D]` | `Result<BBox<D>, RectgridError>` | 角ハンドルドラッグによってboundary boxのbase/offsetを更新する |
| - | `drag_translate<D, P>` | `grid: &RectGrid<D, P>, pointer: [Px; D], drag_offset: [Px; D]` | `[Px; D]` | 移動ドラッグ中のbaseのpx位置を求める |
| - | `snap_bbox_to_unit<D, P>` | `grid: &RectGrid<D, P>, pointer: [Px; D], drag_offset: [Px; D], bx: &BBox<D>, extend: Option<[Unit; D]>` | `Result<BBox<D>, RectgridError>` | DragEnd時、面積を持つboundary boxの移動ドラッグ結果をUnit格子にスナップする |
| - | `snap_point_to_unit<D, P>` | `grid: &RectGrid<D, P>, pointer: [Px; D], drag_offset: [Px; D], snap: [Unit; D]` | `Result<BBox<D>, RectgridError>` | DragEnd時、面積を持たない(点の)boundary boxの移動ドラッグ結果をUnit格子にスナップしたboundary boxを求める |

## 内部ポート

| アイテム | ポート | 引数 | 戻り値 | 説明 |
|-|-|-|-|-|
| `RectGrid<D, P>` | `accumulator` | - | `[Accumulator<P>; D]` | 各軸の順変換・逆変換 |
|               | `px_to_unit_axis` | `i: usize, target: Px` | `Result<Unit, RectgridError>` | accumulator[i].inverseに委譲 |
|               | `contains` | `point: [Px; D], bx: &BBox<D>, extend: Option<([Unit; D], [Unit; D])>` | `Option<(bool, [Px; D], [Px; D])>` | hit_test/hit_tests/hit_test_with_parameterの共通判定。boundary boxが評価不能ならNone |
|               | `unit_to_px_clipped` | `d: usize, unit: &Unit` | `Result<Px, RectgridError>` | extend辺用のunit_to_px。有限の定義域の終端で切り詰める |
|               | `parameter_from_px` | `point: [Px; D], base_px: [Px; D], offset_px: [Px; D]` | `[Parameter; D]` | hit_test_with_parameterで使用 |
|               | `parameter_axis` | `point: Px, base_px: Px, far_px: Px` | `Parameter` | parameter_from_pxの1軸分。get_parameterで共有 |