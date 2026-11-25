<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`probe`](../probe.md) · [index](../index.md)

<a id="type-DrawnEntity"></a>

### `DrawnEntity` (struct)

One entity of dump_drawn.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `id` | `string` | `""` | — | — | — | Its probe id. |
| `name` | `string \| null` | `null` | — | — | — | Its name. |
| `visible` | `boolean \| null` | `null` | — | — | — | As Visibility.visible. |
| `reason` | `DrawReason` | `"drawn"` | — | — | — | As Visibility.reason. |
| `detail` | `string` | `""` | — | — | optional | As Visibility.detail. |
| `drawn` | `number` | `0` | — | — | — | As Visibility.drawn. |
| `culled` | `number` | `0` | — | — | — | As Visibility.culled. |

```ogt
struct DrawnEntity {
  doc "One entity of dump_drawn."
  field id String "Its probe id."
  field name Option.String "Its name."
  field visible Option.bool "As Visibility.visible."
  field reason DrawReason "As Visibility.reason."
  field detail String "As Visibility.detail." {
    optional true
  }
  field drawn u32 "As Visibility.drawn."
  field culled u32 "As Visibility.culled."
}
```

<sub>Module `probe` line 504</sub>
