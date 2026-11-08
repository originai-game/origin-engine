<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-DpadDesc"></a>

### `DpadDesc` (desc struct)

A d-pad: one round pad (part Dpad.base, which a style dresses) with four chevrons on it, read as one target by the angle of the finger from its centre: eight ways (a diagonal holds both of its arms), nothing in its middle, the held direction lit under its chevron. The thumb rolls between directions without lifting.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `size` | `number` | `0` | ≥ 0 | `cssPx` | — | The pad's size; 0 is 132 (each arm at least 44). |
| `customize` | `Customize \| null` | `null` | — | — | — | The player can move, scale and fade it in the HUD edit mode (og.ui.setHudEdit). |
| `disabled` | `boolean` | `false` | — | — | — | It answers nothing and shows faded. |

```ogt
struct DpadDesc {
  doc "A d-pad: one round pad (part Dpad.base, which a style dresses) with four chevrons on it, read as one target by the angle of the finger from its centre: eight ways (a diagonal holds both of its arms), nothing in its middle, the held direction lit under its chevron. The thumb rolls between directions without lifting."
  desc true
  field size f32 "The pad's size; 0 is 132 (each arm at least 44)." {
    unit cssPx
    min 0
  }
  field customize Option.Customize "The player can move, scale and fade it in the HUD edit mode (og.ui.setHudEdit)."
  field disabled bool "It answers nothing and shows faded."
}
```

<sub>Module `ui` line 1105</sub>
