<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`load`](../load.md) · [index](../index.md)

<a id="type-EnvironmentSunTurn"></a>

### `EnvironmentSunTurn` (struct)

The Environment.rotation og.load.environmentSunRotation answered, and where the map's sun stands after it.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `rotation` | `number` | `0` | — | — | — | The Environment.rotation to set, in radians about +Y (-pi to pi); pass the same to og.load.environmentSun for the DirectionalLight. |
| `direction` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | — | — | The world-space unit direction towards the sun after the turn: og.load.environmentSun's direction at `rotation`. |
| `elevation` | `number` | `0` | — | — | — | The sun's elevation above the horizon in radians, which is the map's own: a turn about +Y never changes it, so a sun lower or higher than wanted needs another sky. |

```ogt
struct EnvironmentSunTurn {
  doc "The Environment.rotation og.load.environmentSunRotation answered, and where the map's sun stands after it."
  field rotation f32 "The Environment.rotation to set, in radians about +Y (-pi to pi); pass the same to og.load.environmentSun for the DirectionalLight."
  field direction Vector3 "The world-space unit direction towards the sun after the turn: og.load.environmentSun's direction at `rotation`."
  field elevation f32 "The sun's elevation above the horizon in radians, which is the map's own: a turn about +Y never changes it, so a sun lower or higher than wanted needs another sky."
}
```

<sub>Module `load` line 340</sub>
