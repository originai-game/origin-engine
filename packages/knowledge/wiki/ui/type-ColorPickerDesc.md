<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-ColorPickerDesc"></a>

### `ColorPickerDesc` (desc struct)

A colour picker: its swatches and the parts it shows.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `swatches` | `readonly string[]` | `[]` | — | — | — | Preset colours as `#rrggbb` (a livery's, a team's); empty for none. |
| `alpha` | `boolean` | `false` | — | — | — | An opacity strip. |
| `hex` | `boolean` | `true` | — | — | — | A hex field. |
| `label` | `string` | `""` | — | — | — | Its name; empty for none. |
| `width` | `Size \| null` | `null` | — | — | — | Its width; null is 280. |
| `disabled` | `boolean` | `false` | — | — | — | Shown but inert. |

```ogt
struct ColorPickerDesc {
  doc "A colour picker: its swatches and the parts it shows."
  desc true
  field swatches List.String "Preset colours as `#rrggbb` (a livery's, a team's); empty for none."
  field alpha bool "An opacity strip."
  field hex bool "A hex field." {
    default true
  }
  field label String "Its name; empty for none."
  field width Option.Size "Its width; null is 280."
  field disabled bool "Shown but inert."
}
```

<sub>`api.ogt` line 19804</sub>
