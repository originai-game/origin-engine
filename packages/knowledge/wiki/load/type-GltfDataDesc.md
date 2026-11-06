<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`load`](../load.md) · [index](../index.md)

<a id="type-GltfDataDesc"></a>

### `GltfDataDesc` (desc struct)

How og.load.gltfData reads a model.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `budget` | `LoadBudget` | `{}` | — | — | — | The limits it is held to. |

```ogt
struct GltfDataDesc {
  doc "How og.load.gltfData reads a model."
  desc true
  field budget LoadBudget "The limits it is held to."
}
```

<sub>`api.ogt` line 8349</sub>
