<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`load`](../load.md) · [index](../index.md)

<a id="type-DataDesc"></a>

### `DataDesc` (desc struct)

Which catalog file og.load.data reads, and how.

See `loading §14`.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `entry` | `CatalogEntry` | `{}` | — | — | — | The file. |
| `format` | `DataFormat` | `"json"` | — | — | — | How to read it. |
| `budget` | `LoadBudget` | `{}` | — | — | — | The limits it is held to: bytes alone. |

```ogt
struct DataDesc {
  doc "Which catalog file og.load.data reads, and how."
  guide "loading §14"
  desc true
  field entry CatalogEntry "The file."
  field format DataFormat "How to read it."
  field budget LoadBudget "The limits it is held to: bytes alone."
}
```

<sub>Module `load` line 629</sub>
