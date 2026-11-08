<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-SettingChange"></a>

### `SettingChange` (struct)

One entry changed by the player this tick.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `key` | `string` | `""` | — | — | — | The entry. |
| `on` | `boolean` | `false` | — | — | — | A toggle's new state. |
| `value` | `number` | `0` | — | — | — | A slider's new value. |
| `index` | `number` | `0` | — | — | — | A choice's new word. |

```ogt
struct SettingChange {
  doc "One entry changed by the player this tick."
  field key String "The entry."
  field on bool "A toggle's new state."
  field value f32 "A slider's new value."
  field index u32 "A choice's new word."
}
```

<sub>Module `ui` line 3551</sub>
