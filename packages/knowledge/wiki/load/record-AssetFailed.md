<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`load`](../load.md) · [index](../index.md)

<a id="record-AssetFailed"></a>

### `og.AssetFailed` (event)

A load failed.

Token `og.AssetFailed`, registered under the schema name `og.AssetFailed`; its value type is `AssetFailed`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `asset` | `Asset` | `0` | — | — | — | The handle. |
| `path` | `string` | `""` | — | — | — | The path it was loaded from. |
| `reason` | `string` | `""` | — | — | — | Why, naming the fix. |

```ogt
event AssetFailed {
  doc "A load failed."
  field asset Asset "The handle."
  field path String "The path it was loaded from."
  field reason String "Why, naming the fix."
}
```

<sub>Module `load` line 593</sub>
