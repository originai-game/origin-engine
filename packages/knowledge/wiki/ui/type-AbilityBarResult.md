<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-AbilityBarResult"></a>

### `AbilityBarResult` (struct)

What an ability bar answered.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `clicked` | `string \| null` | `null` | — | — | — | The slot pressed and released this tick (its key). |
| `pressed` | `readonly string[]` | `[]` | — | — | — | The slots held. |

```ogt
struct AbilityBarResult {
  doc "What an ability bar answered."
  field clicked Option.String "The slot pressed and released this tick (its key)."
  field pressed List.String "The slots held."
}
```

<sub>Module `ui` line 6004</sub>
