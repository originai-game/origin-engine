<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-JoystickResult"></a>

### `JoystickResult` (struct)

A joystick's vector this tick.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `x` | `number` | `0` | — | — | — | -1 left to 1 right; 0 inside the dead zone. |
| `y` | `number` | `0` | — | — | — | -1 up to 1 down; 0 inside the dead zone. |
| `active` | `boolean` | `false` | — | — | — | A press that started on it is held. |

```ogt
struct JoystickResult {
  doc "A joystick's vector this tick."
  field x f32 "-1 left to 1 right; 0 inside the dead zone."
  field y f32 "-1 up to 1 down; 0 inside the dead zone."
  field active bool "A press that started on it is held."
}
```

<sub>`api.ogt` line 16769</sub>
