<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-StatCompareRow"></a>

### `StatCompareRow` (desc struct)

A stat compared: its value now and the value it would take.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `label` | `string` | `""` | — | — | — | The stat. |
| `current` | `number` | `0` | — | — | — | Its value now. |
| `next` | `number \| null` | `null` | — | — | — | The value it would take; null is no change. |
| `max` | `number \| null` | `null` | — | — | — | The stat's scale: a bar shows the values and the change; null is no bar. |
| `lowerIsBetter` | `boolean` | `false` | — | — | — | Less is better (a cooldown, a cost): a fall is green. |
| `format` | `NumberFormat` | `"grouped"` | — | — | — | How its values read. |
| `digits` | `number` | `0` | — | — | — | Decimals. |
| `unit` | `string` | `""` | — | — | — | A unit after them. |
| `icon` | `string` | `""` | — | — | — | An icon before the label; empty is none. |

```ogt
struct StatCompareRow {
  doc "A stat compared: its value now and the value it would take."
  desc true
  field label String "The stat."
  field current f32 "Its value now."
  field next Option.f32 "The value it would take; null is no change."
  field max Option.f32 "The stat's scale: a bar shows the values and the change; null is no bar."
  field lowerIsBetter bool "Less is better (a cooldown, a cost): a fall is green."
  field format NumberFormat "How its values read."
  field digits u32 "Decimals."
  field unit String "A unit after them."
  field icon String "An icon before the label; empty is none."
}
```

<sub>Module `ui` line 2743</sub>
