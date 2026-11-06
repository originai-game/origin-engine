<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`load`](../load.md) · [index](../index.md)

<a id="record-AssetReady"></a>

### `og.AssetReady` (event)

A load completed.

Token `og.AssetReady`, registered under the schema name `og.AssetReady`; its value type is `AssetReady`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `asset` | `Asset` | `0` | — | — | — | The handle. |
| `path` | `string` | `""` | — | — | — | The path it was loaded from. |

```ogt
event AssetReady {
  doc "A load completed."
  field asset Asset "The handle."
  field path String "The path it was loaded from."
}
```

<sub>`api.ogt` line 8491</sub>
