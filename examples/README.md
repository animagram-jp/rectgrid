// This file includes untranslated text (ja).

# Rectgrid

## Rule

- [common for projects](./docs/Rule.md)

## Commands

```bash
# unit test
cargo test

# wasm-pack compile
wasm-pack build --target web --out-dir examples/app --out-name app
```

## Modules

| Module              | Port           | Description |
|-|-|-|
| Px                  | -              | type Px = f64 |
| Unit                | -              | type Unit = f64 |
| Length              | -              | enum { Px(Px), Unit(Unit) } |
| Region\<D\>         | -              | struct { base: [Length; D], offset: [Length; D] } |
| DefiningExpression  | -              | enum { ForwardDifference(Fn(u32) -> Result<Px, OutOfIndex>), VectorList(Vec<Px>), Scale(f64) } |
| DefiningExpression  | build          | -> Box<dyn Fn(f64) -> Result<Px, OutOfIndex>> |
| Rectgrid\<D\>          | new            | origin: [Px; D], expressions: [DefiningExpression; D] |
| Rectgrid\<D\>          | eval           | axis: usize, x: f64 -> Result<Px, OutOfIndex> |
| Rectgrid\<D\>          | unit_length    | axis: usize, l: &Length -> Result<Px, OutOfIndex> |
| Rectgrid\<D\>          | unit_point     | point: &[Length; D] -> Result<[Px; D], OutOfIndex> |
| Rectgrid\<D\>          | update         | regions: Vec<Region\<D\>> -> Result<Vec<([Px; D], [Px; D])>, OutOfIndex> |
| Rectgrid\<D\>          | set_expression | axis: usize, DefiningExpression |
| Rectgrid\<D\>          | judge          | coord: [Px; D], regions: &[Region\<D\>], extend: Option<([Unit; D], [Unit; D])>, detail: bool -> Result<Vec<Option<JudgeResult\<D\>>>, OutOfIndex> |
| JudgeResult\<D\>    | -              | enum { Hit, Ratio([f64; D]) } |
| Region\<D\>         | snap_floor     | extend: Option<[Unit; D]> -> &mut Self |

## Details

### Px, Unit

- Px: alias type of f64 means length unit in the BASE one of two Cartesian coordinate.
- Unit: alias type of f64 means length unit in DERIVED one.

### DefiningExpression

- DefiningExpression::ForwardDifference(Fn(u32) -> Result<Px, OutOfIndex>)
- DefiningExpression::VectorList(Vec<Px>)
- DefiningExpression::Scale(f64)

- DefiningExpression::build
    - ForwardDifference:
        - 端数は線形補間でpxに変換する
    -

### Rectgrid::judge

- Region配列に対して、Pointが範囲内にあるかを判定して同長の配列を戻す

- extend: Option<([Unit; D], [Unit; D])>
    - 判定時のbase, offsetに加算するオプション値。

- JudgeResult::Ratio([f64; D])
    - base(左上隅)を[0.0; D], 右下隅を[1.0; D]とした、符号ありの割合値。
    - offsetのpx値に対するカーソル位置px値の機械的除算値。
    - extendの作用は無視する。

---

## 修正中スケッチ(rectgrid)

| Module | Port | Parameter | Return | Description |
|-|-|-|-|-|
| IncrementFunction | as_px | f64 | Result<Px, OutOfBoundary> | 評価式
| IncrementFunction | - | - | 階差関数。DiscreteDifference, VectorList, Scaleを擁する列挙型 |
|                   |  | - |
| DiscreteDifference | - | i32 | Result<Px, OutOfBoundary> | f(i) = P(i+1) - P(i) の離散式 |
| VectorList | - | - | - | 原点から正方向に間隔の与単位値を列挙した配列。`Vec<Px>` |
| Scale | - | - | - |  間隔の与単位の単一定数定義。`f64` |
| Px   | - | - | - | 与単位名。f64のタイプエイリアス |
| Rectgrid | `<D>` | - | 次元数。usizeの定数ジェネリクス |
| Rectgrid | origin | [Px; D] | - | 原点座標の与単位定義。呼び出し側による更新のため公開 |
| Rectgrid | new   | origin: [Px; D], definition: [IncrementFunction; D] | Self | ランタイム上で任意単位系定義を作成 |
| Rectgrid | as_px | `&[Box<D>; U]` | `[(Result<([Px; D], [Px; D]), OutOfBoundary>; U]` | box群の与単位座標を算出 |
| Unit | - | - | - | 任意単位系の単位名。f64のタイプエイリアス |

- Px = f64
    - px_grid = RectGrid<const D: usize>::new(origin: [0.0; D], increment_functions: [Fn(u32) {Ok(1.0)}; D])
- IncrementFunction = Box(Fn(unit:u32) -> Result<Px, OutOfIndex>)

- Parametric  = f64

- Cell\<D\>::parametric(point) ->

`ξ_d = (P_d − P1_d) / (P2_d − P1_d)`

Regionの操作関数群を提供する。座標はpx(画面表示の最小単位、常に一意なf64)と、unit(軸ごとに定義された式によって解決される、可変幅もありうる目盛り)の2種類で表現される。

Regionは座標上の2点(base, offset)で一つに定まる領域を指し、px/unitいずれの単位でも同じ操作関数を通して扱える。unitはその軸のDefiningExpressionによって都度pxへ解決され、端数(セル内部の位置)は線形補間によってpxへ変換される。