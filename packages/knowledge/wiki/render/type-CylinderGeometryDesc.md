<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="type-CylinderGeometryDesc"></a>

### `CylinderGeometryDesc` (desc struct)

A cylinder (or a truncated cone) along +Y centred at the origin, as three.js CylinderGeometry(radiusTop, radiusBottom, height, radialSegments, heightSegments, openEnded, thetaStart, thetaLength).

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `radiusTop` | `number` | `1` | ≥ 0 | `m` | — | The radius at the top. |
| `radiusBottom` | `number` | `1` | ≥ 0 | `m` | — | The radius at the bottom. |
| `height` | `number` | `1` | ≥ 0 | `m` | — | Its extent along y. |
| `radialSegments` | `number` | `32` | ≥ 3 | — | — | Segments around. |
| `heightSegments` | `number` | `1` | ≥ 1 | — | — | Rows along the height. |
| `openEnded` | `boolean` | `false` | — | — | — | Leave the ends open (no caps). |
| `thetaStart` | `number` | `0` | — | `rad` | — | Where the side starts around y, from +Z towards +X. |
| `thetaLength` | `number` | `6.283185307179586` | — | `rad` | — | How far around the side goes; 2 pi is whole. |
| `uvRepeat` | `Vector2` | `{ x: 1, y: 1 }` | — | — | — | How many times a texture repeats across it, along u and v: its UVs scaled by uvRepeat (three.js texture.repeat, here on the geometry; material textures sample with repeat wrapping). |

```ogt
struct CylinderGeometryDesc {
  doc "A cylinder (or a truncated cone) along +Y centred at the origin, as three.js CylinderGeometry(radiusTop, radiusBottom, height, radialSegments, heightSegments, openEnded, thetaStart, thetaLength)."
  desc true
  field radiusTop f32 "The radius at the top." {
    default 1
    min 0
    unit m
  }
  field radiusBottom f32 "The radius at the bottom." {
    default 1
    min 0
    unit m
  }
  field height f32 "Its extent along y." {
    default 1
    min 0
    unit m
  }
  field radialSegments u32 "Segments around." {
    default 32
    min 3
  }
  field heightSegments u32 "Rows along the height." {
    default 1
    min 1
  }
  field openEnded bool "Leave the ends open (no caps)."
  field thetaStart f32 "Where the side starts around y, from +Z towards +X." {
    unit rad
  }
  field thetaLength f32 "How far around the side goes; 2 pi is whole." {
    default 6.283185307179586
    unit rad
  }
  field uvRepeat Vector2 "How many times a texture repeats across it, along u and v: its UVs scaled by uvRepeat (three.js texture.repeat, here on the geometry; material textures sample with repeat wrapping)." {
    default 1 1
  }
}
```

<sub>Module `render` line 2925</sub>
