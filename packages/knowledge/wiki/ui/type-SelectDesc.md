<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-SelectDesc"></a>

### `SelectDesc` (desc struct)

A select: a trigger that opens a menu of options.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `placeholder` | `string` | `""` | — | — | — | Shown while nothing is chosen. |
| `width` | `Size \| null` | `null` | — | — | — | Its width; null grows from 160. |
| `disabled` | `boolean` | `false` | — | — | — | Neutral and inert. |
| `label` | `string` | `""` | — | — | — | What a screen reader calls it (its accessible name; the chosen option is its value), never drawn; empty is its id. A localised game passes its string table's. |

```ogt
struct SelectDesc {
  doc "A select: a trigger that opens a menu of options."
  desc true
  field placeholder String "Shown while nothing is chosen."
  field width Option.Size "Its width; null grows from 160."
  field disabled bool "Neutral and inert."
  field label String "What a screen reader calls it (its accessible name; the chosen option is its value), never drawn; empty is its id. A localised game passes its string table's."
}
```

<sub>`api.ogt` line 16703</sub>
