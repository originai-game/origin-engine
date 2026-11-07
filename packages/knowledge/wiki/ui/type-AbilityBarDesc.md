<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-AbilityBarDesc"></a>

### `AbilityBarDesc` (desc struct)

A bar of ability slots.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `slots` | `readonly AbilitySlotDesc[]` | `[]` | — | — | — | The slots, in order; each one's key is its id. |
| `size` | `number` | `0` | ≥ 0 | `cssPx` | — | Each slot's side; 0 is 56 (48 on a phone). |
| `gap` | `number` | `0` | ≥ 0 | `cssPx` | — | Between slots; 0 is the style's sm space. |
| `anchor` | `Anchor \| null` | `"bottom"` | — | — | — | Opens its own anchored group there; null builds it in place. |
| `direction` | `Direction` | `"row"` | — | — | — | Which way the slots run. |

```ogt
struct AbilityBarDesc {
  doc "A bar of ability slots."
  desc true
  field slots List.AbilitySlotDesc "The slots, in order; each one's key is its id."
  field size f32 "Each slot's side; 0 is 56 (48 on a phone)." {
    min 0
    unit cssPx
  }
  field gap f32 "Between slots; 0 is the style's sm space." {
    min 0
    unit cssPx
  }
  field anchor Option.Anchor "Opens its own anchored group there; null builds it in place." {
    default bottom
  }
  field direction Direction "Which way the slots run."
}
```

<sub>Module `ui` line 6067</sub>
