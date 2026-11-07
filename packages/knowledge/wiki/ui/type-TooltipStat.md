<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-TooltipStat"></a>

### `TooltipStat` (desc struct)

A row of a tooltip's stats, with an optional comparison.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `label` | `string` | `""` | — | — | — | What it is (Damage). |
| `value` | `string` | `""` | — | — | — | Its value as shown (42–58). |
| `delta` | `number` | `0` | — | — | — | Against what is compared (the equipped item): positive is better, drawn with an up mark in success; negative with a down mark in danger; 0 none. |
| `deltaText` | `string` | `""` | — | — | — | The delta as shown (+12); empty formats delta. |
| `better` | `boolean` | `false` | — | — | — | Higher is worse for this stat (a cooldown): flips the marks' colours. |

```ogt
struct TooltipStat {
  doc "A row of a tooltip's stats, with an optional comparison."
  desc true
  field label String "What it is (Damage)."
  field value String "Its value as shown (42–58)."
  field delta f32 "Against what is compared (the equipped item): positive is better, drawn with an up mark in success; negative with a down mark in danger; 0 none."
  field deltaText String "The delta as shown (+12); empty formats delta."
  field better bool "Higher is worse for this stat (a cooldown): flips the marks' colours."
}
```

<sub>Module `ui` line 5423</sub>
