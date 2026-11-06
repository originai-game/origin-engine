<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`nav`](../nav.md) · [index](../index.md)

<a id="type-NavGridDesc"></a>

### `NavGridDesc` (desc struct)

A grid of square cells over the plane: cell (0, 0)'s lower corner at `origin`, rows running up.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `width` | `number` | `32` | 1 … 4096 | — | — | Cells across. |
| `height` | `number` | `32` | 1 … 4096 | — | — | Cells up. |
| `cellSize` | `number` | `1` | 0.01 … 100 | `m` | — | A cell's side. |
| `origin` | `Vector2` | `{ x: 0, y: 0 }` | — | `m` | — | Where cell (0, 0)'s lower corner is (x, y in 2D; x, z in 3D). |
| `costs` | `readonly number[]` | `[]` | — | — | — | Row-major cell costs from row 0 (0 blocked, 1 normal, up to 255 to avoid); empty: every cell 1. |

```ogt
struct NavGridDesc {
  doc "A grid of square cells over the plane: cell (0, 0)'s lower corner at `origin`, rows running up."
  desc true
  field width u32 "Cells across." {
    default 32
    min 1
    max 4096
  }
  field height u32 "Cells up." {
    default 32
    min 1
    max 4096
  }
  field cellSize f32 "A cell's side." {
    default 1
    min 0.01
    max 100
    unit m
  }
  field origin Vector2 "Where cell (0, 0)'s lower corner is (x, y in 2D; x, z in 3D)." {
    unit m
  }
  field costs List.u32 "Row-major cell costs from row 0 (0 blocked, 1 normal, up to 255 to avoid); empty: every cell 1."
}
```

<sub>`api.ogt` line 13868</sub>
