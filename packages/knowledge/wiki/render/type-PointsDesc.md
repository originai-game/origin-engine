<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="type-PointsDesc"></a>

### `PointsDesc` (desc struct)

A point cloud drawn this frame only (immediate mode), as three.js Points with a PointsMaterial: a square per point facing the camera. Submit it every tick it should show (og.Points keeps a static set).

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `points` | `readonly Vector3[]` | `[]` | — | `m` | — | The positions: world, or with screen CSS pixels from the top left, y down. |
| `colors` | `readonly Color[]` | `[]` | — | — | — | Per point linear colours; empty is `color` for all. |
| `color` | `Color` | `{ r: 1, g: 1, b: 1 }` | — | — | — | The linear colour. |
| `opacity` | `number` | `1` | 0 … 1 | — | — | The alpha. |
| `size` | `number` | `1` | ≥ 0 | — | — | The side of each square: world units with sizeAttenuation, else CSS pixels. |
| `sizeAttenuation` | `boolean` | `true` | — | — | — | Size in world units, smaller with distance, as three.js PointsMaterial.sizeAttenuation; false keeps every point `size` CSS pixels. |
| `depthTest` | `boolean` | `true` | — | — | — | Hidden behind nearer geometry; false draws them over the scene. |
| `screen` | `boolean` | `false` | — | — | — | Draw in screen space over the finished frame, positions and size in CSS pixels. |

```ogt
struct PointsDesc {
  doc "A point cloud drawn this frame only (immediate mode), as three.js Points with a PointsMaterial: a square per point facing the camera. Submit it every tick it should show (og.Points keeps a static set)."
  desc true
  field points List.Vector3 "The positions: world, or with screen CSS pixels from the top left, y down." {
    unit m
  }
  field colors List.Color "Per point linear colours; empty is `color` for all."
  field color Color "The linear colour." {
    default 1 1 1
  }
  field opacity f32 "The alpha." {
    default 1
    min 0
    max 1
  }
  field size f32 "The side of each square: world units with sizeAttenuation, else CSS pixels." {
    default 1
    min 0
  }
  field sizeAttenuation bool "Size in world units, smaller with distance, as three.js PointsMaterial.sizeAttenuation; false keeps every point `size` CSS pixels." {
    default true
  }
  field depthTest bool "Hidden behind nearer geometry; false draws them over the scene." {
    default true
  }
  field screen bool "Draw in screen space over the finished frame, positions and size in CSS pixels."
}
```

<sub>Module `render` line 3516</sub>
