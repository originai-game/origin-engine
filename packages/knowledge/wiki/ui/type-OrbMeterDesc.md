<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-OrbMeterDesc"></a>

### `OrbMeterDesc` (desc struct)

An orb of health or mana.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `value` | `number` | `0` | — | — | — | The value. |
| `max` | `number` | `100` | — | — | — | Its maximum. |
| `kind` | `MeterKind` | `"health"` | — | — | — | Its liquid's colour: health (red) or mana (blue). |
| `color` | `Paint \| null` | `null` | — | — | — | A colour of its own. |
| `size` | `number` | `0` | ≥ 0 | `cssPx` | — | Its diameter; 0 is 112. |
| `showValue` | `boolean` | `true` | — | — | — | The value in the middle (the default). |
| `label` | `string` | `""` | — | — | — | A caption under it; empty is none. |
| `anchor` | `Anchor \| null` | `null` | — | — | — | Opens its own anchored group there; null builds it in place. |
| `name` | `string` | `""` | — | — | — | What a screen reader calls it; empty is the label, else Health. |

```ogt
struct OrbMeterDesc {
  doc "An orb of health or mana."
  desc true
  field value f32 "The value."
  field max f32 "Its maximum." {
    default 100
  }
  field kind MeterKind "Its liquid's colour: health (red) or mana (blue)." {
    default health
  }
  field color Option.Paint "A colour of its own."
  field size f32 "Its diameter; 0 is 112." {
    min 0
    unit cssPx
  }
  field showValue bool "The value in the middle (the default)." {
    default true
  }
  field label String "A caption under it; empty is none."
  field anchor Option.Anchor "Opens its own anchored group there; null builds it in place."
  field name String "What a screen reader calls it; empty is the label, else Health."
}
```

<sub>Module `ui` line 5955</sub>
