# Facing: the axes, forward and camera-relative movement

Which way things face in the engine, how to turn them and read their directions, and how to turn a stick or WASD into a world direction that matches the screen. Read before writing movement, aiming or a camera rig; it is the one place the axes are stated.

The API: `og api show transform`, `og api show input`, `og api show camera`.

## 1. The axes

The world is right-handed with +Y up. What faces which way:

| What | Forward | Right | Up |
|---|---|---|---|
| An object: a character, a vehicle's chassis, a nav or AI agent, a path follower, a sound source's cone | local +Z | local -X | local +Y |
| A camera, a light and what projects like one: og.Decal, og.RectLight, og.AudioListener, a directional og.WindMotor | local -Z | local +X | local +Y |
| Anything on the 2D plane (x right, y up) | local +X | local -Y | turns about +Z |

In every row right is forward × up. An object faces +Z as a glTF model
does (`models` covers import), so a model needs no turn to walk forwards. A
camera looks down -Z, so a camera at +Z looking at the origin has the
identity rotation.

A heading is an angle about +Y from +Z towards +X: 0 faces +Z, a quarter
turn faces +X. In 2D it is the angle about +Z from +X towards +Y.

## 2. Turning things and reading their directions

Never write the axes out by hand; ask for them:

```ts
// Turn a character to face a point, and a camera to look at it.
world.set(hero, og.Transform, { quaternion: og.transform.face(heroPosition, point) });
must(world.resource(og.Camera)).lookAt(point); // or { quaternion: og.transform.lookAt(eye, point) }

// The directions of a rotation.
const ahead = og.transform.forward(heroRotation);        // an object's +Z
const side = og.transform.right(heroRotation);           // an object's -X
const view = og.transform.cameraForward(cameraRotation); // a camera's -Z
const screenRight = og.transform.cameraRight(cameraRotation);

// A heading about +Y, and the rotation of one.
const heading = og.transform.heading(heroRotation);      // of a direction: math.atan2(d.x, d.z)
const turned = og.transform.fromHeading(heading + turnSpeed * dt);
```

On the 2D plane the calls end in `2d`: a thing faces its +X and turns
about +Z, so it stays in the plane.

```ts
// Turn a sprite to face a point, and read the way it faces.
world.set(ship, og.Transform, { quaternion: og.transform.face2d(shipPosition, point) });
const ahead2d = og.transform.forward2d(shipRotation);    // a 2D thing's +X; z is 0

// A 2D heading about +Z, and the rotation of one.
const angle = og.transform.heading2d(shipRotation);      // of a direction: math.atan2(d.y, d.x)
const spun = og.transform.fromHeading2d(angle + turnSpeed * dt);
```

- `og.transform.face` is for objects and `og.transform.lookAt` for cameras
  and lights. Using the wrong one turns the thing to face away.
- In a 2D game use `og.transform.face2d`, never `og.transform.face`: the
  3D call points the sprite's +Z at the target and turns it edge-on.
- `og.transform.right` of a camera's rotation is the camera's left: use
  `og.transform.cameraRight`.
- What the engine turns for you follows the table: nav's `faceMovement`,
  og.PathFollower's `align`, a vehicle's chassis and an AI agent's
  `forward` use an object's +Z; og.OrbitCamera, the virtual cameras and
  shadows use -Z.
- An offset that follows an entity's rotation (og.FollowCamera and
  og.VirtualCamera with `followRotation`, og.PathFollower's `offset`) is x
  to the right, y up and z backwards: `{ x: 0, y: 2, z: 6 }` is two metres
  above and six behind.

## 3. Input to world

The directions input reports:

| Read | x | y |
|---|---|---|
| `og.input.actionVector` (an `axis2` action) | right is +1 | up or forward is +1 (W, a stick pushed away) |
| og.Pointers.movement, a pointer's x and y | right is positive | down is positive (CSS px) |
| og.Gamepad's raw stick axes | right is +1 | down is +1 (the Gamepad API's) |

Read movement through an action, never the raw axes, so the sign is the
same for keys, pads and touch.

In a 3D game the action's y has to become a world direction, and which one
depends on the camera. `og.input.moveRelative` does it: y goes into the
screen, x to the screen's right, flat on the ground whatever the camera's
pitch, with the action's length kept.

```ts
// in a system that reads og.Camera (its pose at the end of the last tick)
const camera = must(world.resource(og.Camera));
const move = og.input.moveRelative(world, "move", camera.quaternion);
if (move.x !== 0 || move.z !== 0) {
  position = { x: position.x + move.x * speed * dt, y: position.y, z: position.z + move.z * speed * dt };
  rotation = og.transform.face({ x: 0, y: 0, z: 0 }, move); // face the way it walks
}
```

W then walks away from the camera and D to the screen's right wherever the
camera has orbited. For movement fixed to the world instead (a top-down
board), map the action yourself once: `{ x: v.x, y: 0, z: -v.y }` makes W
go towards -Z, the top of the screen for a camera that looks down -Z from
above. In 2D, `actionVector` is already world x and y.

For a tank or a car, steer by the object's own axes: `og.transform.forward`
of its rotation times the action's y.

## 4. From three.js

| three.js | Origin Engine |
|---|---|
| `camera.lookAt(target)`, `light.lookAt(target)` | `og.transform.lookAt(eye, target)` |
| `mesh.lookAt(target)` | `og.transform.face(position, target)` |
| `object.getWorldDirection(v)` (a mesh: its +Z) | `og.transform.forward(quaternion)` |
| `camera.getWorldDirection(v)` (its -Z) | `og.transform.cameraForward(quaternion)` |
| `new Vector3(1, 0, 0).applyQuaternion(camera.quaternion)` | `og.transform.cameraRight(quaternion)` |
| the hand-written "forward = camera direction with y = 0, right = forward × up" block | `og.input.moveRelative(world, action, camera.quaternion)` |
| `Math.atan2(dx, dz)` for a yaw | `math.atan2(dx, dz)`, the same angle (`determinism §1`); of a rotation, `og.transform.heading(quaternion)` |
| `object.rotation.y = yaw` | `og.transform.fromHeading(yaw)` |
| `sprite.rotation.z = Math.atan2(dy, dx)` (2D) | `og.transform.face2d(position, target)`, or `og.transform.fromHeading2d(angle)` |

The axes are three.js's, so positions and rotations port unchanged. What
ports badly is hand-written sign logic: replace it with the calls above
instead of copying it.
