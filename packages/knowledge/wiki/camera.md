<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

# The `camera` module

The main camera, as a resource with three.js PerspectiveCamera and OrthographicCamera parameter names.

- **TypeScript**: functions are none at module level (its members live on the types below); components, resources and events are tokens on `og`.
- **Publish builds**: kept (members may override).
- **Index**: [every module and name](index.md).

```ogt
module camera {
  doc "The main camera, as a resource with three.js PerspectiveCamera and OrthographicCamera parameter names."
  # declarations follow, each quoted below
```

<sub>`api.ogt` line 874</sub>

## Contents

| Name | Kind | Summary |
|---|---|---|
| [`og.Camera`](#record-Camera) | resource | The camera the frame is rendered from. Rendering reads it after the tick; it never feeds back into the simulation. |

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

<sub>`api.ogt` line 877</sub>

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

<sub>`api.ogt` line 921</sub>

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

<sub>`api.ogt` line 925</sub>

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

<sub>`api.ogt` line 930</sub>
