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
```

---

# Ja

- 各軸が独立した階差関数を持つ直交座標系(rectilinear grid: separable coordinates)の、固有の原点座標と各軸の階差関数を与単位で定義した任意単位系(unit: 一般単位)の格子上で、2点間座標領域(box)とその集合(region)を操作するための幾何計算モジュール。さらに、単一のboxの、各軸のベクトル長を1とした局所単位系(parameter)への変換関数により、任意の幾何による境界判定を実装可能にする。

- 与単位系とは、原点の座標が(0,...,0), 全ての軸の階差関数が定数1を返す単位系を指す。単位名をPx(pixel: picture element)とする。