<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

# The `camera` module

The main camera, as a resource with three.js PerspectiveCamera and OrthographicCamera parameter names.

Camera controllers are components (OrbitCamera, FollowCamera, Follow2D) on the entity the camera orbits or follows; the systems that drive them ship in TypeScript as og.systems.cameras.

- **TypeScript**: functions are none at module level (its members live on the types below); components, resources and events are tokens on `og`.
- **Publish builds**: kept (members may override).
- **Index**: [every module and name](index.md).

```ogt
module camera {
  doc "The main camera, as a resource with three.js PerspectiveCamera and OrthographicCamera parameter names."
  doc "Camera controllers are components (OrbitCamera, FollowCamera, Follow2D) on the entity the camera orbits or follows; the systems that drive them ship in TypeScript as og.systems.cameras."
  # declarations follow, each quoted below
```

<sub>`api.ogt` line 1148</sub>

## Contents

| Name | Kind | Summary |
|---|---|---|
| [`og.Camera`](#record-Camera) | resource | The camera the frame is rendered from. Rendering reads it after the tick; it never feeds back into the simulation. |
| [`og.OrbitCamera`](#record-OrbitCamera) | component | Orbits og.Camera around the entity carrying it, as three.js OrbitControls: a left-button drag on the play surface turns it, the wheel zooms, autoRotate turns it on its own. |
| [`og.FollowCamera`](#record-FollowCamera) | component | Chases the entity carrying it with og.Camera: a third-person 3D view that eases towards a point offset from the entity and looks at it. Driven by og.systems.cameras; the first entity carrying one, in query order, drives the camera. |
| [`og.Follow2D`](#record-Follow2D) | component | Keeps og.Camera on the entity carrying it in the xy plane, for 2D games: the camera moves only when the entity leaves the dead zone, eases by smoothTime and, with clamp, stays inside the bounds. The camera keeps its z, rotation and projection. Driven by og.systems.cameras. |

## Components, resources and events

<a id="record-Camera"></a>

### `og.Camera` (resource)

The camera the frame is rendered from. Rendering reads it after the tick; it never feeds back into the simulation.

Token `og.Camera`, registered under the schema name `og.Camera`; its value type is `Camera`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `position` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | — | Where the camera is. |
| `quaternion` | `Quaternion` | `{ x: 0, y: 0, z: 0, w: 1 }` | — | — | — | Its rotation; the camera looks down its local -Z. |
| `fov` | `number` | `50` | 1 … 179 | `deg` | tunable | The vertical field of view of the perspective projection. |
| `near` | `number` | `0.1` | ≥ 0.001 | `m` | — | The near clipping plane. |
| `far` | `number` | `2000` | ≥ 0.01 | `m` | — | The far clipping plane. |
| `zoom` | `number` | `1` | 0.01 … 100 | — | tunable | The zoom factor, as three.js Camera.zoom. |
| `orthographic` | `boolean` | `false` | — | — | — | Use an orthographic projection (the 2D profile) instead of a perspective one. |
| `height` | `number` | `10` | ≥ 0.01 | `m` | — | The visible world height of the orthographic projection. |
| `background` | `Color` | `{ r: 0.00699, g: 0.00803, b: 0.01096 }` | — | — | — | The clear colour, as three.js Scene.background. The default is the brand ink #14161b in linear RGB. |
| `exposure` | `number` | `1` | 0 … 16 | — | tunable | Exposure before tone mapping, as three.js toneMappingExposure. |

```ogt
resource Camera {
  doc "The camera the frame is rendered from. Rendering reads it after the tick; it never feeds back into the simulation."
  field position Vector3 "Where the camera is." {
    unit m
  }
  field quaternion Quaternion "Its rotation; the camera looks down its local -Z."
  field fov f32 "The vertical field of view of the perspective projection." {
    default 50
    min 1
    max 179
    unit deg
    tunable true
  }
  field near f32 "The near clipping plane." {
    default 0.1
    min 0.001
    unit m
  }
  field far f32 "The far clipping plane." {
    default 2000
    min 0.01
    unit m
  }
  field zoom f32 "The zoom factor, as three.js Camera.zoom." {
    default 1
    min 0.01
    max 100
    tunable true
  }
  field orthographic bool "Use an orthographic projection (the 2D profile) instead of a perspective one."
  field height f32 "The visible world height of the orthographic projection." {
    default 10
    min 0.01
    unit m
  }
  field background Color "The clear colour, as three.js Scene.background. The default is the brand ink #14161b in linear RGB." {
    default 0.00699 0.00803 0.01096
  }
  field exposure f32 "Exposure before tone mapping, as three.js toneMappingExposure." {
    default 1
    min 0
    max 16
    tunable true
  }
  fn lookAt {
    doc "Rotate the camera to look at a point, as three.js Camera.lookAt."
    param target Vector3 "The point to look at."
  }
  fn worldToScreen {
    doc "Where a world point lands on the play surface."
    param point Vector3 "A world point."
    returns Option.Vector2 "CSS pixels, or null behind the camera."
  }
  fn screenToRay {
    doc "The ray through a point of the play surface, for picking."
    param x f32 "Horizontal position in CSS pixels."
    param y f32 "Vertical position in CSS pixels."
    returns Ray
  }
}
```

<sub>`api.ogt` line 1152</sub>

<a id="member-Camera-lookAt"></a>

#### `Camera.lookAt`

Rotate the camera to look at a point, as three.js Camera.lookAt.

```ts
Camera.lookAt(target: Vector3): void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `target` | `Vector3` | no | The point to look at. |

**Returns** `void`.

Binding symbol `og_camera_look_at` · publish builds: kept

```ogt
fn lookAt {
  doc "Rotate the camera to look at a point, as three.js Camera.lookAt."
  param target Vector3 "The point to look at."
}
```

<sub>`api.ogt` line 1196</sub>

<a id="member-Camera-worldToScreen"></a>

#### `Camera.worldToScreen`

Where a world point lands on the play surface.

```ts
Camera.worldToScreen(point: Vector3): Vector2 | null
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `point` | `Vector3` | no | A world point. |

**Returns** `Vector2 | null`: CSS pixels, or null behind the camera.

Binding symbol `og_camera_world_to_screen` · publish builds: kept

```ogt
fn worldToScreen {
  doc "Where a world point lands on the play surface."
  param point Vector3 "A world point."
  returns Option.Vector2 "CSS pixels, or null behind the camera."
}
```

<sub>`api.ogt` line 1200</sub>

<a id="member-Camera-screenToRay"></a>

#### `Camera.screenToRay`

The ray through a point of the play surface, for picking.

```ts
Camera.screenToRay(x: number, y: number): Ray
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `x` | `number` | no | Horizontal position in CSS pixels. |
| `y` | `number` | no | Vertical position in CSS pixels. |

**Returns** `Ray`.

Binding symbol `og_camera_screen_to_ray` · publish builds: kept

```ogt
fn screenToRay {
  doc "The ray through a point of the play surface, for picking."
  param x f32 "Horizontal position in CSS pixels."
  param y f32 "Vertical position in CSS pixels."
  returns Ray
}
```

<sub>`api.ogt` line 1205</sub>

<a id="record-OrbitCamera"></a>

### `og.OrbitCamera` (component)

Orbits og.Camera around the entity carrying it, as three.js OrbitControls: a left-button drag on the play surface turns it, the wheel zooms, autoRotate turns it on its own.

Camera controllers are game-level systems the engine ships in TypeScript (og.systems.cameras); list them in GameDesc.systems: `systems: [...og.systems.cameras, ...yours]`. The first entity carrying one, in query order, drives the camera.

The camera sits `distance` from the orbited point (the entity's position plus `target`) in the direction given by `azimuth` about +Y and `polar` from +Y, and looks at the point. Every angle is computed with deterministic arithmetic, so a replay moves the camera identically on every host.

Token `og.OrbitCamera`, registered under the schema name `og.OrbitCamera`; its value type is `OrbitCamera`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `target` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | — | The orbited point, relative to the entity's position. |
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
| `pointer` | `Vector2` | `{ x: 0, y: 0 }` | — | `cssPx` | read only | Controller state: where the pointer was on the previous tick of a drag. |

```ogt
component OrbitCamera {
  doc "Orbits og.Camera around the entity carrying it, as three.js OrbitControls: a left-button drag on the play surface turns it, the wheel zooms, autoRotate turns it on its own."
  doc "Camera controllers are game-level systems the engine ships in TypeScript (og.systems.cameras); list them in GameDesc.systems: `systems: [...og.systems.cameras, ...yours]`. The first entity carrying one, in query order, drives the camera."
  doc "The camera sits `distance` from the orbited point (the entity's position plus `target`) in the direction given by `azimuth` about +Y and `polar` from +Y, and looks at the point. Every angle is computed with deterministic arithmetic, so a replay moves the camera identically on every host."
  field target Vector3 "The orbited point, relative to the entity's position." {
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
    readonly true
    unit cssPx
  }
}
```

<sub>`api.ogt` line 1213</sub>

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

<sub>`api.ogt` line 1287</sub>

<a id="record-Follow2D"></a>

### `og.Follow2D` (component)

Keeps og.Camera on the entity carrying it in the xy plane, for 2D games: the camera moves only when the entity leaves the dead zone, eases by smoothTime and, with clamp, stays inside the bounds. The camera keeps its z, rotation and projection. Driven by og.systems.cameras.

Token `og.Follow2D`, registered under the schema name `og.Follow2D`; its value type is `Follow2D`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `offset` | `Vector2` | `{ x: 0, y: 0 }` | — | `m` | — | Where the camera centre sits relative to the entity. |
| `deadZone` | `Vector2` | `{ x: 0, y: 0 }` | — | `m` | — | Half the width and height of the box around the camera centre inside which the entity moves without moving the camera. |
| `smoothTime` | `number` | `0.15` | ≥ 0 | `s` | tunable | Roughly how long the camera takes to catch up; 0 snaps it every tick. |
| `clamp` | `boolean` | `false` | — | — | — | Keep the camera centre between boundsMin and boundsMax. |
| `boundsMin` | `Vector2` | `{ x: 0, y: 0 }` | — | `m` | — | The smallest x and y the camera centre takes with clamp. |
| `boundsMax` | `Vector2` | `{ x: 0, y: 0 }` | — | `m` | — | The largest x and y the camera centre takes with clamp. |

```ogt
component Follow2D {
  doc "Keeps og.Camera on the entity carrying it in the xy plane, for 2D games: the camera moves only when the entity leaves the dead zone, eases by smoothTime and, with clamp, stays inside the bounds. The camera keeps its z, rotation and projection. Driven by og.systems.cameras."
  field offset Vector2 "Where the camera centre sits relative to the entity." {
    unit m
  }
  field deadZone Vector2 "Half the width and height of the box around the camera centre inside which the entity moves without moving the camera." {
    unit m
  }
  field smoothTime f32 "Roughly how long the camera takes to catch up; 0 snaps it every tick." {
    default 0.15
    min 0
    unit s
    tunable true
  }
  field clamp bool "Keep the camera centre between boundsMin and boundsMax."
  field boundsMin Vector2 "The smallest x and y the camera centre takes with clamp." {
    unit m
  }
  field boundsMax Vector2 "The largest x and y the camera centre takes with clamp." {
    unit m
  }
}
```

<sub>`api.ogt` line 1306</sub>
