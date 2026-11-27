<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`audio`](../audio.md) · [index](../index.md)

<a id="type-SfxSampleDesc"></a>

### `SfxSampleDesc` (desc struct)

An authored mono PCM source for a synth layer, instead of its oscillator. Linear interpolation; outside a non-looping recording the source is silent. Envelope, pitch slide, vibrato and filters still apply.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `samples` | `readonly number[]` | `[]` | — | — | — | Finite mono PCM samples in -1..1; at least one sample. |
| `rate` | `number` | `0` | — | — | — | Recording sample rate in Hz, 8000..192000. |
| `reference` | `number` | `440` | ≥ 1 | `hz` | — | Layer frequency that plays the recording at original speed. |
| `loop` | `boolean` | `false` | — | — | — | Wrap at the recording's end; envelope duration remains explicit. |

```ogt
struct SfxSampleDesc {
  doc "An authored mono PCM source for a synth layer, instead of its oscillator. Linear interpolation; outside a non-looping recording the source is silent. Envelope, pitch slide, vibrato and filters still apply."
  desc true
  field samples List.f32 "Finite mono PCM samples in -1..1; at least one sample."
  field rate u32 "Recording sample rate in Hz, 8000..192000."
  field reference f32 "Layer frequency that plays the recording at original speed." {
    default 440
    min 1
    unit hz
  }
  field loop bool "Wrap at the recording's end; envelope duration remains explicit."
}
```

<sub>Module `audio` line 740</sub>
