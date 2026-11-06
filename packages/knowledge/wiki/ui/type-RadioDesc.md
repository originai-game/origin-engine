<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-RadioDesc"></a>

### `RadioDesc` (desc struct)

How a radio group lays out, and its name.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `horizontal` | `boolean` | `false` | — | — | — | Options in a row (the default is a column). |
| `label` | `string` | `""` | — | — | — | What a screen reader calls the group (its accessible name), never drawn; empty is its id. A localised game passes its string table's. |

```ogt
struct RadioDesc {
  doc "How a radio group lays out, and its name."
  desc true
  field horizontal bool "Options in a row (the default is a column)."
  field label String "What a screen reader calls the group (its accessible name), never drawn; empty is its id. A localised game passes its string table's."
}
```

<sub>`api.ogt` line 16731</sub>
