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
|           | `snap_floor` | `extend: Option<[Unit; D]>` | `&mut Self` | base/offsetをfloor整数格子にスナップ。extendはbaseにのみfloor前に加算 |
|           | `has_size` | - | `bool` | 全軸のoffsetが非ゼロか(面積/体積を持つBBoxか) |
| `RectgridError` | `OutOfIndex` | `u32` | - | 範囲外アクセス。範囲内に収まる最後の有効index |
|                 | `InvalidDefinition` | - | - | 定義が不正で評価クロージャを構築できない |
| `IncrementFunction` | `ForwardDifference` | `Rc<dyn Fn(u32) -> Result<Px, RectgridError>>` | - | - |
|                     | `VectorList` | `Vec<Px>` | - | - |
|                     | `Scale` | `f64` | - | - |
|                     | `accumulate` | - | `Result<Box<dyn Fn(f64) -> Result<Px, RectgridError>>, RectgridError>` | - |
| `RectGrid<D>` | `origin` | - | `[Px; D]` | フィールド |
|               | `new`                 | `origin: [Px; D], definitions: [IncrementFunction; D]` | `Result<Self, RectgridError>` | - |
|               | `set_definition`      | `definition: IncrementFunction, d: usize` | `Result<(), RectgridError>` | d軸の定義を差し替える |
|               | `point_to_unit`       | `point: [Px; D]` | `[Result<Unit, RectgridError>; D]` | pxをunitへ数値的に逆変換(originを差し引いてから変換。accumulatorがUnit>=0で単調非減少である前提) |
|               | `unit_to_px`          | `d: usize, unit: &Unit` | `Result<Px, RectgridError>` | unit座標をpxへ変換(accumulatorをそのまま評価) |
|               | `point_as_px`         | `points: &Vec<Point<D>>` | `Vec<Result<[Px; D], RectgridError>>` | 複数のunit座標点をpxへ変換。評価不能な軸がある点はErr |
|               | `box_as_px`           | `boxes: &Vec<BBox<D>>` | `Vec<Result<([Px; D], [Px; D]), RectgridError>>` | 複数のBBoxを(base_px, offset_px)へ変換。評価不能な軸があるboxはErr |
|               | `hit_test`            | `point: [Px; D], boxes: &Vec<BBox<D>>, extend: Option<([Unit; D], [Unit; D])>` | `Option<usize>` | pointにhitするboxesのうちindex最大のものを返す |
|               | `hit_test_with_ratio` | `point: [Px; D], boxes: &Vec<BBox<D>>, extend: Option<([Unit; D], [Unit; D])>` | `Option<(usize, [Parameter; D])>` | hit_testと同様にindex最大のhitとget_ratio相当の値を返す |
|               | `hit_tests`           | `point: [Px; D], boxes: &Vec<BBox<D>>, extend: Option<([Unit; D], [Unit; D])>` | `Vec<bool>` | pointがhitするboxを全て走査し、boxesと同じ長さのhit有無を返す |
|               | `get_ratio`           | `point: [Px; D], bx: BBox<D>` | `[Parameter; D]` | 単一boxの各辺長を1とした符号付き局所座標 |
|               | `offset`              | `pointer: [Px; D], z: [Px; D]` | `[Px; D]` | pointerのlocal座標(origin補正後)からzを差し引いた値 |

| Free function | Parameter | Return | Description |
|-|-|-|-|
| `corner_test<D>` | `grid: &RectGrid<D>, point: [Px; D], bx: &BBox<D>, threshold: f64` | `(Option<[Parameter; D]>, Option<[Option<bool>; D]>)` | 面積を持つBBoxに対しpointが辺付近(threshold未満)にあるかを判定 |
| `drag_resize<D>` | `grid: &RectGrid<D>, pointer: [Px; D], bx: &BBox<D>, corner: [Option<bool>; D]` | `Result<BBox<D>, RectgridError>` | 角ハンドルドラッグによってBBoxのbase/offsetを更新する |
| `drag_translate<D>` | `grid: &RectGrid<D>, pointer: [Px; D], drag_offset: [Px; D]` | `[Px; D]` | 移動ドラッグ中のbaseのpx位置を求める |
| `snap_region_to_unit<D>` | `grid: &RectGrid<D>, pointer: [Px; D], drag_offset: [Px; D], bx: &BBox<D>, extend: Option<[Unit; D]>` | `Result<BBox<D>, RectgridError>` | DragEnd時、面積を持つBBoxの移動ドラッグ結果をUnit格子にスナップする |
| `snap_point_to_unit<D>` | `grid: &RectGrid<D>, pointer: [Px; D], drag_offset: [Px; D], snap: [Unit; D]` | `Result<BBox<D>, RectgridError>` | DragEnd時、点BBoxの移動ドラッグ結果をUnit格子にスナップしたBBoxを求める |

---

# Ja

- 各軸が独立した階差関数を持つ直交座標系(rectilinear grid)の、固有の原点座標と各軸の階差関数を与単位で定義した任意単位系(unit: 一般単位)の格子上で、2点間座標領域(box)とその集合(region)を操作するための幾何計算モジュール。さらに、単一のboxの、各軸のベクトル長を1とした局所単位系(parameter)への変換関数により、任意の幾何による境界判定を実装可能にする。

- 与単位系とは、原点の座標が(0,...,0), 全ての軸の階差関数が定数1を返す単位系を指す。単位名をPx(pixel: picture element)とする。