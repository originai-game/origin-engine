<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-NodeGraphResult"></a>

### `NodeGraphResult` (struct)

What a graph answered this tick.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `chosen` | `string \| null` | `null` | — | — | — | The node pressed. |
| `zoom` | `number` | `0` | — | — | — | The zoom after this tick: keep it. |
| `changed` | `boolean` | `false` | — | — | — | The zoom changed this tick. |

```ogt
struct NodeGraphResult {
  doc "What a graph answered this tick."
  field chosen Option.String "The node pressed."
  field zoom f32 "The zoom after this tick: keep it."
  field changed bool "The zoom changed this tick."
}
```

<sub>Module `ui` line 2850</sub>
