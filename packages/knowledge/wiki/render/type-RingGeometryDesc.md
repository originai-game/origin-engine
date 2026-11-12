<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="type-RingGeometryDesc"></a>

### `RingGeometryDesc` (desc struct)

A flat ring in the xy plane facing +Z, as three.js RingGeometry(innerRadius, outerRadius, thetaSegments, phiSegments, thetaStart, thetaLength).

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `innerRadius` | `number` | `0.5` | ≥ 0 | `m` | — | The hole's radius. |
| `outerRadius` | `number` | `1` | ≥ 0 | `m` | — | The ring's radius. |
| `thetaSegments` | `number` | `32` | ≥ 3 | — | — | Segments around. |
| `phiSegments` | `number` | `1` | ≥ 1 | — | — | Rings from the inside out. |
| `thetaStart` | `number` | `0` | — | `rad` | — | Where it starts, from +X towards +Y. |
| `thetaLength` | `number` | `6.283185307179586` | — | `rad` | — | How far around it goes. |
| `uvRepeat` | `Vector2` | `{ x: 1, y: 1 }` | — | — | — | How many times a texture repeats across it, along u and v: its UVs scaled by uvRepeat (three.js texture.repeat, here on the geometry; material textures sample with repeat wrapping). |

```ogt
struct RingGeometryDesc {
  doc "A flat ring in the xy plane facing +Z, as three.js RingGeometry(innerRadius, outerRadius, thetaSegments, phiSegments, thetaStart, thetaLength)."
  desc true
  field innerRadius f32 "The hole's radius." {
    default 0.5
    min 0
    unit m
  }
  field outerRadius f32 "The ring's radius." {
    default 1
    min 0
    unit m
  }
  field thetaSegments u32 "Segments around." {
    default 32
    min 3
  }
  field phiSegments u32 "Rings from the inside out." {
    default 1
    min 1
  }
  field thetaStart f32 "Where it starts, from +X towards +Y." {
    unit rad
  }
  field thetaLength f32 "How far around it goes." {
    default 6.283185307179586
    unit rad
  }
  field uvRepeat Vector2 "How many times a texture repeats across it, along u and v: its UVs scaled by uvRepeat (three.js texture.repeat, here on the geometry; material textures sample with repeat wrapping)." {
    default 1 1
  }
}
```

<sub>Module `render` line 3263</sub>
