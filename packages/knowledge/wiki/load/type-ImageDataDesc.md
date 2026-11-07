<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`load`](../load.md) · [index](../index.md)

<a id="type-ImageDataDesc"></a>

### `ImageDataDesc` (desc struct)

How og.load.imageData reads an image.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `kind` | `ImageKind` | `"color"` | — | — | — | How its texels are read. |
| `budget` | `LoadBudget` | `{}` | — | — | — | The limits it is held to. |

```ogt
struct ImageDataDesc {
  doc "How og.load.imageData reads an image."
  desc true
  field kind ImageKind "How its texels are read."
  field budget LoadBudget "The limits it is held to."
}
```

<sub>Module `load` line 479</sub>
