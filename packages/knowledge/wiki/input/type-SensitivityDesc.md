<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`input`](../input.md) · [index](../index.md)

<a id="type-SensitivityDesc"></a>

### `SensitivityDesc` (desc struct)

The player's sensitivity and inversion of an axis action on one device class: a look action's mouse and its stick each have their own (a settings screen's Mouse sensitivity and Stick sensitivity sliders, Invert Y). Kept by og.input.saveBindings, forgotten by og.input.resetBindings; a world whose player set none saves and hashes as before.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `action` | `string` | `""` | — | — | — | The action (axis or axis2). |
| `device` | `InputDevice` | `"keyboardMouse"` | — | — | — | The device class it is for: keyboardMouse (the mouse's movement and the wheel; inversion turns the keys over too), any pad family (every pad), or touch (a touch stick). |
| `scale` | `number` | `1` | 0.01 … 100 | — | — | Multiplies the value of the action's analog bindings on that class (the mouse's movement, the wheel, a stick, a trigger, a touch stick), after each binding's own scale: 2 turns twice as far for the same mouse movement. Keys, buttons, the d-pad and composites read whole. |
| `invertX` | `boolean` | `false` | — | — | — | The value's x (an axis action's only value) is turned over. |
| `invertY` | `boolean` | `false` | — | — | — | A two-axis value's y is turned over: inverted look (an action's y is up-positive, so the mouse moved up then reads down). |

```ogt
struct SensitivityDesc {
  doc "The player's sensitivity and inversion of an axis action on one device class: a look action's mouse and its stick each have their own (a settings screen's Mouse sensitivity and Stick sensitivity sliders, Invert Y). Kept by og.input.saveBindings, forgotten by og.input.resetBindings; a world whose player set none saves and hashes as before."
  desc true
  field action String "The action (axis or axis2)."
  field device InputDevice "The device class it is for: keyboardMouse (the mouse's movement and the wheel; inversion turns the keys over too), any pad family (every pad), or touch (a touch stick)."
  field scale f32 "Multiplies the value of the action's analog bindings on that class (the mouse's movement, the wheel, a stick, a trigger, a touch stick), after each binding's own scale: 2 turns twice as far for the same mouse movement. Keys, buttons, the d-pad and composites read whole." {
    default 1
    min 0.01
    max 100
  }
  field invertX bool "The value's x (an axis action's only value) is turned over."
  field invertY bool "A two-axis value's y is turned over: inverted look (an action's y is up-positive, so the mouse moved up then reads down)."
}
```

<sub>`api.ogt` line 2209</sub>
