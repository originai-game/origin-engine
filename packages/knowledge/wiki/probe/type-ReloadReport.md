<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`probe`](../probe.md) · [index](../index.md)

<a id="type-ReloadReport"></a>

### `ReloadReport` (struct)

What a hot reload did. Components are re-registered by name and migrated field by field through the schema.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `tick` | `number` | `0` | — | — | — | The tick the world was restored at. |
| `ms` | `number` | `0` | — | `ms` | — | From bundle swap to restored world. |
| `migrated` | `readonly string[]` | `[]` | — | — | — | Fields carried across a layout change, as `Health.hp`. |
| `dropped` | `readonly string[]` | `[]` | — | — | — | Components or fields that no longer exist, dropped with their data. |
| `warnings` | `readonly string[]` | `[]` | — | — | — | Anything else worth reading, each naming the fix. |

```ogt
struct ReloadReport {
  doc "What a hot reload did. Components are re-registered by name and migrated field by field through the schema."
  field tick u32 "The tick the world was restored at."
  field ms f32 "From bundle swap to restored world." {
    unit ms
  }
  field migrated List.String "Fields carried across a layout change, as `Health.hp`."
  field dropped List.String "Components or fields that no longer exist, dropped with their data."
  field warnings List.String "Anything else worth reading, each naming the fix."
}
```

<sub>`api.ogt` line 24288</sub>
