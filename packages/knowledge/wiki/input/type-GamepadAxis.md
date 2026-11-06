<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`input`](../input.md) · [index](../index.md)

<a id="type-GamepadAxis"></a>

### `GamepadAxis` (enum)

An analog input of a gamepad.

A string: one of the variants below. The zero value is `"leftX"`.

| Variant | Doc |
|---|---|
| `"leftX"` | The left stick's x, -1 (left) to 1 (right). |
| `"leftY"` | The left stick's y, -1 (up) to 1 (down), as the Gamepad API reports it. |
| `"rightX"` | The right stick's x. |
| `"rightY"` | The right stick's y, down positive. |
| `"leftTrigger"` | The left trigger, 0 to 1. |
| `"rightTrigger"` | The right trigger, 0 to 1. |

```ogt
enum GamepadAxis {
  doc "An analog input of a gamepad."
  variant leftX "The left stick's x, -1 (left) to 1 (right)."
  variant leftY "The left stick's y, -1 (up) to 1 (down), as the Gamepad API reports it."
  variant rightX "The right stick's x."
  variant rightY "The right stick's y, down positive."
  variant leftTrigger "The left trigger, 0 to 1."
  variant rightTrigger "The right trigger, 0 to 1."
}
```

<sub>Module `input` line 556</sub>
