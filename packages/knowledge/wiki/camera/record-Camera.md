<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`camera`](../camera.md) · [index](../index.md)

<a id="record-Camera"></a>

### `og.Camera` (resource)

The camera the frame is rendered from. Rendering reads it after the tick; it never feeds back into the simulation.

Also known as: main camera, view, PerspectiveCamera, Camera3D

Token `og.Camera`, registered under the schema name `og.Camera`; its value type is `Camera`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `position` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | worldspace `world` | Where the camera is. |
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
| `zoneMin` | `Vector2` | `{ x: 0, y: 0 }` | — | `m` | worldspace `world` | The zone the view keeps inside: its bottom-left corner. |
| `zoneMax` | `Vector2` | `{ x: 0, y: 0 }` | — | `m` | worldspace `world` | Its top-right corner. |
| `zoneFromActive` | `boolean` | `false` | — | — | — | A room transition is in progress from the zone zoneFromMin to zoneFromMax: the view scrolls from where that zone kept it to where the new one does. |
| `zoneFromMin` | `Vector2` | `{ x: 0, y: 0 }` | — | `m` | worldspace `world` | The zone being left: its bottom-left corner. |
| `zoneFromMax` | `Vector2` | `{ x: 0, y: 0 }` | — | `m` | worldspace `world` | Its top-right corner. |
| `zoneScroll` | `number` | `1` | 0 … 1 | — | — | How far the transition has come, 0 to 1 (eased; 1 is settled). |
| `zoneScrollTime` | `number` | `0` | ≥ 0 | `s` | — | How long the transition takes: the entered zone's scrollTime. |
| `layers` | `number` | `1` | — | — | — | Three.js layers, as Camera.layers.mask: the frame draws the meshes, models, world sprites, particle systems, lines and points whose layers share a bit with it. 1 (layer 0) by default, as every object's. |

```ogt
resource Camera {
  doc "The camera the frame is rendered from. Rendering reads it after the tick; it never feeds back into the simulation."
  aliases "main camera, view, PerspectiveCamera, Camera3D"
  field position Vector3 "Where the camera is." {
    worldspace world
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
    worldspace world
    unit m
  }
  field zoneMax Vector2 "Its top-right corner." {
    worldspace world
    unit m
  }
  field zoneFromActive bool "A room transition is in progress from the zone zoneFromMin to zoneFromMax: the view scrolls from where that zone kept it to where the new one does."
  field zoneFromMin Vector2 "The zone being left: its bottom-left corner." {
    worldspace world
    unit m
  }
  field zoneFromMax Vector2 "Its top-right corner." {
    worldspace world
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
    aliases "world to screen, project to screen, screen position, Unity WorldToScreenPoint, Godot unproject_position, three.js Vector3.project"
    param point Vector3 "A world point."
    returns Option.Vector2 "CSS pixels, or null behind the camera."
  }
  fn screenToRay {
    doc "The ray through a point of the play surface, for picking."
    param x f32 "Horizontal position in CSS pixels."
    param y f32 "Vertical position in CSS pixels."
    returns Ray
  }
  field layers u32 "Three.js layers, as Camera.layers.mask: the frame draws the meshes, models, world sprites, particle systems, lines and points whose layers share a bit with it. 1 (layer 0) by default, as every object's." {
    default 1
  }
}
```

<sub>`api.ogt` line 3234</sub>

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

<sub>`api.ogt` line 3353</sub>

<a id="member-Camera-worldToScreen"></a>

#### `Camera.worldToScreen`

Where a world point lands on the play surface.

Also known as: world to screen, project to screen, screen position, Unity WorldToScreenPoint, Godot unproject_position, three.js Vector3.project

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
  aliases "world to screen, project to screen, screen position, Unity WorldToScreenPoint, Godot unproject_position, three.js Vector3.project"
  param point Vector3 "A world point."
  returns Option.Vector2 "CSS pixels, or null behind the camera."
}
```

<sub>`api.ogt` line 3357</sub>

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

<sub>`api.ogt` line 3363</sub>
