<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="type-LatheGeometryDesc"></a>

### `LatheGeometryDesc` (desc struct)

A profile turned round +Y, as three.js LatheGeometry(points, segments, phiStart, phiLength): vases, bottles, columns, wheels. three.js's vertices, indices and smooth normals; u round the axis, v along the profile (flipped to run down the image). A whole turn's seam meets exactly.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `points` | `readonly Vector2[]` | `[]` | — | `m` | — | The profile, x the distance from the axis and y the height, in order (at least two). Without it, three.js's default (0, -0.5), (0.5, 0), (0, 0.5). |
| `segments` | `number` | `12` | ≥ 1 | — | — | Segments round the axis. |
| `phiStart` | `number` | `0` | — | `rad` | — | Where the turn starts, from +Z towards +X. |
| `phiLength` | `number` | `6.283185307179586` | — | `rad` | — | How far it turns; 2 pi is whole (clamped to it). |
| `uvRepeat` | `Vector2` | `{ x: 1, y: 1 }` | — | — | — | How many times a texture repeats across it, along u and v: its UVs scaled by uvRepeat (three.js texture.repeat, here on the geometry; material textures sample with repeat wrapping). |
| `physics` | `boolean` | `true` | — | — | — | Register it as a physics mesh too. |
| `dynamic` | `boolean` | `false` | — | — | — | Whether og.render.updateGeometry may rewrite it later. |

```ogt
struct LatheGeometryDesc {
  doc "A profile turned round +Y, as three.js LatheGeometry(points, segments, phiStart, phiLength): vases, bottles, columns, wheels. three.js's vertices, indices and smooth normals; u round the axis, v along the profile (flipped to run down the image). A whole turn's seam meets exactly."
  desc true
  field points List.Vector2 "The profile, x the distance from the axis and y the height, in order (at least two). Without it, three.js's default (0, -0.5), (0.5, 0), (0, 0.5)." {
    unit m
  }
  field segments u32 "Segments round the axis." {
    default 12
    min 1
  }
  field phiStart f32 "Where the turn starts, from +Z towards +X." {
    unit rad
  }
  field phiLength f32 "How far it turns; 2 pi is whole (clamped to it)." {
    default 6.283185307179586
    unit rad
  }
  field uvRepeat Vector2 "How many times a texture repeats across it, along u and v: its UVs scaled by uvRepeat (three.js texture.repeat, here on the geometry; material textures sample with repeat wrapping)." {
    default 1 1
  }
  field physics bool "Register it as a physics mesh too." {
    default true
  }
  field dynamic bool "Whether og.render.updateGeometry may rewrite it later."
}
```

<sub>Module `render` line 3220</sub>
