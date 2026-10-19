<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

# The `camera` module

The main camera, as a resource with three.js PerspectiveCamera and OrthographicCamera parameter names.

Camera controllers are components (OrbitCamera, FollowCamera, Follow2D) on the entity the camera orbits or follows; the systems that drive them ship in TypeScript as og.systems.cameras.

- **TypeScript**: functions are the namespace `og.camera`; components, resources and events are tokens on `og`.
- **Publish builds**: kept (members may override).
- **Skill**: `og-feel`, which teaches the module (how much of it to use, and how to prove it).
- **Index**: [every module and name](index.md).

```ogt
module camera {
  # declarations follow, each quoted below
```

<sub>`api.ogt` line 1220</sub>

## Contents

| Name | Kind | Summary |
|---|---|---|
| [`og.Camera`](#record-Camera) | resource | The camera the frame is rendered from. Rendering reads it after the tick; it never feeds back into the simulation. |
| [`og.OrbitCamera`](#record-OrbitCamera) | component | Orbits og.Camera around the entity carrying it, as three.js OrbitControls: a left-button drag on the play surface turns it, the wheel zooms, autoRotate turns it on its own. |
| [`og.FollowCamera`](#record-FollowCamera) | component | Chases the entity carrying it with og.Camera: a third-person 3D view that eases towards a point offset from the entity and looks at it. Driven by og.systems.cameras; the first entity carrying one, in query order, drives the camera. |
| [`og.Follow2D`](#record-Follow2D) | component | Keeps og.Camera on the entity carrying it in the xy plane, for 2D games: the camera moves only when the entity leaves the dead zone, eases by smoothTime (smoothTimeY for y when set) and, with clamp, stays inside the bounds; lookAhead leads it in the direction the entity moves. The camera keeps its z, rotation and projection. Driven by og.systems.cameras; og.CameraZone entities then keep the view inside the zone the entity is in. |
| [`og.CameraZone`](#record-CameraZone) | component | A rectangle of the xy plane (min to max, world units, not moved by the entity's Transform) that keeps a 2D view inside it while the entity followed by og.Follow2D is in it: rooms, arenas, a level's bounds. Of overlapping zones the highest priority wins, then the first in query order. The engine's og.camera system picks the zone after the game's systems (into og.Camera's zone fields); entering another zone scrolls the view across over the new zone's scrollTime, or jumps when it is 0. A zone smaller than the view centres it. |
| [`og.CameraRail`](#record-CameraRail) | component | Moves og.Camera along a spline through p0 to p(points - 1) (Catmull-Rom, through every point) to the point nearest the entity carrying it, easing by smoothTime: a corridor, a side-scroller's track, a 3D chase on rails. A perspective camera looks at the entity; an orthographic one keeps its z and rotation. Applied by the engine's og.camera system after the game's systems. |
| [`og.camera.shake`](#fn-shake) | function | Add screen shake: trauma, clamped to 1, which og.Camera decays by traumaDecay. The shake grows with trauma squared, so 0.1-0.2 is a light hit, 0.25-0.4 a medium one, 0.5-0.8 an explosion or a boss slam; 1 is almost never right (nauseating). Visual only. Systems may call it; it takes effect at the engine's og.camera system this tick. |
| [`og.camera.punch`](#fn-punch) | function | A zoom punch: the view narrows by `amount` (0.05 is a tap, 0.15 a slam) and springs back by Camera.punchDecay. Visual only. |
| [`og.camera.kick`](#fn-kick) | function | A directional nudge of the view (recoil, an impact from one side): offset by `distance` along `direction`, returning by Camera.kickDecay. Visual only. |

## Functions

<a id="fn-shake"></a>

### `og.camera.shake`

Add screen shake: trauma, clamped to 1, which og.Camera decays by traumaDecay. The shake grows with trauma squared, so 0.1-0.2 is a light hit, 0.25-0.4 a medium one, 0.5-0.8 an explosion or a boss slam; 1 is almost never right (nauseating). Visual only. Systems may call it; it takes effect at the engine's og.camera system this tick.

```ts
og.camera.shake(world: World, trauma: number): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `trauma` | `number` | no | Trauma to add, 0 to 1. |

**Returns** `Result<void>`.

Binding symbol `og_camera_shake` · publish builds: kept

```ogt
fn shake {
  doc "Add screen shake: trauma, clamped to 1, which og.Camera decays by traumaDecay. The shake grows with trauma squared, so 0.1-0.2 is a light hit, 0.25-0.4 a medium one, 0.5-0.8 an explosion or a boss slam; 1 is almost never right (nauseating). Visual only. Systems may call it; it takes effect at the engine's og.camera system this tick."
  param world World "The world."
  param trauma f32 "Trauma to add, 0 to 1."
  returns Result.Void
}
```

<sub>`api.ogt` line 1556</sub>

<a id="fn-punch"></a>

### `og.camera.punch`

A zoom punch: the view narrows by `amount` (0.05 is a tap, 0.15 a slam) and springs back by Camera.punchDecay. Visual only.

```ts
og.camera.punch(world: World, amount: number): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `amount` | `number` | no | The fraction to zoom in (negative zooms out), -0.9 to 4. |

**Returns** `Result<void>`.

Binding symbol `og_camera_punch` · publish builds: kept

```ogt
fn punch {
  doc "A zoom punch: the view narrows by `amount` (0.05 is a tap, 0.15 a slam) and springs back by Camera.punchDecay. Visual only."
  param world World "The world."
  param amount f32 "The fraction to zoom in (negative zooms out), -0.9 to 4."
  returns Result.Void
}
```

<sub>`api.ogt` line 1563</sub>

<a id="fn-kick"></a>

### `og.camera.kick`

A directional nudge of the view (recoil, an impact from one side): offset by `distance` along `direction`, returning by Camera.kickDecay. Visual only.

```ts
og.camera.kick(world: World, direction: Vector3, distance: number): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `direction` | `Vector3` | no | The direction to push the view (normalised here; zero does nothing). |
| `distance` | `number` | no | How far, in world units. |

**Returns** `Result<void>`.

Binding symbol `og_camera_kick` · publish builds: kept

```ogt
fn kick {
  doc "A directional nudge of the view (recoil, an impact from one side): offset by `distance` along `direction`, returning by Camera.kickDecay. Visual only."
  param world World "The world."
  param direction Vector3 "The direction to push the view (normalised here; zero does nothing)."
  param distance f32 "How far, in world units."
  returns Result.Void
}
```

<sub>`api.ogt` line 1570</sub>

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
| `trauma` | `number` | `0` | 0 … 1 | — | tunable | Screen shake, 0 to 1, added by og.camera.shake and decayed by traumaDecay every tick (unscaled, so a hit stop still shakes). The offset and roll grow with trauma squared and follow smooth noise of the tick: visual only, a replay shakes the same. |
| `traumaDecay` | `number` | `1.5` | ≥ 0 | `perSecond` | tunable | How much trauma drains per second. |
| `shakeOffset` | `number` | `0.04` | ≥ 0 | — | tunable | The largest offset at trauma 1, as a fraction of the view height. |
| `shakeRoll` | `number` | `2` | ≥ 0 | `deg` | tunable | The largest roll at trauma 1. |
| `shakeFrequency` | `number` | `15` | ≥ 0 | `hz` | — | How fast the shake moves. |
| `punch` | `number` | `0` | -0.9 … 4 | — | — | A zoom punch in progress (og.camera.punch): the view is narrowed by this fraction, decaying by punchDecay. Visual only. |
| `punchDecay` | `number` | `8` | ≥ 0 | `perSecond` | tunable | How fast a punch springs back (exponential, per second). |
| `kick` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | — | A kick in progress (og.camera.kick): the view is offset by this, decaying by kickDecay. Visual only. |
| `kickDecay` | `number` | `12` | ≥ 0 | `perSecond` | tunable | How fast a kick returns (exponential, per second). |
| `zoneActive` | `boolean` | `false` | — | — | — | The followed entity is in an og.CameraZone: the view keeps inside zoneMin to zoneMax. Written by the og.camera system. |
| `zoneMin` | `Vector2` | `{ x: 0, y: 0 }` | — | `m` | — | The zone the view keeps inside: its bottom-left corner. |
| `zoneMax` | `Vector2` | `{ x: 0, y: 0 }` | — | `m` | — | Its top-right corner. |
| `zoneFromActive` | `boolean` | `false` | — | — | — | A room transition is in progress from the zone zoneFromMin to zoneFromMax: the view scrolls from where that zone kept it to where the new one does. |
| `zoneFromMin` | `Vector2` | `{ x: 0, y: 0 }` | — | `m` | — | The zone being left: its bottom-left corner. |
| `zoneFromMax` | `Vector2` | `{ x: 0, y: 0 }` | — | `m` | — | Its top-right corner. |
| `zoneScroll` | `number` | `1` | 0 … 1 | — | — | How far the transition has come, 0 to 1 (eased; 1 is settled). |
| `zoneScrollTime` | `number` | `0` | ≥ 0 | `s` | — | How long the transition takes: the entered zone's scrollTime. |

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
  field trauma f32 "Screen shake, 0 to 1, added by og.camera.shake and decayed by traumaDecay every tick (unscaled, so a hit stop still shakes). The offset and roll grow with trauma squared and follow smooth noise of the tick: visual only, a replay shakes the same." {
    min 0
    max 1
    tunable true
  }
  field traumaDecay f32 "How much trauma drains per second." {
    default 1.5
    min 0
    unit perSecond
    tunable true
  }
  field shakeOffset f32 "The largest offset at trauma 1, as a fraction of the view height." {
    default 0.04
    min 0
    tunable true
  }
  field shakeRoll f32 "The largest roll at trauma 1." {
    default 2
    min 0
    unit deg
    tunable true
  }
  field shakeFrequency f32 "How fast the shake moves." {
    default 15
    min 0
    unit hz
  }
  field punch f32 "A zoom punch in progress (og.camera.punch): the view is narrowed by this fraction, decaying by punchDecay. Visual only." {
    min -0.9
    max 4
  }
  field punchDecay f32 "How fast a punch springs back (exponential, per second)." {
    default 8
    min 0
    unit perSecond
    tunable true
  }
  field kick Vector3 "A kick in progress (og.camera.kick): the view is offset by this, decaying by kickDecay. Visual only." {
    unit m
  }
  field kickDecay f32 "How fast a kick returns (exponential, per second)." {
    default 12
    min 0
    unit perSecond
    tunable true
  }
  field zoneActive bool "The followed entity is in an og.CameraZone: the view keeps inside zoneMin to zoneMax. Written by the og.camera system."
  field zoneMin Vector2 "The zone the view keeps inside: its bottom-left corner." {
    unit m
  }
  field zoneMax Vector2 "Its top-right corner." {
    unit m
  }
  field zoneFromActive bool "A room transition is in progress from the zone zoneFromMin to zoneFromMax: the view scrolls from where that zone kept it to where the new one does."
  field zoneFromMin Vector2 "The zone being left: its bottom-left corner." {
    unit m
  }
  field zoneFromMax Vector2 "Its top-right corner." {
    unit m
  }
  field zoneScroll f32 "How far the transition has come, 0 to 1 (eased; 1 is settled)." {
    default 1
    min 0
    max 1
  }
  field zoneScrollTime f32 "How long the transition takes: the entered zone's scrollTime." {
    unit s
    min 0
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

<sub>`api.ogt` line 1225</sub>

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

<sub>`api.ogt` line 1338</sub>

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

<sub>`api.ogt` line 1342</sub>

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

<sub>`api.ogt` line 1347</sub>

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

<sub>`api.ogt` line 1355</sub>

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

<sub>`api.ogt` line 1429</sub>

<a id="record-Follow2D"></a>

### `og.Follow2D` (component)

Keeps og.Camera on the entity carrying it in the xy plane, for 2D games: the camera moves only when the entity leaves the dead zone, eases by smoothTime (smoothTimeY for y when set) and, with clamp, stays inside the bounds; lookAhead leads it in the direction the entity moves. The camera keeps its z, rotation and projection. Driven by og.systems.cameras; og.CameraZone entities then keep the view inside the zone the entity is in.

Token `og.Follow2D`, registered under the schema name `og.Follow2D`; its value type is `Follow2D`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `offset` | `Vector2` | `{ x: 0, y: 0 }` | — | `m` | — | Where the camera centre sits relative to the entity. |
| `deadZone` | `Vector2` | `{ x: 0, y: 0 }` | — | `m` | — | Half the width and height of the box around the camera centre inside which the entity moves without moving the camera. |
| `smoothTime` | `number` | `0.15` | ≥ 0 | `s` | tunable | Roughly how long the camera takes to catch up; 0 snaps it every tick. |
| `smoothTimeY` | `number` | `-1` | — | `s` | tunable | The same for y alone (a platformer follows jumps more lazily than runs); below 0 uses smoothTime. |
| `lookAhead` | `number` | `0` | ≥ 0 | `s` | tunable | Lead the entity by its velocity times this many seconds (0: no look-ahead). |
| `lookAheadMax` | `Vector2` | `{ x: 3, y: 1.5 }` | ≥ 0 | `m` | — | The largest lead along x and y. |
| `lookAheadSmoothing` | `number` | `0.4` | ≥ 0 | `s` | tunable | Roughly how long the lead takes to turn when the entity does. |
| `lead` | `Vector2` | `{ x: 0, y: 0 }` | — | `m` | read only | Controller state: the current lead. |
| `last` | `Vector2` | `{ x: 0, y: 0 }` | — | `m` | read only | Controller state: the entity's position on the previous tick. |
| `clamp` | `boolean` | `false` | — | — | — | Keep the camera centre between boundsMin and boundsMax. |
| `boundsMin` | `Vector2` | `{ x: 0, y: 0 }` | — | `m` | — | The smallest x and y the camera centre takes with clamp. |
| `boundsMax` | `Vector2` | `{ x: 0, y: 0 }` | — | `m` | — | The largest x and y the camera centre takes with clamp. |

```ogt
component Follow2D {
  doc "Keeps og.Camera on the entity carrying it in the xy plane, for 2D games: the camera moves only when the entity leaves the dead zone, eases by smoothTime (smoothTimeY for y when set) and, with clamp, stays inside the bounds; lookAhead leads it in the direction the entity moves. The camera keeps its z, rotation and projection. Driven by og.systems.cameras; og.CameraZone entities then keep the view inside the zone the entity is in."
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
  field smoothTimeY f32 "The same for y alone (a platformer follows jumps more lazily than runs); below 0 uses smoothTime." {
    default -1
    unit s
    tunable true
  }
  field lookAhead f32 "Lead the entity by its velocity times this many seconds (0: no look-ahead)." {
    min 0
    unit s
    tunable true
  }
  field lookAheadMax Vector2 "The largest lead along x and y." {
    default 3 1.5
    min 0
    unit m
  }
  field lookAheadSmoothing f32 "Roughly how long the lead takes to turn when the entity does." {
    default 0.4
    min 0
    unit s
    tunable true
  }
  field lead Vector2 "Controller state: the current lead." {
    readonly true
    unit m
  }
  field last Vector2 "Controller state: the entity's position on the previous tick." {
    readonly true
    unit m
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

<sub>`api.ogt` line 1448</sub>

<a id="record-CameraZone"></a>

### `og.CameraZone` (component)

A rectangle of the xy plane (min to max, world units, not moved by the entity's Transform) that keeps a 2D view inside it while the entity followed by og.Follow2D is in it: rooms, arenas, a level's bounds. Of overlapping zones the highest priority wins, then the first in query order. The engine's og.camera system picks the zone after the game's systems (into og.Camera's zone fields); entering another zone scrolls the view across over the new zone's scrollTime, or jumps when it is 0. A zone smaller than the view centres it.

Token `og.CameraZone`, registered under the schema name `og.CameraZone`; its value type is `CameraZone`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `min` | `Vector2` | `{ x: 0, y: 0 }` | — | `m` | — | The bottom-left corner. |
| `max` | `Vector2` | `{ x: 10, y: 10 }` | — | `m` | — | The top-right corner. |
| `priority` | `number` | `0` | — | — | — | Higher wins where zones overlap. |
| `scrollTime` | `number` | `0` | ≥ 0 | `s` | — | Seconds the view takes to scroll from the zone the entity left into this one (a room transition, eased in and out); 0 jumps at once. Visual only: the camera resource records the transition, so replays scroll the same. |

```ogt
component CameraZone {
  doc "A rectangle of the xy plane (min to max, world units, not moved by the entity's Transform) that keeps a 2D view inside it while the entity followed by og.Follow2D is in it: rooms, arenas, a level's bounds. Of overlapping zones the highest priority wins, then the first in query order. The engine's og.camera system picks the zone after the game's systems (into og.Camera's zone fields); entering another zone scrolls the view across over the new zone's scrollTime, or jumps when it is 0. A zone smaller than the view centres it."
  field min Vector2 "The bottom-left corner." {
    unit m
  }
  field max Vector2 "The top-right corner." {
    default 10 10
    unit m
  }
  field priority i32 "Higher wins where zones overlap."
  field scrollTime f32 "Seconds the view takes to scroll from the zone the entity left into this one (a room transition, eased in and out); 0 jumps at once. Visual only: the camera resource records the transition, so replays scroll the same." {
    unit s
    min 0
  }
}
```

<sub>`api.ogt` line 1500</sub>

<a id="record-CameraRail"></a>

### `og.CameraRail` (component)

Moves og.Camera along a spline through p0 to p(points - 1) (Catmull-Rom, through every point) to the point nearest the entity carrying it, easing by smoothTime: a corridor, a side-scroller's track, a 3D chase on rails. A perspective camera looks at the entity; an orthographic one keeps its z and rotation. Applied by the engine's og.camera system after the game's systems.

Token `og.CameraRail`, registered under the schema name `og.CameraRail`; its value type is `CameraRail`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `points` | `number` | `2` | 2 … 8 | — | — | How many of p0...p7 the rail uses (2 to 8). |
| `p0` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | — | Rail point 0. |
| `p1` | `Vector3` | `{ x: 10, y: 0, z: 0 }` | — | `m` | — | Rail point 1. |
| `p2` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | — | Rail point 2. |
| `p3` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | — | Rail point 3. |
| `p4` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | — | Rail point 4. |
| `p5` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | — | Rail point 5. |
| `p6` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | — | Rail point 6. |
| `p7` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | — | Rail point 7. |
| `smoothTime` | `number` | `0.2` | ≥ 0 | `s` | tunable | Roughly how long the camera takes to catch up along the rail; 0 snaps. |

```ogt
component CameraRail {
  doc "Moves og.Camera along a spline through p0 to p(points - 1) (Catmull-Rom, through every point) to the point nearest the entity carrying it, easing by smoothTime: a corridor, a side-scroller's track, a 3D chase on rails. A perspective camera looks at the entity; an orthographic one keeps its z and rotation. Applied by the engine's og.camera system after the game's systems."
  field points u32 "How many of p0...p7 the rail uses (2 to 8)." {
    default 2
    min 2
    max 8
  }
  field p0 Vector3 "Rail point 0." {
    unit m
  }
  field p1 Vector3 "Rail point 1." {
    default 10 0 0
    unit m
  }
  field p2 Vector3 "Rail point 2." {
    unit m
  }
  field p3 Vector3 "Rail point 3." {
    unit m
  }
  field p4 Vector3 "Rail point 4." {
    unit m
  }
  field p5 Vector3 "Rail point 5." {
    unit m
  }
  field p6 Vector3 "Rail point 6." {
    unit m
  }
  field p7 Vector3 "Rail point 7." {
    unit m
  }
  field smoothTime f32 "Roughly how long the camera takes to catch up along the rail; 0 snaps." {
    default 0.2
    min 0
    unit s
    tunable true
  }
}
```

<sub>`api.ogt` line 1516</sub>
