<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-StatCompareDesc"></a>

### `StatCompareDesc` (desc struct)

Stat-compare rows: an equip screen's comparison, each change a signed delta with a mark.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `rows` | `readonly StatCompareRow[]` | `[]` | — | — | — | The stats. |
| `currentLabel` | `string` | `""` | — | — | — | A header over the current values (`Equipped`); empty is no header. |
| `nextLabel` | `string` | `""` | — | — | — | A header over the new values (`Selected`). |
| `dividers` | `boolean` | `false` | — | — | — | Hairlines between rows. |
| `width` | `Size \| null` | `null` | — | — | — | Its width; null grows. |
| `label` | `string` | `""` | — | — | — | What a screen reader calls the list. |

```ogt
struct StatCompareDesc {
  doc "Stat-compare rows: an equip screen's comparison, each change a signed delta with a mark."
  desc true
  field rows List.StatCompareRow "The stats."
  field currentLabel String "A header over the current values (`Equipped`); empty is no header."
  field nextLabel String "A header over the new values (`Selected`)."
  field dividers bool "Hairlines between rows."
  field width Option.Size "Its width; null grows."
  field label String "What a screen reader calls the list."
}
```

<sub>Module `ui` line 2757</sub>
