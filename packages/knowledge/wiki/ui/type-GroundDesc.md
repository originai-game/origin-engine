<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-GroundDesc"></a>

### `GroundDesc` (desc struct)

What a ground shape (a selection ring, a telegraph, a path line, a placement ghost) lies on: its outline is cut into vertices `spacing` apart and each is lifted to the surface under it, so it follows a slope or terrain instead of floating over or cutting into it. With neither a terrain nor colliders it lies flat at its points' own y.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `terrain` | `TerrainData` | `0` | — | — | — | A terrain sampled under each vertex (og.terrain.sample); 0 is none. |
| `origin` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | — | — | Where the terrain stands: its og.Terrain entity's world position. |
| `quaternion` | `Quaternion` | `{ x: 0, y: 0, z: 0, w: 1 }` | — | — | — | How the terrain is turned (the zero value: along world x and z). |
| `scale` | `Vector3` | `{ x: 1, y: 1, z: 1 }` | — | — | — | How the terrain is scaled. |
| `colliders` | `boolean` | `false` | — | — | — | Also cast straight down against colliders (og.physics.castRay, as the tick began); the highest surface of the two wins. Exclude the unit itself (exclude) or keep units off mask's layers, or the ring climbs onto it. |
| `mask` | `number` | `0` | — | — | — | The layers the downward cast considers; 0 every layer. |
| `exclude` | `Entity \| null` | `null` | — | — | — | An entity the cast ignores (the selected unit). |
| `reach` | `number` | `50` | ≥ 0.1 | — | — | How far above and below a vertex's own y the cast looks. |
| `bias` | `number` | `0.04` | ≥ 0 | — | — | Lifted this far above the surface, so the shape never z-fights or sinks between samples. |
| `spacing` | `number` | `0.5` | ≥ 0.05 | — | — | The longest edge between sampled vertices; a shape's outline keeps to 512 vertices, spacing them wider when it would need more. |

```ogt
struct GroundDesc {
  doc "What a ground shape (a selection ring, a telegraph, a path line, a placement ghost) lies on: its outline is cut into vertices `spacing` apart and each is lifted to the surface under it, so it follows a slope or terrain instead of floating over or cutting into it. With neither a terrain nor colliders it lies flat at its points' own y."
  desc true
  field terrain TerrainData "A terrain sampled under each vertex (og.terrain.sample); 0 is none."
  field origin Vector3 "Where the terrain stands: its og.Terrain entity's world position."
  field quaternion Quaternion "How the terrain is turned (the zero value: along world x and z)."
  field scale Vector3 "How the terrain is scaled." {
    default 1 1 1
  }
  field colliders bool "Also cast straight down against colliders (og.physics.castRay, as the tick began); the highest surface of the two wins. Exclude the unit itself (exclude) or keep units off mask's layers, or the ring climbs onto it."
  field mask u32 "The layers the downward cast considers; 0 every layer."
  field exclude Option.Entity "An entity the cast ignores (the selected unit)."
  field reach f32 "How far above and below a vertex's own y the cast looks." {
    default 50
    min 0.1
  }
  field bias f32 "Lifted this far above the surface, so the shape never z-fights or sinks between samples." {
    default 0.04
    min 0
  }
  field spacing f32 "The longest edge between sampled vertices; a shape's outline keeps to 512 vertices, spacing them wider when it would need more." {
    default 0.5
    min 0.05
  }
}
```

<sub>Module `ui` line 6641</sub>
