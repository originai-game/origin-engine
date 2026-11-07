<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="type-ConeGeometryDesc"></a>

### `ConeGeometryDesc` (desc struct)

A cone along +Y centred at the origin, its tip up, as three.js ConeGeometry(radius, height, radialSegments, heightSegments, openEnded).

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `radius` | `number` | `1` | ≥ 0 | `m` | — | The base's radius. |
| `height` | `number` | `1` | ≥ 0 | `m` | — | Its extent along y. |
| `radialSegments` | `number` | `32` | ≥ 3 | — | — | Segments around. |
| `heightSegments` | `number` | `1` | ≥ 1 | — | — | Rows along the height. |
| `openEnded` | `boolean` | `false` | — | — | — | Leave the base open. |

```ogt
struct ConeGeometryDesc {
  doc "A cone along +Y centred at the origin, its tip up, as three.js ConeGeometry(radius, height, radialSegments, heightSegments, openEnded)."
  desc true
  field radius f32 "The base's radius." {
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
  field openEnded bool "Leave the base open."
}
```

<sub>Module `render` line 2950</sub>
