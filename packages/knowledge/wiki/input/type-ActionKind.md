<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`input`](../input.md) · [index](../index.md)

<a id="type-ActionKind"></a>

### `ActionKind` (enum)

What an action gives.

A string: one of the variants below. The zero value is `"button"`.

| Variant | Doc |
|---|---|
| `"button"` | On or off: down, pressed, released, held. An analog control (a trigger, a half axis) presses it at ActionDesc.threshold and lets go under three quarters of it. |
| `"axis"` | One value, -1 to 1: A and D, a stick's x, a trigger (0 to 1), the wheel, the mouse's x movement (CSS px). |
| `"axis2"` | Two values, x right and y up: WASD, a stick, the d-pad, a touch stick, the mouse's movement. |

```ogt
enum ActionKind {
  doc "What an action gives."
  variant button "On or off: down, pressed, released, held. An analog control (a trigger, a half axis) presses it at ActionDesc.threshold and lets go under three quarters of it."
  variant axis "One value, -1 to 1: A and D, a stick's x, a trigger (0 to 1), the wheel, the mouse's x movement (CSS px)."
  variant axis2 "Two values, x right and y up: WASD, a stick, the d-pad, a touch stick, the mouse's movement."
}
```

<sub>`api.ogt` line 2036</sub>
