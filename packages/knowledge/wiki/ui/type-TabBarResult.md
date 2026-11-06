<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-TabBarResult"></a>

### `TabBarResult` (struct)

What a tab bar answered this tick.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `selected` | `number` | `0` | — | — | — | The selected tab after this tick: keep it. |
| `changed` | `boolean` | `false` | — | — | — | It changed this tick. |

```ogt
struct TabBarResult {
  doc "What a tab bar answered this tick."
  field selected u32 "The selected tab after this tick: keep it."
  field changed bool "It changed this tick."
}
```

<sub>`api.ogt` line 18051</sub>
