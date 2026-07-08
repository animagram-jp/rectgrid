# rectgrid

Region operations on rectilinear grids with arbitrary unit systems.

- A geometry module for operating on a two-point coordinate region (a box) and its collections (a region), defined over the grid of an arbitrary unit system — a rectilinear grid whose axes each have an independent increment function. Such a unit system is defined by an intrinsic origin and per-axis difference functions, both expressed in a base unit (unit: a general-purpose unit). It further provides, for any single box, a conversion function into a local unit system (parameter) in which each axis has unit vector length, making it possible to implement boundary tests against arbitrary geometry.
- The base unit system is defined as the unit system whose origin lies at (0, ..., 0) and whose per-axis difference functions all return the constant 1. This base unit is named Px (pixel: picture element).

[English](#rectgrid) | [日本語](#ja)

---

## Version

| Version | Status    | Date       | Description |
|---------|-----------|------------|-------------|
| 0.1.0   | Scheduled | 2026-07-31 | 1st release |

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

## Public ports

| Module | Port | Parameter | Return | Description |
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
| `BBox<D>` | `base` | - | `Point<D>` | フィールド |
|           | `offset` | - | `Point<D>` | フィールド |
| `RectgridError` | `OutOfIndex` | `u32` | - | 範囲外アクセス。範囲内に収まる最後の有効index |
|                 | `InvalidDefinition` | - | - | 定義が不正で評価クロージャを構築できない |
| `IncrementFunction` | `ForwardDifference` | `Rc<dyn Fn(u32) -> Result<Px, RectgridError>>` | - | - |
|                     | `VectorList` | `Vec<Px>` | - | - |
|                     | `Scale` | `f64` | - | - |
|                     | `accumulate` | - | `Result<Box<dyn Fn(f64) -> Result<Px, RectgridError>>, RectgridError>` | - |
| `RectGrid` | `origin` | - | `[Px; D]` | フィールド |
|            | `new`            | `origin: [Px; D], definitions: [IncrementFunction; D]` | `Result<Self, RectgridError>` | - |
|            | `set_definition` | `definition: IncrementFunction, d: usize` | `Result<(), RectgridError>` | - |
|            | `point_to_unit`  | `point: [Px; D]` | `[Result<Unit, RectgridError>; D]` | pxをunitへ数値的に逆変換(originを差し引いてから変換。accumulatorがUnit>=0で単調非減少である前提) |
|            | `point_as_px`    | `points: &Vec<Point<D>>` | `Vec<[Px; D]>` | - |
|            | `box_as_px`      | `boxes: &Vec<BBox<D>>` | `Vec<([Px; D], [Px; D])>` | - |
|            | `get_ratio`      | `point: [Px; D], bx: BBox<D>` | `[Parameter; D]` | 単一boxの各辺長を1とした符号付き局所座標 |
|            | `as_px`          | `boxes: &Vec<BBox<D>>` | `Vec<Result<([Px; D], [Px; D]), RectgridError>>` | - |

---

## Todo

Ports present in `examples/src/ugrid.rs` (prior prototype) not yet ported to `src/rectgrid.rs`.

| Module | Port | Parameter | Return | Description |
|-|-|-|-|-|
| `BBox<D>` | `snap_floor` | `extend: Option<[Unit; D]>` | `&mut Self` | base,offset(Point<D>ベース)のUnit成分をfloorして格子にスナップ |
|           | `has_size` | - | `bool` | offsetのいずれかの軸が非ゼロUnitか |
| `RectGrid` | `set_origin` | `origin: [Px; D]` | - | originの更新 |
|            | `drag_offset` | `pointer: [Px; D], base_px: [Px; D]` | `[Px; D]` | ドラッグ開始オフセットを求める |
|            | `drag_move` | `pointer: [Px; D], offset: [Px; D]` | `[Px; D]` | オフセットから現在位置を求める |
|            | `judge` | `coord: [Px; D], boxes: &[BBox<D>], extend: Option<([Unit; D], [Unit; D])>, detail: bool` | `Result<Vec<Option<JudgeResult<D>>>, RectgridError>` | coordが各BBoxに内包されるか判定 |
| `JudgeResult` | - | - | - | `Hit` または `Ratio([f64; D])` |
| - | `hit_test` | `rectgrid: &mut RectGrid<D>, coord: [Px; D], boxes: &[BBox<D>], extend: Option<([Unit; D], [Unit; D])>` | `Option<usize>` | 最前面からhitするBBoxのindexを返す |
| - | `corner_test` | `rectgrid: &mut RectGrid<D>, coord: [Px; D], bx: &BBox<D>, extend: Option<([Unit; D], [Unit; D])>, threshold: f64` | `(Option<[f64; D]>, Option<[Option<bool>; D]>)` | 辺/角(リサイズハンドル)判定 |
| - | `pointer_down_offset` | `rectgrid: &mut RectGrid<D>, coord: [Px; D], bx: &BBox<D>` | `Result<[Px; D], RectgridError>` | PointerDown時のドラッグオフセットを求める |
| - | `drag_resize` | `rectgrid: &mut RectGrid<D>, pointer: [Px; D], bx: &BBox<D>, corner: [Option<bool>; D]` | `Result<(BBox<D>, [Px; D], [Px; D]), RectgridError>` | 角ハンドルドラッグでBBoxを更新 |
| - | `drag_translate` | `rectgrid: &mut RectGrid<D>, pointer: [Px; D], drag_offset: [Px; D], bx: &BBox<D>` | `(BBox<D>, [Px; D])` | 移動ドラッグでBBoxのbaseを更新 |
| - | `snap_region_to_unit` | `rectgrid: &mut RectGrid<D>, bx: &BBox<D>, extend: Option<[Unit; D]>` | `Result<(BBox<D>, [Px; D]), RectgridError>` | 面積持ちBBoxをUnit格子にスナップ |
| - | `snap_point_to_unit` | `rectgrid: &mut RectGrid<D>, drag_px: [Px; D], snap: [Unit; D]` | `Result<(BBox<D>, [Px; D]), RectgridError>` | 点BBoxをUnit格子にスナップ |

---

# Ja

- 各軸が独立した階差関数を持つ直交座標系(rectilinear grid)の、固有の原点座標と各軸の階差関数を与単位で定義した任意単位系(unit: 一般単位)の格子上で、2点間座標領域(box)とその集合(region)を操作するための幾何計算モジュール。さらに、単一のboxの、各軸のベクトル長を1とした局所単位系(parameter)への変換関数により、任意の幾何による境界判定を実装可能にする。

- 与単位系とは、原点の座標が(0,...,0), 全ての軸の階差関数が定数1を返す単位系を指す。単位名をPx(pixel: picture element)とする。