<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`input`](../input.md) · [index](../index.md)

<a id="type-RecordedIntent"></a>

### `RecordedIntent` (struct)

One deterministic input, independent of presentation checkpoints. Replay it between ticks, when world.time.tick equals tick.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `tick` | `number` | `0` | — | — | — | The simulation boundary at which the input was sent. |
| `name` | `string` | `""` | — | — | — | The reflected event name. |
| `layoutHash` | `string` | `""` | — | — | — | The schema layout hash as an exact decimal u64. |
| `payload` | `readonly number[]` | `[]` | — | — | — | The reflected packed field bytes, each 0 to 255. |

```ogt
struct RecordedIntent {
  doc "One deterministic input, independent of presentation checkpoints. Replay it between ticks, when world.time.tick equals tick."
  field tick f64 "The simulation boundary at which the input was sent."
  field name String "The reflected event name."
  field layoutHash String "The schema layout hash as an exact decimal u64."
  field payload List.u32 "The reflected packed field bytes, each 0 to 255."
}
```

<sub>Module `input` line 20</sub>
