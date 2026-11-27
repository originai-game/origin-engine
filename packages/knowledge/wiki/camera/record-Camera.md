<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`camera`](../camera.md) · [index](../index.md)

<a id="record-Camera"></a>

### `og.Camera` (resource)

The camera the frame is rendered from. Rendering reads it after the tick; it never feeds back into the simulation.

Also known as: main camera, view, PerspectiveCamera, Camera3D

Token `og.Camera`, registered under the schema name `og.Camera`; its value type is `Camera`.

#### Pose

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `position` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | worldspace `world` | Where the camera is. |
| `quaternion` | `Quaternion` | `{ x: 0, y: 0, z: 0, w: 1 }` | — | — | — | Its rotation; the camera looks down its local -Z. |

#### Projection

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `fov` | `number` | `50` | 1 … 179 | `deg` | tunable | The vertical field of view of the perspective projection. |
| `near` | `number` | `0.1` | ≥ 0.001 | `m` | — | The near clipping plane. |
| `far` | `number` | `2000` | ≥ 0.01 | `m` | — | The far clipping plane. |
| `zoom` | `number` | `1` | 0.01 … 100 | — | tunable | The zoom factor, as three.js Camera.zoom. |
| `orthographic` | `boolean` | `false` | — | — | — | Use an orthographic projection (the 2D profile) instead of a perspective one. |
| `height` | `number` | `10` | ≥ 0.01 | `m` | — | The visible world height of the orthographic projection. |

#### Physical lens

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `lensMode` | `CameraLensMode` | `"artistic"` | — | — | — | Artistic keeps fov; physical derives perspective from focalLength and sensorSize. An incomplete lens falls back to fov. |
| `focalLength` | `number` | `0` | ≥ 0 | — | — | Physical focal length, in millimetres; zero leaves the lens incomplete. |
| `sensorSize` | `Vector2` | `{ x: 0, y: 0 }` | — | — | — | Sensor width and height, in millimetres; both must be positive. |
| `gateFit` | `CameraGateFit` | `"vertical"` | — | — | — | How the sensor fits the output aspect without stretching. |
| `aperture` | `number` | `0` | ≥ 0 | — | — | Physical f-number; zero disables physical depth of field. |
| `focusDistance` | `number` | `0` | ≥ 0 | `m` | — | Physical focus plane's axial distance from the camera, in metres. |
| `shutterSpeed` | `number` | `0` | ≥ 0 | `s` | — | Physical exposure duration in seconds, not its reciprocal; zero disables physical motion blur. |
| `iso` | `number` | `0` | ≥ 0 | — | — | Physical exposure sensitivity; zero leaves settings-derived exposure incomplete. |
| `exposureMode` | `CameraExposureMode` | `"artistic"` | — | — | — | Artistic exposure, settings-derived exposure, or explicit EV100; automatic exposure overrides manual physical exposure. |
| `ev100` | `number` | `0` | — | — | — | Explicit photographic EV100 in stops; zero is a valid exposure. |
| `lensShift` | `Vector2` | `{ x: 0, y: 0 }` | — | — | — | Shift in fractions of the fitted gate; positive x/y moves the viewed window right/up (the optical-axis subject left/down). Perspective only. |
| `lensBlending` | `boolean` | `false` | — | — | read only | Brain state: lensProjection overrides the projection during a mixed-mode or interrupted lens blend. |
| `lensProjection` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | — | read only | Brain state: resolved vertical focal factor and gate shift x/y, before zoom, at the blend's aspect. |
| `lensStatus` | `CameraLensStatus` | `"artistic"` | — | — | read only | Projection status after the camera system resolves its lens: artistic, physical, invalid or orthographic. |
| `cocSources` | `readonly number[]` | `[]` | — | — | a list of at most 36 `f32` (`List.f32.36`): reads a frozen copy, writes a whole array, read only | Brain-owned CoC: at most four oldest-first sources, nine numbers each in CameraCocDropped.source order. Empty leaves the direct camera in control; zero-weight entries are inactive. |
| `postBlending` | `boolean` | `false` | — | — | read only | Brain-owned: postShutter resolves an active crossfade, including interrupted physical/artistic blends. |
| `postShutter` | `Vector2` | `{ x: 0, y: 0 }` | — | — | read only | Resolved shutter coefficients: x seconds / motion-vector interval + y artistic frame fraction. Read only while postBlending. |

#### Output

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `background` | `Color` | `{ r: 0.00699, g: 0.00803, b: 0.01096 }` | — | — | — | The clear colour, as three.js Scene.background. The default is the brand ink #14161b in linear RGB. |
| `backgroundTexture` | `Texture` | `0` | — | — | — | A texture behind the scene in background's place, cropped to cover the view, as three.js Scene.background with a texture: a video plate (og.VideoPlayer.texture, or a video track's `layer under`). Drawn as it is: no exposure, bloom or tone curve touches it, and auto exposure ignores it. 0: none. |
| `exposure` | `number` | `1` | 0 … 16 | — | tunable | Exposure before tone mapping, as three.js toneMappingExposure. |

#### Shake

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `trauma` | `number` | `0` | 0 … 1 | — | tunable | Screen shake, 0 to 1, added by og.camera.shake and decayed by traumaDecay every tick (unscaled, so a hit stop still shakes). The offset and roll grow with trauma squared and follow smooth noise of the tick: visual only, a replay shakes the same. |
| `traumaDecay` | `number` | `1.5` | ≥ 0 | `perSecond` | tunable | How much trauma drains per second. |
| `shakeOffset` | `number` | `0.04` | ≥ 0 | — | tunable | The largest offset at trauma 1, as a fraction of the view height. |
| `shakeRoll` | `number` | `2` | ≥ 0 | `deg` | tunable | The largest roll at trauma 1. |
| `shakeFrequency` | `number` | `15` | ≥ 0 | `hz` | — | How fast the shake moves. |

#### Punch and kick

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `punch` | `number` | `0` | -0.9 … 4 | — | — | A zoom punch in progress (og.camera.punch): the view is narrowed by this fraction, decaying by punchDecay. Visual only. |
| `punchDecay` | `number` | `8` | ≥ 0 | `perSecond` | tunable | How fast a punch springs back (exponential, per second). |
| `kick` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | — | A kick in progress (og.camera.kick): the view is offset by this, decaying by kickDecay. Visual only. |
| `kickDecay` | `number` | `12` | ≥ 0 | `perSecond` | tunable | How fast a kick returns (exponential, per second). |

#### Zones

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `zoneActive` | `boolean` | `false` | — | — | — | The followed entity is in an og.CameraZone: the view keeps inside zoneMin to zoneMax. Written by the og.camera system. |
| `zoneMin` | `Vector2` | `{ x: 0, y: 0 }` | — | `m` | worldspace `world` | The zone the view keeps inside: its bottom-left corner. |
| `zoneMax` | `Vector2` | `{ x: 0, y: 0 }` | — | `m` | worldspace `world` | Its top-right corner. |
| `zoneFromActive` | `boolean` | `false` | — | — | — | A room transition is in progress from the zone zoneFromMin to zoneFromMax: the view scrolls from where that zone kept it to where the new one does. |
| `zoneFromMin` | `Vector2` | `{ x: 0, y: 0 }` | — | `m` | worldspace `world` | The zone being left: its bottom-left corner. |
| `zoneFromMax` | `Vector2` | `{ x: 0, y: 0 }` | — | `m` | worldspace `world` | Its top-right corner. |
| `zoneScroll` | `number` | `1` | 0 … 1 | — | — | How far the transition has come, 0 to 1 (eased; 1 is settled). |
| `zoneScrollTime` | `number` | `0` | ≥ 0 | `s` | — | How long the transition takes: the entered zone's scrollTime. |

#### Layers

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `layers` | `number` | `1` | — | — | — | Three.js layers, as Camera.layers.mask: the frame draws the meshes, models, world sprites, particle systems, lines and points whose layers share a bit with it. 1 (layer 0) by default, as every object's. |

```ogt
resource Camera {
  doc "The camera the frame is rendered from. Rendering reads it after the tick; it never feeds back into the simulation."
  aliases "main camera, view, PerspectiveCamera, Camera3D"
  group "Pose"
  field position Vector3 "Where the camera is." {
    worldspace world
    unit m
  }
  field quaternion Quaternion "Its rotation; the camera looks down its local -Z."
  group "Projection"
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
  group "Physical lens"
  field lensMode CameraLensMode "Artistic keeps fov; physical derives perspective from focalLength and sensorSize. An incomplete lens falls back to fov."
  field focalLength f32 "Physical focal length, in millimetres; zero leaves the lens incomplete." {
    min 0
  }
  field sensorSize Vector2 "Sensor width and height, in millimetres; both must be positive."
  field gateFit CameraGateFit "How the sensor fits the output aspect without stretching."
  field aperture f32 "Physical f-number; zero disables physical depth of field." {
    min 0
  }
  field focusDistance f32 "Physical focus plane's axial distance from the camera, in metres." {
    min 0
    unit m
  }
  field shutterSpeed f32 "Physical exposure duration in seconds, not its reciprocal; zero disables physical motion blur." {
    min 0
    unit s
  }
  field iso f32 "Physical exposure sensitivity; zero leaves settings-derived exposure incomplete." {
    min 0
  }
  field exposureMode CameraExposureMode "Artistic exposure, settings-derived exposure, or explicit EV100; automatic exposure overrides manual physical exposure."
  field ev100 f32 "Explicit photographic EV100 in stops; zero is a valid exposure."
  field lensShift Vector2 "Shift in fractions of the fitted gate; positive x/y moves the viewed window right/up (the optical-axis subject left/down). Perspective only."
  field lensBlending bool "Brain state: lensProjection overrides the projection during a mixed-mode or interrupted lens blend." {
    readonly true
  }
  field lensProjection Vector3 "Brain state: resolved vertical focal factor and gate shift x/y, before zoom, at the blend's aspect." {
    readonly true
  }
  field lensStatus CameraLensStatus "Projection status after the camera system resolves its lens: artistic, physical, invalid or orthographic." {
    readonly true
  }
  field cocSources List.f32.36 "Brain-owned CoC: at most four oldest-first sources, nine numbers each in CameraCocDropped.source order. Empty leaves the direct camera in control; zero-weight entries are inactive." {
    readonly true
  }
  field postBlending bool "Brain-owned: postShutter resolves an active crossfade, including interrupted physical/artistic blends." {
    readonly true
  }
  field postShutter Vector2 "Resolved shutter coefficients: x seconds / motion-vector interval + y artistic frame fraction. Read only while postBlending." {
    readonly true
  }
  group "Output"
  field background Color "The clear colour, as three.js Scene.background. The default is the brand ink #14161b in linear RGB." {
    default 0.00699 0.00803 0.01096
  }
  field backgroundTexture Texture "A texture behind the scene in background's place, cropped to cover the view, as three.js Scene.background with a texture: a video plate (og.VideoPlayer.texture, or a video track's `layer under`). Drawn as it is: no exposure, bloom or tone curve touches it, and auto exposure ignores it. 0: none." {
  }
  field exposure f32 "Exposure before tone mapping, as three.js toneMappingExposure." {
    default 1
    min 0
    max 16
    tunable true
  }
  group "Shake"
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
  group "Punch and kick"
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
  group "Zones"
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
  group "Layers"
  field layers u32 "Three.js layers, as Camera.layers.mask: the frame draws the meshes, models, world sprites, particle systems, lines and points whose layers share a bit with it. 1 (layer 0) by default, as every object's." {
    default 1
  }
}
```

<sub>Module `camera` line 6</sub>

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

<sub>Module `camera` line 175</sub>

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

<sub>Module `camera` line 179</sub>

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

<sub>Module `camera` line 185</sub>
