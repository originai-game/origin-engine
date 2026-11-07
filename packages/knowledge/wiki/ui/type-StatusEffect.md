<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-StatusEffect"></a>

### `StatusEffect` (desc struct)

A buff or debuff on a unit.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `key` | `string` | `""` | — | — | — | Its key (its probe id). |
| `icon` | `string` | `""` | — | — | — | An icon of the atlas. |
| `image` | `Texture \| null` | `null` | — | — | — | Its art (art kind buff, 1:1). |
| `name` | `string` | `""` | — | — | — | What it is (Haste); what a screen reader says. |
| `stacks` | `number` | `0` | — | — | — | Stacks (a count at its corner from 2 on). |
| `remaining` | `number` | `0` | ≥ 0 | `s` | — | Seconds left; 0 lasts until removed. |
| `duration` | `number` | `0` | ≥ 0 | `s` | — | Its whole length, for the sweep. |
| `debuff` | `boolean` | `false` | — | — | — | Harmful: in the debuff row, with a red border and a notch. |
| `type` | `string` | `""` | — | — | — | A debuff's type for its border colour (magic, curse, poison, disease, bleed); empty is plain. |
| `mine` | `boolean` | `false` | — | — | — | Cast by the player (drawn larger in a target's grid). |

```ogt
struct StatusEffect {
  doc "A buff or debuff on a unit."
  desc true
  field key String "Its key (its probe id)."
  field icon String "An icon of the atlas."
  field image Option.Texture "Its art (art kind buff, 1:1)."
  field name String "What it is (Haste); what a screen reader says."
  field stacks u32 "Stacks (a count at its corner from 2 on)."
  field remaining f32 "Seconds left; 0 lasts until removed." {
    min 0
    unit s
  }
  field duration f32 "Its whole length, for the sweep." {
    min 0
    unit s
  }
  field debuff bool "Harmful: in the debuff row, with a red border and a notch."
  field type String "A debuff's type for its border colour (magic, curse, poison, disease, bleed); empty is plain."
  field mine bool "Cast by the player (drawn larger in a target's grid)."
}
```

<sub>Module `ui` line 6100</sub>
