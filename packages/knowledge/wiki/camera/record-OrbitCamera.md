<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`camera`](../camera.md) · [index](../index.md)

<a id="record-OrbitCamera"></a>

### `og.OrbitCamera` (component)

Orbits og.Camera around the entity carrying it, as three.js OrbitControls: a left-button drag on the play surface turns it, the wheel zooms, autoRotate turns it on its own.

Camera controllers are game-level systems the engine ships in TypeScript (og.systems.cameras); list them in GameDesc.systems: `systems: [...og.systems.cameras, ...yours]`. The first entity carrying one, in query order, drives the camera.

The camera sits `distance` from the orbited point (the entity's position plus `target`) in the direction given by `azimuth` about +Y and `polar` from +Y, and looks at the point. Every angle is computed with deterministic arithmetic, so a replay moves the camera identically on every host.

Token `og.OrbitCamera`, registered under the schema name `og.OrbitCamera`; its value type is `OrbitCamera`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `target` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | worldspace `none` | The orbited point, relative to the entity's position. |
| `distance` | `number` | `10` | ≥ 0.01 | `m` | tunable | How far the camera is from the orbited point, as three.js OrbitControls' distance. |
| `minDistance` | `number` | `1` | ≥ 0.01 | `m` | — | The closest the wheel zooms in, as three.js OrbitControls.minDistance. |
| `maxDistance` | `number` | `100` | ≥ 0.01 | `m` | — | The farthest the wheel zooms out, as three.js OrbitControls.maxDistance. |
| `azimuth` | `number` | `0` | — | `rad` | tunable | The horizontal angle about +Y; 0 puts the camera on the +Z side of the point. |
| `polar` | `number` | `1.1` | 0 … 3.1416 | `rad` | tunable | The angle from +Y: near 0 looks straight down, half pi is level with the point. |
| `minPolarAngle` | `number` | `0.05` | 0 … 3.1416 | `rad` | — | The smallest polar angle a drag reaches, as three.js OrbitControls.minPolarAngle. |
| `maxPolarAngle` | `number` | `3.0916` | 0 … 3.1416 | `rad` | — | The largest polar angle a drag reaches, as three.js OrbitControls.maxPolarAngle. |
| `rotateSpeed` | `number` | `1` | ≥ 0 | — | tunable | Drag sensitivity, as three.js OrbitControls.rotateSpeed: at 1, dragging the play surface's height turns a full circle. |
| `zoomSpeed` | `number` | `1` | ≥ 0 | — | tunable | Wheel sensitivity, as three.js OrbitControls.zoomSpeed: at 1, 100 CSS pixels of wheel change the distance by a tenth. |
| `autoRotate` | `boolean` | `false` | — | — | — | Turn about the point on its own while not dragged, as three.js OrbitControls.autoRotate. |
| `autoRotateSpeed` | `number` | `0.5` | — | `radps` | tunable | How fast autoRotate turns. |
| `enabled` | `boolean` | `true` | — | — | — | Read the pointer and the wheel; false keeps only autoRotate and the fields' own values. |
| `dragging` | `boolean` | `false` | — | — | read only | Controller state: a left-button drag that started on the play surface is in progress. |
| `pointer` | `Vector2` | `{ x: 0, y: 0 }` | — | `cssPx` | read only, worldspace `none` | Controller state: where the pointer was on the previous tick of a drag. |

```ogt
component OrbitCamera {
  doc "Orbits og.Camera around the entity carrying it, as three.js OrbitControls: a left-button drag on the play surface turns it, the wheel zooms, autoRotate turns it on its own."
  doc "Camera controllers are game-level systems the engine ships in TypeScript (og.systems.cameras); list them in GameDesc.systems: `systems: [...og.systems.cameras, ...yours]`. The first entity carrying one, in query order, drives the camera."
  doc "The camera sits `distance` from the orbited point (the entity's position plus `target`) in the direction given by `azimuth` about +Y and `polar` from +Y, and looks at the point. Every angle is computed with deterministic arithmetic, so a replay moves the camera identically on every host."
  field target Vector3 "The orbited point, relative to the entity's position." {
    worldspace none
    unit m
  }
  field distance f32 "How far the camera is from the orbited point, as three.js OrbitControls' distance." {
    default 10
    min 0.01
    unit m
    tunable true
  }
  field minDistance f32 "The closest the wheel zooms in, as three.js OrbitControls.minDistance." {
    default 1
    min 0.01
    unit m
  }
  field maxDistance f32 "The farthest the wheel zooms out, as three.js OrbitControls.maxDistance." {
    default 100
    min 0.01
    unit m
  }
  field azimuth f32 "The horizontal angle about +Y; 0 puts the camera on the +Z side of the point." {
    unit rad
    tunable true
  }
  field polar f32 "The angle from +Y: near 0 looks straight down, half pi is level with the point." {
    default 1.1
    min 0
    max 3.1416
    unit rad
    tunable true
  }
  field minPolarAngle f32 "The smallest polar angle a drag reaches, as three.js OrbitControls.minPolarAngle." {
    default 0.05
    min 0
    max 3.1416
    unit rad
  }
  field maxPolarAngle f32 "The largest polar angle a drag reaches, as three.js OrbitControls.maxPolarAngle." {
    default 3.0916
    min 0
    max 3.1416
    unit rad
  }
  field rotateSpeed f32 "Drag sensitivity, as three.js OrbitControls.rotateSpeed: at 1, dragging the play surface's height turns a full circle." {
    default 1
    min 0
    tunable true
  }
  field zoomSpeed f32 "Wheel sensitivity, as three.js OrbitControls.zoomSpeed: at 1, 100 CSS pixels of wheel change the distance by a tenth." {
    default 1
    min 0
    tunable true
  }
  field autoRotate bool "Turn about the point on its own while not dragged, as three.js OrbitControls.autoRotate."
  field autoRotateSpeed f32 "How fast autoRotate turns." {
    default 0.5
    unit radps
    tunable true
  }
  field enabled bool "Read the pointer and the wheel; false keeps only autoRotate and the fields' own values." {
    default true
  }
  field dragging bool "Controller state: a left-button drag that started on the play surface is in progress." {
    readonly true
  }
  field pointer Vector2 "Controller state: where the pointer was on the previous tick of a drag." {
    worldspace none
    readonly true
    unit cssPx
  }
}
```

<sub>Module `camera` line 145</sub>
