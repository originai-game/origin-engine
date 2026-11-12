<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`terrain`](../terrain.md) · [index](../index.md)

<a id="type-TerrainSample"></a>

### `TerrainSample` (struct)

The terrain under a point.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `height` | `number` | `0` | — | `m` | — | The world y of the surface (the triangles drawn at the finest level and collided with) on the terrain's up axis through (x, z): straight down for a terrain turned only about y. |
| `normal` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | — | — | The surface normal, smoothed between samples as the renderer shades it. |
| `slope` | `number` | `0` | — | `deg` | — | The normal's angle from straight up. |
| `inside` | `boolean` | `false` | — | — | — | Whether the point is over the terrain (outside, the nearest edge answers). |
| `hole` | `boolean` | `false` | — | — | — | Whether the point is over one of its holes: nothing collides there, and `height` is the surface the hole was cut from. |

```ogt
struct TerrainSample {
  doc "The terrain under a point."
  field height f32 "The world y of the surface (the triangles drawn at the finest level and collided with) on the terrain's up axis through (x, z): straight down for a terrain turned only about y." {
    unit m
  }
  field normal Vector3 "The surface normal, smoothed between samples as the renderer shades it."
  field slope f32 "The normal's angle from straight up." {
    unit deg
  }
  field inside bool "Whether the point is over the terrain (outside, the nearest edge answers)."
  field hole bool "Whether the point is over one of its holes: nothing collides there, and `height` is the surface the hole was cut from."
}
```

<sub>Module `terrain` line 461</sub>
