<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="type-CapsuleGeometryDesc"></a>

### `CapsuleGeometryDesc` (desc struct)

A capsule along +Y centred at the origin, as three.js CapsuleGeometry(radius, height, capSegments, radialSegments, heightSegments): a cylinder of `height` between two hemispheres, `height + 2 radius` long in all (an og.Collider capsule of halfHeight height / 2 and that radius matches it).

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `radius` | `number` | `1` | ≥ 0 | `m` | — | The radius. |
| `height` | `number` | `1` | ≥ 0 | `m` | — | The straight middle's length. |
| `capSegments` | `number` | `4` | ≥ 1 | — | — | Rows in each hemisphere. |
| `radialSegments` | `number` | `8` | ≥ 3 | — | — | Segments around. |
| `heightSegments` | `number` | `1` | ≥ 1 | — | — | Rows along the middle. |
| `uvRepeat` | `Vector2` | `{ x: 1, y: 1 }` | — | — | — | How many times a texture repeats across it, along u and v: its UVs scaled by uvRepeat (three.js texture.repeat, here on the geometry; material textures sample with repeat wrapping). |

```ogt
struct CapsuleGeometryDesc {
  doc "A capsule along +Y centred at the origin, as three.js CapsuleGeometry(radius, height, capSegments, radialSegments, heightSegments): a cylinder of `height` between two hemispheres, `height + 2 radius` long in all (an og.Collider capsule of halfHeight height / 2 and that radius matches it)."
  desc true
  field radius f32 "The radius." {
    default 1
    min 0
    unit m
  }
  field height f32 "The straight middle's length." {
    default 1
    min 0
    unit m
  }
  field capSegments u32 "Rows in each hemisphere." {
    default 4
    min 1
  }
  field radialSegments u32 "Segments around." {
    default 8
    min 3
  }
  field heightSegments u32 "Rows along the middle." {
    default 1
    min 1
  }
  field uvRepeat Vector2 "How many times a texture repeats across it, along u and v: its UVs scaled by uvRepeat (three.js texture.repeat, here on the geometry; material textures sample with repeat wrapping)." {
    default 1 1
  }
}
```

<sub>Module `render` line 3218</sub>
