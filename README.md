# rectgrid

[![Crates.io](https://img.shields.io/crates/v/rectgrid.svg)](https://crates.io/crates/rectgrid)
[![docs.rs](https://img.shields.io/docsrs/rectgrid)](https://docs.rs/rectgrid)

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
| 0.4.1   | Released  | 2026-10-15 | Improved docComment. |

This project adheres to [Semantic Versioning](https://semver.org/).

---

## Coordinate system

- When treating rectgrid's x and y as 2D coordinates, x is the axis that becomes the width in the viewport, y is the height direction, and the origin (0,0) is the top-left corner.
- Px passed into a function from outside this crate is global (an external coordinate not yet corrected for origin, e.g. a viewport coordinate); each function subtracts origin internally to make it local. Px derived from a boundary box (base/offset) — the return value of `unit_to_px`, and anything built on it such as `hit_test`/`*_as_px` results — is always local (origin=0 as the reference). If such a value is passed back across a `RectGrid` boundary, treat it as local px.

---

## Public ports

The reference for every public item, with its contract and a runnable example, is the crate documentation: https://docs.rs/rectgrid

| Module | Item |
|-|-|
| `rectgrid` | `Value<Tag>`, `PxTag`, `UnitTag`, `ParameterTag`, `Px`, `Unit`, `Parameter`, `Point<D>` |
|            | `BBox<D>`, `Error`, `StepFn`, `DefaultSteps`, `IncrementFunction<P>`, `Accumulator<P>`, `RectGrid<D, P>` |
|            | `corner_test`, `drag_resize`, `drag_translate`, `snap_bbox_to_unit`, `snap_point_to_unit` |
| `geometry` | `Line<D>`, `Circle<D>`, `Ellipse<D>`, `Polygon<D>`, `PointOnGeometry<D>` |
|            | `as_on_line`, `as_on_circle`, `as_on_ellipse`, `as_on_polygon` |

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

各公開アイテムの契約と実行可能な例は、クレートドキュメント(https://docs.rs/rectgrid)を参照。アイテムの一覧は[Public ports](#public-ports)にある。
