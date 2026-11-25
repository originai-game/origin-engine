<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`shared`](../shared.md) · [index](../index.md)

<a id="type-SharedRemote"></a>

### `SharedRemote` (struct)

Who an og.RemoteAvatar is.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `player` | `string` | `""` | — | — | — | Its opaque player id. |
| `name` | `string` | `""` | — | — | — | Its display name. |
| `kind` | `number` | `0` | — | — | — | Its kind. |
| `sinceTick` | `number` | `0` | — | `ticks` | — | The tick it came into sight. |

```ogt
struct SharedRemote {
  doc "Who an og.RemoteAvatar is."
  field player String "Its opaque player id."
  field name String "Its display name."
  field kind u32 "Its kind."
  field sinceTick f64 "The tick it came into sight." {
    unit ticks
  }
}
```

<sub>Module `shared` line 174</sub>
