<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="type-PlaneGeometryDesc"></a>

### `PlaneGeometryDesc` (desc struct)

A rectangle in the xy plane facing +z, centred at the origin, as three.js PlaneGeometry(width, height); turn it -pi/2 about x for a floor.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `width` | `number` | `1` | ≥ 0 | `m` | — | Its extent along x. |
| `height` | `number` | `1` | ≥ 0 | `m` | — | Its extent along y. |
| `uvRepeat` | `Vector2` | `{ x: 1, y: 1 }` | — | — | — | How many times a texture repeats across it, along u and v: its UVs run 0 to uvRepeat instead of 0 to 1 (three.js texture.repeat, here on the geometry; material textures sample with repeat wrapping). A ground grid every 10 m on a 1,600 m plane is uvRepeat 160 160. |

```ogt
struct PlaneGeometryDesc {
  doc "A rectangle in the xy plane facing +z, centred at the origin, as three.js PlaneGeometry(width, height); turn it -pi/2 about x for a floor."
  desc true
  field width f32 "Its extent along x." {
    default 1
    min 0
    unit m
  }
  field height f32 "Its extent along y." {
    default 1
    min 0
    unit m
  }
  field uvRepeat Vector2 "How many times a texture repeats across it, along u and v: its UVs run 0 to uvRepeat instead of 0 to 1 (three.js texture.repeat, here on the geometry; material textures sample with repeat wrapping). A ground grid every 10 m on a 1,600 m plane is uvRepeat 160 160." {
    default 1 1
  }
}
```

<sub>Module `render` line 2695</sub>
