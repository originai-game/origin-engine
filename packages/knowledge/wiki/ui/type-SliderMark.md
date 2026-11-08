<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-SliderMark"></a>

### `SliderMark` (struct)

A labelled point on a slider's track.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `value` | `number` | `0` | — | — | — | Where it is. |
| `label` | `string` | `""` | — | — | — | What it says under the track; empty for a tick only. |

```ogt
struct SliderMark {
  doc "A labelled point on a slider's track."
  field value f32 "Where it is."
  field label String "What it says under the track; empty for a tick only."
}
```

<sub>Module `ui` line 3493</sub>
