<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-DropSpec"></a>

### `DropSpec` (desc struct)

The element takes drops of dragged elements.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `accept` | `readonly string[]` | `[]` | — | — | — | The drag tags it takes (empty: any). |

```ogt
struct DropSpec {
  doc "The element takes drops of dragged elements."
  desc true
  field accept List.String "The drag tags it takes (empty: any)."
}
```

<sub>Module `ui` line 1806</sub>
