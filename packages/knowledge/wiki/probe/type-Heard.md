<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`probe`](../probe.md) · [index](../index.md)

<a id="type-Heard"></a>

### `Heard` (struct)

What the host's mixer rendered last: the audio up to the current tick.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `tick` | `number` | `0` | — | — | — | The tick it ends at. |
| `rate` | `number` | `0` | — | `hz` | — | Frames per second. |
| `frames` | `number` | `0` | — | — | — | Stereo frames in the WAV. |
| `peak` | `number` | `0` | — | — | — | The largest sample, 0 to 1. |
| `rms` | `number` | `0` | — | — | — | The root mean square of both channels. |
| `wav` | `string` | `""` | — | — | — | A 16-bit stereo WAV, base64. |

```ogt
struct Heard {
  doc "What the host's mixer rendered last: the audio up to the current tick."
  field tick u32 "The tick it ends at."
  field rate u32 "Frames per second." {
    unit hz
  }
  field frames u32 "Stereo frames in the WAV."
  field peak f32 "The largest sample, 0 to 1."
  field rms f32 "The root mean square of both channels."
  field wav String "A 16-bit stereo WAV, base64."
}
```

<sub>`api.ogt` line 24227</sub>
