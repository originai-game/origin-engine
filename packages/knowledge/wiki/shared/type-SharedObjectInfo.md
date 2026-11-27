<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`shared`](../shared.md) · [index](../index.md)

<a id="type-SharedObjectInfo"></a>

### `SharedObjectInfo` (struct)

Which shared object an og.SharedObject entity is.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `key` | `string` | `""` | — | — | — | Its key. |
| `version` | `number` | `0` | — | — | — | Its version. |

```ogt
struct SharedObjectInfo {
  doc "Which shared object an og.SharedObject entity is."
  field key String "Its key."
  field version u32 "Its version."
}
```

<sub>Module `shared` line 145</sub>
