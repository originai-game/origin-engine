<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

# The `camera` module

The main camera, as a resource with three.js PerspectiveCamera and OrthographicCamera parameter names.

Camera controllers are components (OrbitCamera, FollowCamera, Follow2D, FramingCamera) on the entity the camera orbits, follows or frames; the systems that drive them ship in TypeScript as og.systems.cameras.

- **TypeScript**: functions are the namespace `og.camera`; components, resources and events are tokens on `og`.
- **Publish builds**: kept (members may override).
- **Core** (what most games use): `og.Camera`, `og.Follow2D`, `og.camera.shake`; 20 more are advanced (`og api show camera --all`).
- **Index**: [every module](index.md).

```ogt
module camera {
  doc "The main camera, as a resource with three.js PerspectiveCamera and OrthographicCamera parameter names."
  doc "Camera controllers are components (OrbitCamera, FollowCamera, Follow2D, FramingCamera) on the entity the camera orbits, follows or frames; the systems that drive them ship in TypeScript as og.systems.cameras."
  # declarations follow, each on its own page
```

<sub>Module `camera` line 1</sub>

## Contents

| Name | Kind | Signature | Summary |
|---|---|---|---|
| [`og.Camera`](camera/record-Camera.md) | resource |  | The camera the frame is rendered from. |
| [`CameraLensMode`](camera/type-CameraLensMode.md) | enum |  | The source of a perspective camera's projection and optical effects. |
| [`CameraLensMode()`](#column-CameraLensMode) | column kind |  | A CameraLensMode field. |
| [`CameraGateFit`](camera/type-CameraGateFit.md) | enum |  | Fit a sensor to a target aspect without stretching pixels. |
| [`CameraGateFit()`](#column-CameraGateFit) | column kind |  | A CameraGateFit field. |
| [`CameraExposureMode`](camera/type-CameraExposureMode.md) | enum |  | Manual exposure source; existing artistic multipliers remain compensation. |
| [`CameraExposureMode()`](#column-CameraExposureMode) | column kind |  | A CameraExposureMode field. |
| [`CameraLensStatus`](camera/type-CameraLensStatus.md) | enum |  | The main camera's resolved projection source. |
| [`CameraLensStatus()`](#column-CameraLensStatus) | column kind |  | A CameraLensStatus field. |
| [`MAX_CAMERA_COC_SOURCES`](#const-MAX_CAMERA_COC_SOURCES) | constant |  | Maximum distinct primitive CoC sources in an interrupted camera blend; identical sources coalesce before overflow selection. |
| [`og.CameraCocDropped`](camera/record-CameraCocDropped.md) | event |  | A camera CoC blend exceeded MAX_CAMERA_COC_SOURCES after identical-source coalescing. |
| [`og.OrbitCamera`](camera/record-OrbitCamera.md) | component |  | Orbits og.Camera around the entity carrying it, as three.js OrbitControls: a left-button drag on the play surface turns it, the wheel zooms, autoRotate turns it on its own. |
| [`og.FollowCamera`](camera/record-FollowCamera.md) | component |  | Chases the entity carrying it with og.Camera: a third-person 3D view that eases towards a point offset from the entity and looks at it. |
| [`og.Follow2D`](camera/record-Follow2D.md) | component |  | Keeps og.Camera on the entity carrying it in the xy plane, for 2D games: the camera moves only when the entity leaves the dead zone, eases by smoothTime (smoothTimeY for y when set) and, with clamp, … |
| [`og.FramingCamera`](camera/record-FramingCamera.md) | component |  | Frames the entity carrying it with og.Camera by beats, as a shot list states them: per beat the subject's height as a fraction of the frame's, where its centre sits, and the view's yaw and pitch. |
| [`og.CameraZone`](camera/record-CameraZone.md) | component |  | A rectangle of the xy plane (min to max, world units, not moved by the entity's Transform) that keeps a 2D view inside it while the entity followed by og.Follow2D is in it: rooms, arenas, a level's b… |
| [`og.CameraRail`](camera/record-CameraRail.md) | component |  | Moves og.Camera along a spline through p0 to p(points - 1) (Catmull-Rom, through every point) to the point nearest the entity carrying it, easing by smoothTime: a corridor, a side-scroller's track, a… |
| [`og.VirtualCamera`](camera/record-VirtualCamera.md) | component |  | A shot og.Camera can take, as Cinemachine's virtual cameras: a position, what it looks at, a field of view, roll and focus. |
| [`og.CameraBrain`](camera/record-CameraBrain.md) | resource |  | Picks the live og.VirtualCamera every tick and blends og.Camera to it, as Cinemachine's brain: the solo shot if one is set, else the enabled one with the highest priority. |
| [`og.CameraCut`](camera/record-CameraCut.md) | event |  | The live virtual camera changed (sent by the og.camera system; read the tick after). |
| [`og.camera.shake`](camera/fn-shake.md) | function | `(world: World, trauma: number): Result<void>` | Add screen shake: trauma, clamped to 1, which og.Camera decays by traumaDecay. |
| [`og.camera.punch`](camera/fn-punch.md) | function | `(world: World, amount: number): Result<void>` | A zoom punch: the view narrows by `amount` and springs back by Camera.punchDecay. |
| [`og.camera.kick`](camera/fn-kick.md) | function | `(world: World, direction: Vector3, distance: number): Result<void>` | A directional nudge of the view (recoil, an impact from one side): offset by `distance` along `direction`, returning by Camera.kickDecay. |

## Column kinds

Each is a factory, `CameraLensMode(value?, options?)`, that makes one field of a component or resource layout.

| Column | Value | Typed array | Stride | Doc |
|---|---|---|---|---|
| <a id="column-CameraLensMode"></a>`CameraLensMode` | `CameraLensMode` | `Uint32Array` | 1 | A CameraLensMode field. |
| <a id="column-CameraGateFit"></a>`CameraGateFit` | `CameraGateFit` | `Uint32Array` | 1 | A CameraGateFit field. |
| <a id="column-CameraExposureMode"></a>`CameraExposureMode` | `CameraExposureMode` | `Uint32Array` | 1 | A CameraExposureMode field. |
| <a id="column-CameraLensStatus"></a>`CameraLensStatus` | `CameraLensStatus` | `Uint32Array` | 1 | A CameraLensStatus field. |

```ogt
column CameraLensMode {
  doc "A CameraLensMode field."
  array Uint32Array
  stride 1
  value CameraLensMode
}
```

<sub>Module `camera` line 201</sub>

```ogt
column CameraGateFit {
  doc "A CameraGateFit field."
  array Uint32Array
  stride 1
  value CameraGateFit
}
```

<sub>Module `camera` line 214</sub>

```ogt
column CameraExposureMode {
  doc "A CameraExposureMode field."
  array Uint32Array
  stride 1
  value CameraExposureMode
}
```

<sub>Module `camera` line 226</sub>

```ogt
column CameraLensStatus {
  doc "A CameraLensStatus field."
  array Uint32Array
  stride 1
  value CameraLensStatus
}
```

<sub>Module `camera` line 239</sub>

## Constants

| Constant | Type | Value | Doc |
|---|---|---|---|
| <a id="const-MAX_CAMERA_COC_SOURCES"></a>`MAX_CAMERA_COC_SOURCES` | `number` | `4` | Maximum distinct primitive CoC sources in an interrupted camera blend; identical sources coalesce before overflow selection. |

```ogt
const MAX_CAMERA_COC_SOURCES u32 4 "Maximum distinct primitive CoC sources in an interrupted camera blend; identical sources coalesce before overflow selection."
```

<sub>Module `camera` line 245</sub>
