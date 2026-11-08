<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-DialogueResult"></a>

### `DialogueResult` (struct)

A dialogue box's answer this tick.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `shown` | `number` | `0` | — | — | — | Characters shown. |
| `done` | `boolean` | `false` | — | — | — | The whole line is shown. |
| `advanced` | `boolean` | `false` | — | — | — | Pressed (or activated) while done: show the next line. |

```ogt
struct DialogueResult {
  doc "A dialogue box's answer this tick."
  field shown u32 "Characters shown."
  field done bool "The whole line is shown."
  field advanced bool "Pressed (or activated) while done: show the next line."
}
```

<sub>Module `ui` line 1075</sub>
