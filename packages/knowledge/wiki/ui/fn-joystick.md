<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-joystick"></a>

### `og.ui.joystick`

A virtual joystick: a base and a knob that follows a press that started on it. It keeps the finger that grabbed it (any of og.Pointers), so another finger can hold an action button at the same time.

```ts
og.ui.joystick(id: string, desc?: JoystickDesc): JoystickResult
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `desc` | `JoystickDesc` | yes | Size and dead zone. |

**Returns** `JoystickResult`.

Binding symbol `og_ui_joystick` · publish builds: kept

```ogt
fn joystick {
  doc "A virtual joystick: a base and a knob that follows a press that started on it. It keeps the finger that grabbed it (any of og.Pointers), so another finger can hold an action button at the same time."
  param id String "Its id."
  param desc JoystickDesc "Size and dead zone."
  returns JoystickResult
}
```

<sub>Module `ui` line 1533</sub>
