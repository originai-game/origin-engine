<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`shared`](../shared.md) · [index](../index.md)

<a id="record-SharedObjectChanged"></a>

### `og.SharedObjectChanged` (event)

A shared object came into sight or changed: its og.SharedObject entity holds the new state from this tick.

Token `og.SharedObjectChanged`, registered under the schema name `og.SharedObjectChanged`; its value type is `SharedObjectChanged`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `entity` | `Entity` | `0` | — | — | — | Its og.SharedObject entity. |
| `key` | `string` | `""` | — | — | — | Its key (at most 63 bytes). |
| `version` | `number` | `0` | — | — | — | Its version now. |

```ogt
event SharedObjectChanged {
  doc "A shared object came into sight or changed: its og.SharedObject entity holds the new state from this tick."
  field entity Entity "Its og.SharedObject entity."
  field key String "Its key (at most 63 bytes)."
  field version u32 "Its version now."
}
```

<sub>Module `shared` line 115</sub>
