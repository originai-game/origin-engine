<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`input`](../input.md) · [index](../index.md)

<a id="type-InputBinding"></a>

### `InputBinding` (desc struct)

One binding of an action: a control, or a composite of parts. A binding with neither a control nor parts is an unbound slot.

Controls are strings: a key code (`KeyW`, `Space`); `mouse.left`, `mouse.middle`, `mouse.right`, `mouse.x`, `mouse.y`, `mouse.xy`; `wheel.x`, `wheel.y`; `pad.a` to `pad.home` (GamepadButton's names); `pad.leftX`, `pad.leftY`, `pad.rightX`, `pad.rightY`, `pad.leftTrigger`, `pad.rightTrigger`, `pad.leftXY`, `pad.rightXY`, `pad.dpad`; `touch` and `touch.xy`. A one-axis control takes `+` or `-` for its half (`pad.leftX+`). Every y is up-positive.

See `input §1`.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `control` | `string` | `""` | — | — | — | The control; empty for a composite or an unbound slot. |
| `negative` | `string` | `""` | — | — | — | An axis composite's control for -1 (`KeyA`). |
| `positive` | `string` | `""` | — | — | — | An axis composite's control for +1 (`KeyD`). |
| `up` | `string` | `""` | — | — | — | A two-axis composite's control for y = 1 (`KeyW`). |
| `down` | `string` | `""` | — | — | — | A two-axis composite's control for y = -1. |
| `left` | `string` | `""` | — | — | — | A two-axis composite's control for x = -1. |
| `right` | `string` | `""` | — | — | — | A two-axis composite's control for x = 1. |
| `scale` | `number` | `1` | — | — | — | Multiplies the value: the binding's sensitivity (a mouse look's 0.1, a stick's 2); negative inverts it. |
| `invertY` | `boolean` | `false` | — | — | — | A two-axis value's y is turned over (inverted look). |
| `deadzone` | `number` | `0` | 0 … 0.95 | — | — | A pad axis's dead zone for this binding, over the pad's (og.input.setDeadzones) when above 0: values inside it read 0 and the rest is rescaled to reach 1. |
| `zone` | `TouchZone` | `{}` | — | — | — | `touch`, `touch.xy`: where a touch must start to count (empty: anywhere). |
| `radius` | `number` | `0` | ≥ 0 | `cssPx` | — | `touch.xy`: how far a touch drags for a full deflection (0: 64). |

```ogt
struct InputBinding {
  doc "One binding of an action: a control, or a composite of parts. A binding with neither a control nor parts is an unbound slot."
  doc "Controls are strings: a key code (`KeyW`, `Space`); `mouse.left`, `mouse.middle`, `mouse.right`, `mouse.x`, `mouse.y`, `mouse.xy`; `wheel.x`, `wheel.y`; `pad.a` to `pad.home` (GamepadButton's names); `pad.leftX`, `pad.leftY`, `pad.rightX`, `pad.rightY`, `pad.leftTrigger`, `pad.rightTrigger`, `pad.leftXY`, `pad.rightXY`, `pad.dpad`; `touch` and `touch.xy`. A one-axis control takes `+` or `-` for its half (`pad.leftX+`). Every y is up-positive."
  guide "input §1"
  desc true
  field control String "The control; empty for a composite or an unbound slot."
  field negative String "An axis composite's control for -1 (`KeyA`)."
  field positive String "An axis composite's control for +1 (`KeyD`)."
  field up String "A two-axis composite's control for y = 1 (`KeyW`)."
  field down String "A two-axis composite's control for y = -1."
  field left String "A two-axis composite's control for x = -1."
  field right String "A two-axis composite's control for x = 1."
  field scale f32 "Multiplies the value: the binding's sensitivity (a mouse look's 0.1, a stick's 2); negative inverts it." {
    default 1
  }
  field invertY bool "A two-axis value's y is turned over (inverted look)."
  field deadzone f32 "A pad axis's dead zone for this binding, over the pad's (og.input.setDeadzones) when above 0: values inside it read 0 and the rest is rescaled to reach 1." {
    min 0
    max 0.95
  }
  field zone TouchZone "`touch`, `touch.xy`: where a touch must start to count (empty: anywhere)."
  field radius f32 "`touch.xy`: how far a touch drags for a full deflection (0: 64)." {
    min 0
    unit cssPx
  }
}
```

<sub>Module `input` line 92</sub>
