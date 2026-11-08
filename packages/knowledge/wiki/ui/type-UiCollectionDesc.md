<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-UiCollectionDesc"></a>

### `UiCollectionDesc` (struct)

A keyed collection's identity rule. The key is data, never the row index.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `key` | `string` | `""` | — | — | optional | The row field containing a nonempty string key; empty means id. |

```ogt
struct UiCollectionDesc {
  doc "A keyed collection's identity rule. The key is data, never the row index."
  field key String "The row field containing a nonempty string key; empty means id." {
    optional true
  }
}
```

<sub>Module `ui` line 7824</sub>
