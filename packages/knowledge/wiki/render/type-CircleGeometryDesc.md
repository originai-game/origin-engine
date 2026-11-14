<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="type-CircleGeometryDesc"></a>

### `CircleGeometryDesc` (desc struct)

A disc (or a sector) in the xy plane facing +Z, as three.js CircleGeometry(radius, segments, thetaStart, thetaLength): a 2D polygon's simplest shape.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `radius` | `number` | `1` | ≥ 0 | `m` | — | The radius. |
| `segments` | `number` | `32` | ≥ 3 | — | — | Segments around. |
| `thetaStart` | `number` | `0` | — | `rad` | — | Where it starts, from +X towards +Y. |
| `thetaLength` | `number` | `6.283185307179586` | — | `rad` | — | How far around it goes. |
| `uvRepeat` | `Vector2` | `{ x: 1, y: 1 }` | — | — | — | How many times a texture repeats across it, along u and v: its UVs scaled by uvRepeat (three.js texture.repeat, here on the geometry; material textures sample with repeat wrapping). |

```ogt
struct CircleGeometryDesc {
  doc "A disc (or a sector) in the xy plane facing +Z, as three.js CircleGeometry(radius, segments, thetaStart, thetaLength): a 2D polygon's simplest shape."
  desc true
  field radius f32 "The radius." {
    default 1
    min 0
    unit m
  }
  field segments u32 "Segments around." {
    default 32
    min 3
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

<sub>Module `render` line 3345</sub>
