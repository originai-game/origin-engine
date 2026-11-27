<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`audio`](../audio.md) · [index](../index.md)

<a id="type-SfxSamples"></a>

### `SfxSamples` (struct)

Synthesised PCM audio samples (og.audio.synth), returned as a native Float32Array over binary wire.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `samples` | `Float32Array` | — | — | — | — | Mono floating-point PCM samples in [-1, 1]. |
| `rate` | `number` | `0` | — | `hz` | — | Sample rate. |
| `channels` | `number` | `0` | — | — | — | Channel count (1 mono). |

```ogt
struct SfxSamples {
  doc "Synthesised PCM audio samples (og.audio.synth), returned as a native Float32Array over binary wire."
  field samples Float32Array "Mono floating-point PCM samples in [-1, 1]."
  field rate u32 "Sample rate." {
    unit hz
  }
  field channels u32 "Channel count (1 mono)."
}
```

<sub>Module `audio` line 905</sub>
