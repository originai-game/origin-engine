<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`audio`](../audio.md) · [index](../index.md)

<a id="record-AudioBusFx"></a>

### `og.AudioBusFx` (component)

A bus's insert effects: a high- and low-pass (12 dB/octave), a compressor, side-chain ducking by another bus, and a send to the shared reverb, whose return joins master. One entity per bus (the first in query order counts); without one a bus passes straight through.

Changes reach the mixer through the command stream, so they are deterministic and in the world hash; the PCM is the mixer's.

Also known as: ducking, side-chain, compressor, low-pass, mixer effects, Unity AudioMixer, Godot AudioEffect

Token `og.AudioBusFx`, registered under the schema name `og.AudioBusFx`; its value type is `AudioBusFx`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `bus` | `AudioBus` | `"music"` | — | — | — | The bus these effects are on. |
| `lowpass` | `number` | `0` | 0 … 20000 | `hz` | tunable | Low-pass cutoff; 0 is off. |
| `highpass` | `number` | `0` | 0 … 20000 | `hz` | tunable | High-pass cutoff; 0 is off. |
| `reverb` | `number` | `0` | 0 … 1 | — | tunable | How much of the bus goes to the reverb. |
| `threshold` | `number` | `0` | -60 … 0 | `db` | tunable | The compressor's threshold; 0 dB is off. |
| `ratio` | `number` | `4` | 1 … 30 | — | tunable | The compressor's ratio: dB in over the threshold per dB out. |
| `attack` | `number` | `0.005` | 0 … 1 | `s` | tunable | The compressor's attack. |
| `release` | `number` | `0.15` | 0 … 3 | `s` | tunable | The compressor's release. |
| `makeup` | `number` | `0` | -24 … 24 | `db` | tunable | Gain after the compressor. |
| `duck` | `boolean` | `false` | — | — | — | Turns this bus down while duckBus is loud: music under effects or dialogue. |
| `duckBus` | `AudioBus` | `"sfx"` | — | — | — | The bus that ducks this one. |
| `duckDepth` | `number` | `9` | 0 … 60 | `db` | tunable | How far it is turned down. |
| `duckThreshold` | `number` | `-36` | -80 … 0 | `db` | tunable | How loud the key bus must be. |
| `duckAttack` | `number` | `0.02` | 0 … 2 | `s` | tunable | How fast it goes down. |
| `duckRelease` | `number` | `0.4` | 0 … 5 | `s` | tunable | How fast it comes back. |
| `userBus` | `number` | `0` | 0 … 15 | — | — | A user bus (4 to 15, an og.AudioBusNode) these effects are on instead of `bus`; 0 uses `bus`. |
| `eqLow` | `number` | `0` | -24 … 24 | `db` | tunable | The equaliser's low shelf (RBJ, 12 dB/octave): its gain below eqLowFreq; 0 dB is flat. |
| `eqLowFreq` | `number` | `200` | 20 … 2000 | `hz` | tunable | The low shelf's corner frequency. |
| `eqMid` | `number` | `0` | -24 … 24 | `db` | tunable | The equaliser's mid band (a peak, or a notch below 0 dB): its gain at eqMidFreq; 0 dB is flat. |
| `eqMidFreq` | `number` | `1000` | 40 … 16000 | `hz` | tunable | The mid band's centre frequency. |
| `eqMidQ` | `number` | `0.7` | 0.1 … 10 | — | tunable | The mid band's Q: higher is narrower. |
| `eqHigh` | `number` | `0` | -24 … 24 | `db` | tunable | The equaliser's high shelf: its gain above eqHighFreq; 0 dB is flat. |
| `eqHighFreq` | `number` | `5000` | 1000 … 20000 | `hz` | tunable | The high shelf's corner frequency. |
| `delay` | `number` | `0` | 0 … 2 | `s` | tunable | An echo after the compressor: the delay's time, to the sample; 0 is off. |
| `delayFeedback` | `number` | `0.35` | 0 … 0.95 | — | tunable | How much of each echo feeds the next. |
| `delayMix` | `number` | `0.35` | 0 … 1 | — | tunable | How loud the echoes are, added to the dry bus. |
| `distortion` | `number` | `0` | 0 … 1 | — | tunable | A tanh waveshaper after the equaliser: its drive, 0 (off) to 1 (heavy: 20x into the curve), scaled so a full-scale input stays at full scale. |
| `distortionMix` | `number` | `1` | 0 … 1 | — | tunable | How much of the distorted signal replaces the clean one. |

```ogt
component AudioBusFx {
  doc "A bus's insert effects: a high- and low-pass (12 dB/octave), a compressor, side-chain ducking by another bus, and a send to the shared reverb, whose return joins master. One entity per bus (the first in query order counts); without one a bus passes straight through."
  doc "Changes reach the mixer through the command stream, so they are deterministic and in the world hash; the PCM is the mixer's."
  aliases "ducking, side-chain, compressor, low-pass, mixer effects, Unity AudioMixer, Godot AudioEffect"
  field bus AudioBus "The bus these effects are on." {
    default music
  }
  field lowpass f32 "Low-pass cutoff; 0 is off." {
    min 0
    max 20000
    unit hz
    tunable true
  }
  field highpass f32 "High-pass cutoff; 0 is off." {
    min 0
    max 20000
    unit hz
    tunable true
  }
  field reverb f32 "How much of the bus goes to the reverb." {
    min 0
    max 1
    tunable true
  }
  field threshold f32 "The compressor's threshold; 0 dB is off." {
    min -60
    max 0
    unit db
    tunable true
  }
  field ratio f32 "The compressor's ratio: dB in over the threshold per dB out." {
    default 4
    min 1
    max 30
    tunable true
  }
  field attack f32 "The compressor's attack." {
    default 0.005
    min 0
    max 1
    unit s
    tunable true
  }
  field release f32 "The compressor's release." {
    default 0.15
    min 0
    max 3
    unit s
    tunable true
  }
  field makeup f32 "Gain after the compressor." {
    min -24
    max 24
    unit db
    tunable true
  }
  field duck bool "Turns this bus down while duckBus is loud: music under effects or dialogue."
  field duckBus AudioBus "The bus that ducks this one." {
    default sfx
  }
  field duckDepth f32 "How far it is turned down." {
    default 9
    min 0
    max 60
    unit db
    tunable true
  }
  field duckThreshold f32 "How loud the key bus must be." {
    default -36
    min -80
    max 0
    unit db
    tunable true
  }
  field duckAttack f32 "How fast it goes down." {
    default 0.02
    min 0
    max 2
    unit s
    tunable true
  }
  field duckRelease f32 "How fast it comes back." {
    default 0.4
    min 0
    max 5
    unit s
    tunable true
  }
  field userBus u32 "A user bus (4 to 15, an og.AudioBusNode) these effects are on instead of `bus`; 0 uses `bus`." {
    min 0
    max 15
  }
  field eqLow f32 "The equaliser's low shelf (RBJ, 12 dB/octave): its gain below eqLowFreq; 0 dB is flat." {
    min -24
    max 24
    unit db
    tunable true
  }
  field eqLowFreq f32 "The low shelf's corner frequency." {
    default 200
    min 20
    max 2000
    unit hz
    tunable true
  }
  field eqMid f32 "The equaliser's mid band (a peak, or a notch below 0 dB): its gain at eqMidFreq; 0 dB is flat." {
    min -24
    max 24
    unit db
    tunable true
  }
  field eqMidFreq f32 "The mid band's centre frequency." {
    default 1000
    min 40
    max 16000
    unit hz
    tunable true
  }
  field eqMidQ f32 "The mid band's Q: higher is narrower." {
    default 0.7
    min 0.1
    max 10
    tunable true
  }
  field eqHigh f32 "The equaliser's high shelf: its gain above eqHighFreq; 0 dB is flat." {
    min -24
    max 24
    unit db
    tunable true
  }
  field eqHighFreq f32 "The high shelf's corner frequency." {
    default 5000
    min 1000
    max 20000
    unit hz
    tunable true
  }
  field delay f32 "An echo after the compressor: the delay's time, to the sample; 0 is off." {
    min 0
    max 2
    unit s
    tunable true
  }
  field delayFeedback f32 "How much of each echo feeds the next." {
    default 0.35
    min 0
    max 0.95
    tunable true
  }
  field delayMix f32 "How loud the echoes are, added to the dry bus." {
    default 0.35
    min 0
    max 1
    tunable true
  }
  field distortion f32 "A tanh waveshaper after the equaliser: its drive, 0 (off) to 1 (heavy: 20x into the curve), scaled so a full-scale input stays at full scale." {
    min 0
    max 1
    tunable true
  }
  field distortionMix f32 "How much of the distorted signal replaces the clean one." {
    default 1
    min 0
    max 1
    tunable true
  }
}
```

<sub>Module `audio` line 267</sub>
