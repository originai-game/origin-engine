<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`input`](../input.md) · [index](../index.md)

<a id="fn-rumble"></a>

### `og.input.rumble`

Rumble a pad (or every connected pad): the web shell plays it on the pad's vibration actuator. Output only: it never feeds back into the simulation, and a native run records it in the frame answer's `rumble`.

Also known as: vibration, vibrate, haptics, force feedback, gamepad motors, Unity Gamepad.SetMotorSpeeds, Godot Input.start_joy_vibration

```ts
og.input.rumble(world: World, desc?: RumbleDesc): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `desc` | `RumbleDesc` | yes | The pad, the motors and how long. |

**Returns** `Result<void>`: Refused for a pad outside -1 to 7 or a value that is not finite.

Binding symbol `og_input_rumble` · publish builds: kept

```ogt
fn rumble {
  doc "Rumble a pad (or every connected pad): the web shell plays it on the pad's vibration actuator. Output only: it never feeds back into the simulation, and a native run records it in the frame answer's `rumble`."
  aliases "vibration, vibrate, haptics, force feedback, gamepad motors, Unity Gamepad.SetMotorSpeeds, Godot Input.start_joy_vibration"
  param world World "The world."
  param desc RumbleDesc "The pad, the motors and how long."
  returns Result.Void "Refused for a pad outside -1 to 7 or a value that is not finite."
}
```

<sub>Module `input` line 565</sub>
