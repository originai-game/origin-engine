<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-StatusGridDesc"></a>

### `StatusGridDesc` (desc struct)

A grid of buffs and debuffs.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `effects` | `readonly StatusEffect[]` | `[]` | — | — | — | The effects, in the game's order. |
| `columns` | `number` | `0` | — | — | — | Per row; 0 is 8. |
| `size` | `number` | `0` | ≥ 0 | `cssPx` | — | An icon's side; 0 is 32. |
| `max` | `number` | `0` | — | — | — | The most shown; more fold into a +n tile; 0 is no limit. |
| `timers` | `boolean` | `true` | — | — | — | The time left under each icon (the default). |
| `split` | `boolean` | `true` | — | — | — | Debuffs in a row of their own under the buffs (the default). |
| `anchor` | `Anchor \| null` | `null` | — | — | — | Opens its own anchored group there; null builds it in place. |

```ogt
struct StatusGridDesc {
  doc "A grid of buffs and debuffs."
  desc true
  field effects List.StatusEffect "The effects, in the game's order."
  field columns u32 "Per row; 0 is 8."
  field size f32 "An icon's side; 0 is 32." {
    min 0
    unit cssPx
  }
  field max u32 "The most shown; more fold into a +n tile; 0 is no limit."
  field timers bool "The time left under each icon (the default)." {
    default true
  }
  field split bool "Debuffs in a row of their own under the buffs (the default)." {
    default true
  }
  field anchor Option.Anchor "Opens its own anchored group there; null builds it in place."
}
```

<sub>`api.ogt` line 21872</sub>
