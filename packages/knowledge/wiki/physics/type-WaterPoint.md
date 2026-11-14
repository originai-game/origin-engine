<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`physics`](../physics.md) · [index](../index.md)

<a id="type-WaterPoint"></a>

### `WaterPoint` (struct)

The water at a point (og.physics.water): whose it is, where its surface stands over the point and how it moves there.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `water` | `Entity` | `0` | — | — | — | The water's entity (its og.Water, og.WaterSurface or og.WaterLine). |
| `height` | `number` | `0` | — | `m` | — | The surface over the point (world y), as it is drawn: og.physics.waterHeight's. |
| `depth` | `number` | `0` | — | `m` | — | How far the point is under the surface; negative over it. |
| `velocity` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `mps` | — | How the water moves at the surface there: a sea's waves carrying it round, a river's current, a splash's rise; zero on still water. |
| `acceleration` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `mps2` | — | How the water's velocity changes there (a sea's waves; zero elsewhere): what a floating thing feels beside gravity. |
| `normal` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | — | — | The surface's unit normal there: up on still water, leaning with a wave's slope (z is 0 on an og.WaterLine). |

```ogt
struct WaterPoint {
  doc "The water at a point (og.physics.water): whose it is, where its surface stands over the point and how it moves there."
  field water Entity "The water's entity (its og.Water, og.WaterSurface or og.WaterLine)."
  field height f32 "The surface over the point (world y), as it is drawn: og.physics.waterHeight's." {
    unit m
  }
  field depth f32 "How far the point is under the surface; negative over it." {
    unit m
  }
  field velocity Vector3 "How the water moves at the surface there: a sea's waves carrying it round, a river's current, a splash's rise; zero on still water." {
    unit mps
  }
  field acceleration Vector3 "How the water's velocity changes there (a sea's waves; zero elsewhere): what a floating thing feels beside gravity." {
    unit mps2
  }
  field normal Vector3 "The surface's unit normal there: up on still water, leaning with a wave's slope (z is 0 on an og.WaterLine)."
}
```

<sub>Module `physics` line 1745</sub>
