<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-KeyValueRowsDesc"></a>

### `KeyValueRowsDesc` (desc struct)

Key-value rows: a stat sheet, an item's properties.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `rows` | `readonly KeyValueRow[]` | `[]` | — | — | — | The rows. |
| `title` | `string` | `""` | — | — | — | A caption over them; empty is none. |
| `dense` | `boolean` | `false` | — | — | — | Smaller type and less space. |
| `dividers` | `boolean` | `true` | — | — | — | Hairlines between rows. |
| `striped` | `boolean` | `false` | — | — | — | Every other row shaded instead of hairlines. |
| `inset` | `number` | `0` | ≥ 0 | `cssPx` | — | Space at each row's ends. |
| `width` | `Size \| null` | `null` | — | — | — | Its width; null grows. |
| `label` | `string` | `""` | — | — | — | What a screen reader calls the list when it has no title. |

```ogt
struct KeyValueRowsDesc {
  doc "Key-value rows: a stat sheet, an item's properties."
  desc true
  field rows List.KeyValueRow "The rows."
  field title String "A caption over them; empty is none."
  field dense bool "Smaller type and less space."
  field dividers bool "Hairlines between rows." {
    default true
  }
  field striped bool "Every other row shaded instead of hairlines."
  field inset f32 "Space at each row's ends." {
    unit cssPx
    min 0
  }
  field width Option.Size "Its width; null grows."
  field label String "What a screen reader calls the list when it has no title."
}
```

<sub>`api.ogt` line 18589</sub>
