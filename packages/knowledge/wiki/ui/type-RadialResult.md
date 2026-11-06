<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-RadialResult"></a>

### `RadialResult` (struct)

What a radial wheel answered this tick.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `chosen` | `number \| null` | `null` | — | — | — | The slice picked (a release, a press). |
| `hovered` | `number \| null` | `null` | — | — | — | The slice pointed at. |
| `dismissed` | `boolean` | `false` | — | — | — | Back or a press outside asked to close it. |

```ogt
struct RadialResult {
  doc "What a radial wheel answered this tick."
  field chosen Option.u32 "The slice picked (a release, a press)."
  field hovered Option.u32 "The slice pointed at."
  field dismissed bool "Back or a press outside asked to close it."
}
```

<sub>Module `ui` line 2572</sub>
