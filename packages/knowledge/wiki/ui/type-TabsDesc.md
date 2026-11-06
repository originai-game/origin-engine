<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-TabsDesc"></a>

### `TabsDesc` (desc struct)

A segmented control of tabs.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `fill` | `boolean` | `false` | — | — | — | Equal segments across the width. |
| `width` | `Size \| null` | `null` | — | — | — | Its width; null fits (grows with fill). |
| `label` | `string` | `""` | — | — | — | What a screen reader calls the row of tabs (its accessible name), never drawn; empty is its id. A localised game passes its string table's. |

```ogt
struct TabsDesc {
  doc "A segmented control of tabs."
  desc true
  field fill bool "Equal segments across the width."
  field width Option.Size "Its width; null fits (grows with fill)."
  field label String "What a screen reader calls the row of tabs (its accessible name), never drawn; empty is its id. A localised game passes its string table's."
}
```

<sub>`api.ogt` line 16667</sub>
