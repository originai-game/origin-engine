<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`input`](../input.md) · [index](../index.md)

<a id="type-GamepadButton"></a>

### `GamepadButton` (enum)

A gamepad button in the W3C standard mapping, in its index order (a is button 0).

A string: one of the variants below. The zero value is `"a"`.

| Variant | Doc |
|---|---|
| `"a"` | The bottom face button (A, Cross). |
| `"b"` | The right face button (B, Circle). |
| `"x"` | The left face button (X, Square). |
| `"y"` | The top face button (Y, Triangle). |
| `"leftBumper"` | The left shoulder button. |
| `"rightBumper"` | The right shoulder button. |
| `"leftTrigger"` | The left trigger, as a button (its analog value is the leftTrigger axis). |
| `"rightTrigger"` | The right trigger, as a button. |
| `"back"` | Back, Select or View. |
| `"start"` | Start, Options or Menu. |
| `"leftStick"` | The left stick pressed in. |
| `"rightStick"` | The right stick pressed in. |
| `"up"` | The d-pad's up. |
| `"down"` | The d-pad's down. |
| `"left"` | The d-pad's left. |
| `"right"` | The d-pad's right. |
| `"home"` | The home or guide button. |

```ogt
enum GamepadButton {
  doc "A gamepad button in the W3C standard mapping, in its index order (a is button 0)."
  variant a "The bottom face button (A, Cross)."
  variant b "The right face button (B, Circle)."
  variant x "The left face button (X, Square)."
  variant y "The top face button (Y, Triangle)."
  variant leftBumper "The left shoulder button."
  variant rightBumper "The right shoulder button."
  variant leftTrigger "The left trigger, as a button (its analog value is the leftTrigger axis)."
  variant rightTrigger "The right trigger, as a button."
  variant back "Back, Select or View."
  variant start "Start, Options or Menu."
  variant leftStick "The left stick pressed in."
  variant rightStick "The right stick pressed in."
  variant up "The d-pad's up."
  variant down "The d-pad's down."
  variant left "The d-pad's left."
  variant right "The d-pad's right."
  variant home "The home or guide button."
}
```

<sub>`api.ogt` line 2535</sub>
