<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`input`](../input.md) · [index](../index.md)

<a id="fn-moveRelative"></a>

### `og.input.moveRelative`

A two-axis action as a horizontal world vector seen from a camera: the action's y goes into the screen and its x to the screen's right, whatever the camera's pitch, so W walks away from the camera wherever it has orbited. The length is the action's (a stick's tilt is kept) and y is 0. A 2D game uses og.input.actionVector as it is.

See `facing`.

Also known as: camera relative movement, WASD to world direction, move direction, third person movement, strafe

```ts
og.input.moveRelative(world: World, name: string, camera: Quaternion): Vector3
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `name` | `string` | no | The two-axis action. |
| `camera` | `Quaternion` | no | The camera's rotation (og.Camera.quaternion). |

**Returns** `Vector3`.

Binding symbol `og_input_move_relative` · publish builds: kept

```ogt
fn moveRelative {
  doc "A two-axis action as a horizontal world vector seen from a camera: the action's y goes into the screen and its x to the screen's right, whatever the camera's pitch, so W walks away from the camera wherever it has orbited. The length is the action's (a stick's tilt is kept) and y is 0. A 2D game uses og.input.actionVector as it is."
  aliases "camera relative movement, WASD to world direction, move direction, third person movement, strafe"
  guide "facing"
  param world World "The world."
  param name String "The two-axis action."
  param camera Quaternion "The camera's rotation (og.Camera.quaternion)."
  returns Vector3
}
```

<sub>Module `input` line 393</sub>
