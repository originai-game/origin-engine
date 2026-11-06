<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-KeyBindingResult"></a>

### `KeyBindingResult` (struct)

What a key-binding row did this tick.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `capturing` | `boolean` | `false` | — | — | — | One of its slots waits for input. |
| `changed` | `boolean` | `false` | — | — | — | Its bindings changed this tick (save with og.input.saveBindings). |
| `conflict` | `boolean` | `false` | — | — | — | A captured control is used by another action: the row asks the player. |
| `slot` | `number` | `0` | — | — | — | The slot capturing or in conflict (-1: none). |

```ogt
struct KeyBindingResult {
  doc "What a key-binding row did this tick."
  field capturing bool "One of its slots waits for input."
  field changed bool "Its bindings changed this tick (save with og.input.saveBindings)."
  field conflict bool "A captured control is used by another action: the row asks the player."
  field slot i32 "The slot capturing or in conflict (-1: none)."
}
```

<sub>`api.ogt` line 19862</sub>
