<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

# The `camera` module

The main camera, as a resource with three.js PerspectiveCamera and OrthographicCamera parameter names.

Camera controllers are components (OrbitCamera, FollowCamera, Follow2D) on the entity the camera orbits or follows; the systems that drive them ship in TypeScript as og.systems.cameras.

- **TypeScript**: functions are the namespace `og.camera`; components, resources and events are tokens on `og`.
- **Publish builds**: kept (members may override).
- **Index**: [every module and guide](index.md).

```ogt
module camera {
  doc "The main camera, as a resource with three.js PerspectiveCamera and OrthographicCamera parameter names."
  doc "Camera controllers are components (OrbitCamera, FollowCamera, Follow2D) on the entity the camera orbits or follows; the systems that drive them ship in TypeScript as og.systems.cameras."
  # declarations follow, each on its own page
```

<sub>`api.ogt` line 3216</sub>

## Contents

| Name | Kind | Signature | Summary |
|---|---|---|---|
| [`og.Camera`](camera/record-Camera.md) | resource |  | The camera the frame is rendered from. |
| [`og.OrbitCamera`](camera/record-OrbitCamera.md) | component |  | Orbits og.Camera around the entity carrying it, as three.js OrbitControls: a left-button drag on the play surface turns it, the wheel zooms, autoRotate turns it on its own. |
| [`og.FollowCamera`](camera/record-FollowCamera.md) | component |  | Chases the entity carrying it with og.Camera: a third-person 3D view that eases towards a point offset from the entity and looks at it. |
| [`og.Follow2D`](camera/record-Follow2D.md) | component |  | Keeps og.Camera on the entity carrying it in the xy plane, for 2D games: the camera moves only when the entity leaves the dead zone, eases by smoothTime (smoothTimeY for y when set) and, with clamp, … |
| [`og.CameraZone`](camera/record-CameraZone.md) | component |  | A rectangle of the xy plane (min to max, world units, not moved by the entity's Transform) that keeps a 2D view inside it while the entity followed by og.Follow2D is in it: rooms, arenas, a level's b… |
| [`og.CameraRail`](camera/record-CameraRail.md) | component |  | Moves og.Camera along a spline through p0 to p(points - 1) (Catmull-Rom, through every point) to the point nearest the entity carrying it, easing by smoothTime: a corridor, a side-scroller's track, a… |
| [`og.VirtualCamera`](camera/record-VirtualCamera.md) | component |  | A shot og.Camera can take, as Cinemachine's virtual cameras: a position (its own entity's, or following another entity), what it looks at, a field of view, roll and focus. |
| [`og.CameraBrain`](camera/record-CameraBrain.md) | resource |  | Picks the live og.VirtualCamera every tick and blends og.Camera to it, as Cinemachine's brain: the solo shot if one is set, else the enabled one with the highest priority. |
| [`og.CameraCut`](camera/record-CameraCut.md) | event |  | The live virtual camera changed (sent by the og.camera system; read the tick after). |
| [`og.camera.shake`](camera/fn-shake.md) | function | `(world: World, trauma: number): Result<void>` | Add screen shake: trauma, clamped to 1, which og.Camera decays by traumaDecay. |
| [`og.camera.punch`](camera/fn-punch.md) | function | `(world: World, amount: number): Result<void>` | A zoom punch: the view narrows by `amount` (0.05 is a tap, 0.15 a slam) and springs back by Camera.punchDecay. |
| [`og.camera.kick`](camera/fn-kick.md) | function | `(world: World, direction: Vector3, distance: number): Result<void>` | A directional nudge of the view (recoil, an impact from one side): offset by `distance` along `direction`, returning by Camera.kickDecay. |
