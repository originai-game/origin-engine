<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-TableColumn"></a>

### `TableColumn` (desc struct)

A column of a table: the row field it shows and how.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `key` | `string` | `""` | — | — | — | The field of each row it shows. |
| `label` | `string` | `""` | — | — | — | Its header; empty is the key. |
| `width` | `number \| null` | `null` | — | `cssPx` | — | Its width; null shares what is left. |
| `numeric` | `boolean` | `false` | — | — | — | Numbers: aligned to the end in the mono face, sorted by value, descending first. |
| `format` | `NumberFormat` | `"grouped"` | — | — | — | How its numbers read. |
| `digits` | `number` | `0` | — | — | — | Decimals. |
| `unit` | `string` | `""` | — | — | — | A unit after its numbers. |
| `sortable` | `boolean` | `true` | — | — | — | A press on its header sorts by it. |
| `filterable` | `boolean` | `true` | — | — | — | The search reads it. |
| `priority` | `number` | `0` | — | — | — | On a phone the columns that fit are kept, highest priority first. |
| `toneKey` | `string` | `""` | — | — | — | A field of each row naming its cell's tone (`up`, `down`, `success`, `danger`). |

```ogt
struct TableColumn {
  doc "A column of a table: the row field it shows and how."
  desc true
  field key String "The field of each row it shows."
  field label String "Its header; empty is the key."
  field width Option.f32 "Its width; null shares what is left." {
    unit cssPx
  }
  field numeric bool "Numbers: aligned to the end in the mono face, sorted by value, descending first."
  field format NumberFormat "How its numbers read."
  field digits u32 "Decimals."
  field unit String "A unit after its numbers."
  field sortable bool "A press on its header sorts by it." {
    default true
  }
  field filterable bool "The search reads it." {
    default true
  }
  field priority i32 "On a phone the columns that fit are kept, highest priority first."
  field toneKey String "A field of each row naming its cell's tone (`up`, `down`, `success`, `danger`)."
}
```

<sub>`api.ogt` line 18306</sub>
