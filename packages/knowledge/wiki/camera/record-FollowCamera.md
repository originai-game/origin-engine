<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`camera`](../camera.md) · [index](../index.md)

<a id="record-FollowCamera"></a>

### `og.FollowCamera` (component)

Chases the entity carrying it with og.Camera: a third-person 3D view that eases towards a point offset from the entity and looks at it. Driven by og.systems.cameras; the first entity carrying one, in query order, drives the camera.

Token `og.FollowCamera`, registered under the schema name `og.FollowCamera`; its value type is `FollowCamera`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `offset` | `Vector3` | `{ x: 0, y: 2, z: 6 }` | — | `m` | — | Where the camera sits relative to the entity: in the entity's own axes with followRotation, else in world axes. |
| `lookOffset` | `Vector3` | `{ x: 0, y: 1, z: 0 }` | — | `m` | — | The point the camera looks at, relative to the entity's position. |
| `followRotation` | `boolean` | `false` | — | — | — | Turn the offset with the entity's rotation, so the camera stays behind it. |
| `smoothTime` | `number` | `0.25` | ≥ 0 | `s` | tunable | Roughly how long the camera takes to catch up with a moved entity; 0 snaps it there every tick. |

```ogt
component FollowCamera {
  doc "Chases the entity carrying it with og.Camera: a third-person 3D view that eases towards a point offset from the entity and looks at it. Driven by og.systems.cameras; the first entity carrying one, in query order, drives the camera."
  field offset Vector3 "Where the camera sits relative to the entity: in the entity's own axes with followRotation, else in world axes." {
    default 0 2 6
    unit m
  }
  field lookOffset Vector3 "The point the camera looks at, relative to the entity's position." {
    default 0 1 0
    unit m
  }
  field followRotation bool "Turn the offset with the entity's rotation, so the camera stays behind it."
  field smoothTime f32 "Roughly how long the camera takes to catch up with a moved entity; 0 snaps it there every tick." {
    default 0.25
    min 0
    unit s
    tunable true
  }
}
```

<sub>Module `camera` line 221</sub>
