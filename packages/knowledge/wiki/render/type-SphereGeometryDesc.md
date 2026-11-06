<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="type-SphereGeometryDesc"></a>

### `SphereGeometryDesc` (desc struct)

A UV sphere centred at the origin, as three.js SphereGeometry(radius, widthSegments, heightSegments).

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `radius` | `number` | `1` | ≥ 0 | `m` | — | Its radius. |
| `widthSegments` | `number` | `32` | ≥ 3 | — | — | Segments around the equator. |
| `heightSegments` | `number` | `16` | ≥ 2 | — | — | Rings from pole to pole. |
| `uvRepeat` | `Vector2` | `{ x: 1, y: 1 }` | — | — | — | How many times a texture repeats across it, along u and v: its UVs run 0 to uvRepeat instead of 0 to 1 (three.js texture.repeat, here on the geometry; material textures sample with repeat wrapping). A ground grid every 10 m on a 1,600 m plane is uvRepeat 160 160. |

```ogt
struct SphereGeometryDesc {
  doc "A UV sphere centred at the origin, as three.js SphereGeometry(radius, widthSegments, heightSegments)."
  desc true
  field radius f32 "Its radius." {
    default 1
    min 0
    unit m
  }
  field widthSegments u32 "Segments around the equator." {
    default 32
    min 3
  }
  field heightSegments u32 "Rings from pole to pole." {
    default 16
    min 2
  }
  field uvRepeat Vector2 "How many times a texture repeats across it, along u and v: its UVs run 0 to uvRepeat instead of 0 to 1 (three.js texture.repeat, here on the geometry; material textures sample with repeat wrapping). A ground grid every 10 m on a 1,600 m plane is uvRepeat 160 160." {
    default 1 1
  }
}
```

<sub>`api.ogt` line 6316</sub>
