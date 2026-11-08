<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-JoystickDesc"></a>

### `JoystickDesc` (desc struct)

A virtual joystick: one translucent disc of the HUD ground with a rim (part Joystick.base, which a style dresses) and a knob that follows the finger that started on it, the rim lit toward the push while held. A finger sliding onto it from elsewhere does not take it.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `size` | `number` | `0` | ≥ 0 | `cssPx` | — | The base's diameter; 0 is 128 (never under three touch targets). |
| `deadZone` | `number` | `0` | 0 … 1 | `ratio` | — | The ignored radius; 0 is 0.15. Past it the vector is rescaled, rising from 0 at the dead zone's edge to 1 at the base's. |
| `customize` | `Customize \| null` | `null` | — | — | — | The player can move, scale and fade it in the HUD edit mode (og.ui.setHudEdit). |
| `disabled` | `boolean` | `false` | — | — | — | It answers nothing and shows faded (a cutscene, a stun). |

```ogt
struct JoystickDesc {
  doc "A virtual joystick: one translucent disc of the HUD ground with a rim (part Joystick.base, which a style dresses) and a knob that follows the finger that started on it, the rim lit toward the push while held. A finger sliding onto it from elsewhere does not take it."
  desc true
  field size f32 "The base's diameter; 0 is 128 (never under three touch targets)." {
    unit cssPx
    min 0
  }
  field deadZone f32 "The ignored radius; 0 is 0.15. Past it the vector is rescaled, rising from 0 at the dead zone's edge to 1 at the base's." {
    unit ratio
    min 0
    max 1
  }
  field customize Option.Customize "The player can move, scale and fade it in the HUD edit mode (og.ui.setHudEdit)."
  field disabled bool "It answers nothing and shows faded (a cutscene, a stun)."
}
```

<sub>Module `ui` line 1118</sub>
