<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`audio`](../audio.md) · [index](../index.md)

<a id="type-SfxLayerDesc"></a>

### `SfxLayerDesc` (desc struct)

One oscillator or PCM sample layer of a procedural sound effect (og.audio.synth, og.audio.createSynthSound).

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `sample` | `SfxSampleDesc \| null` | `null` | — | — | — | Optional recording instead of wave. FM is oscillator-only; sample layers reject it. |
| `wave` | `SfxWave` | `"square"` | — | — | — | Oscillator waveform. |
| `freq` | `number` | `440` | 0 … 48000 | `hz` | — | Start frequency (for noise, >= 24000 or 0 runs full-rate white noise; lower values sample-and-hold at freq). |
| `freqEnd` | `number` | `0` | 0 … 48000 | `hz` | — | Optional target end frequency overriding slide when > 0 (0: use slide). |
| `slide` | `number` | `0` | -24 … 24 | — | — | Pitch slide in octaves per second. |
| `arpAt` | `number` | `0` | 0 … 30 | `s` | — | Seconds in when the frequency multiplies by arp (0: never). |
| `arp` | `number` | `0` | 0 … 16 | — | — | Frequency multiplier applied at arpAt. |
| `vibDepth` | `number` | `0` | 0 … 4 | — | — | Vibrato depth in octaves. |
| `vibRate` | `number` | `0` | 0 … 100 | `hz` | — | Vibrato modulation rate. |
| `attack` | `number` | `0.002` | 0 … 30 | `s` | — | Attack ramp duration. |
| `sustain` | `number` | `0.05` | 0 … 30 | `s` | — | Sustain duration. |
| `decay` | `number` | `0.15` | 0 … 30 | `s` | — | Decay duration. |
| `sustainLevel` | `number` | `1` | 0 … 1 | `ratio` | — | Envelope sustain level (1 keeps attack-sustain-decay; below 1 enables ADSR decay to sustainLevel). |
| `release` | `number` | `0` | 0 … 30 | `s` | — | Release tail duration after sustain. |
| `punch` | `number` | `0` | 0 … 2 | — | — | Smoothed initial punch boost over sustain. |
| `duty` | `number` | `0.5` | 0.02 … 0.98 | `ratio` | — | Square pulse duty cycle. |
| `lowpass` | `number` | `0` | 0 … 20000 | `hz` | — | Low-pass filter cutoff (0: off). |
| `lpSlide` | `number` | `0` | -24 … 24 | — | — | Low-pass cutoff slide in octaves per second. |
| `q` | `number` | `0` | 0 … 20 | — | — | Low-pass resonance Q (> 0 uses a Cytomic 2-pole SVF; 0 uses one-pole). |
| `highpass` | `number` | `0` | 0 … 20000 | `hz` | — | High-pass filter cutoff (0: off). |
| `fmFreq` | `number` | `0` | 0 … 20000 | `hz` | — | FM modulator frequency (0: off). |
| `fmIndex` | `number` | `0` | 0 … 32 | — | — | FM modulation index in radians (0: off). |
| `drive` | `number` | `0` | 0 … 32 | — | — | Soft-clip saturation drive (> 0 enables tanh waveshaping). |
| `gain` | `number` | `0.5` | 0 … 4 | — | — | Layer output gain. |
| `delay` | `number` | `0` | 0 … 30 | `s` | — | Start delay from time 0. |

```ogt
struct SfxLayerDesc {
  doc "One oscillator or PCM sample layer of a procedural sound effect (og.audio.synth, og.audio.createSynthSound)."
  desc true
  field sample Option.SfxSampleDesc "Optional recording instead of wave. FM is oscillator-only; sample layers reject it."
  field wave SfxWave "Oscillator waveform." {
    default square
  }
  field freq f32 "Start frequency (for noise, >= 24000 or 0 runs full-rate white noise; lower values sample-and-hold at freq)." {
    default 440
    min 0
    max 48000
    unit hz
  }
  field freqEnd f32 "Optional target end frequency overriding slide when > 0 (0: use slide)." {
    min 0
    max 48000
    unit hz
  }
  field slide f32 "Pitch slide in octaves per second." {
    min -24
    max 24
  }
  field arpAt f32 "Seconds in when the frequency multiplies by arp (0: never)." {
    min 0
    max 30
    unit s
  }
  field arp f32 "Frequency multiplier applied at arpAt." {
    min 0
    max 16
  }
  field vibDepth f32 "Vibrato depth in octaves." {
    min 0
    max 4
  }
  field vibRate f32 "Vibrato modulation rate." {
    min 0
    max 100
    unit hz
  }
  field attack f32 "Attack ramp duration." {
    default 0.002
    min 0
    max 30
    unit s
  }
  field sustain f32 "Sustain duration." {
    default 0.05
    min 0
    max 30
    unit s
  }
  field decay f32 "Decay duration." {
    default 0.15
    min 0
    max 30
    unit s
  }
  field sustainLevel f32 "Envelope sustain level (1 keeps attack-sustain-decay; below 1 enables ADSR decay to sustainLevel)." {
    default 1
    min 0
    max 1
    unit ratio
  }
  field release f32 "Release tail duration after sustain." {
    min 0
    max 30
    unit s
  }
  field punch f32 "Smoothed initial punch boost over sustain." {
    min 0
    max 2
  }
  field duty f32 "Square pulse duty cycle." {
    default 0.5
    min 0.02
    max 0.98
    unit ratio
  }
  field lowpass f32 "Low-pass filter cutoff (0: off)." {
    min 0
    max 20000
    unit hz
  }
  field lpSlide f32 "Low-pass cutoff slide in octaves per second." {
    min -24
    max 24
  }
  field q f32 "Low-pass resonance Q (> 0 uses a Cytomic 2-pole SVF; 0 uses one-pole)." {
    min 0
    max 20
  }
  field highpass f32 "High-pass filter cutoff (0: off)." {
    min 0
    max 20000
    unit hz
  }
  field fmFreq f32 "FM modulator frequency (0: off)." {
    min 0
    max 20000
    unit hz
  }
  field fmIndex f32 "FM modulation index in radians (0: off)." {
    min 0
    max 32
  }
  field drive f32 "Soft-clip saturation drive (> 0 enables tanh waveshaping)." {
    min 0
    max 32
  }
  field gain f32 "Layer output gain." {
    default 0.5
    min 0
    max 4
  }
  field delay f32 "Start delay from time 0." {
    min 0
    max 30
    unit s
  }
}
```

<sub>Module `audio` line 753</sub>
