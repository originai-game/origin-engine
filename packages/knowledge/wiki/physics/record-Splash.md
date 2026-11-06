<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`physics`](../physics.md) · [index](../index.md)

<a id="record-Splash"></a>

### `og.Splash` (event)

Something crossed a water surface (og.WaterSurface or og.WaterLine): for splash particles and sounds. Read the next tick.

Token `og.Splash`, registered under the schema name `og.Splash`; its value type is `Splash`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `water` | `Entity` | `0` | — | — | — | The water's entity. |
| `entity` | `Entity` | `0` | — | — | — | What crossed it. |
| `point` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | worldspace `world` | Where it crossed the surface. |
| `speed` | `number` | `0` | — | `mps` | — | Its vertical speed (negative going in). |

```ogt
event Splash {
  doc "Something crossed a water surface (og.WaterSurface or og.WaterLine): for splash particles and sounds. Read the next tick."
  field water Entity "The water's entity."
  field entity Entity "What crossed it."
  field point Vector3 "Where it crossed the surface." {
    worldspace world
    unit m
  }
  field speed f32 "Its vertical speed (negative going in)." {
    unit mps
  }
}
```

<sub>`api.ogt` line 10123</sub>
