<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`shared`](../shared.md) · [index](../index.md)

<a id="record-SharedObjectGone"></a>

### `og.SharedObjectGone` (event)

A shared object left sight or was removed: its og.SharedObject entity was despawned this tick.

Token `og.SharedObjectGone`, registered under the schema name `og.SharedObjectGone`; its value type is `SharedObjectGone`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `entity` | `Entity` | `0` | — | — | — | The entity it had. |
| `key` | `string` | `""` | — | — | — | Its key (at most 63 bytes). |

```ogt
event SharedObjectGone {
  doc "A shared object left sight or was removed: its og.SharedObject entity was despawned this tick."
  field entity Entity "The entity it had."
  field key String "Its key (at most 63 bytes)."
}
```

<sub>Module `shared` line 129</sub>
