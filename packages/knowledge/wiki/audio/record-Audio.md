<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`audio`](../audio.md) · [index](../index.md)

<a id="record-Audio"></a>

### `og.Audio` (resource)

Bus volumes; master scales every bus. A change ramps over 256 frames in the mixer.

Token `og.Audio`, registered under the schema name `og.Audio`; its value type is `Audio`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `master` | `number` | `1` | 0 … 2 | — | tunable | Gain of the master bus, applied to every sound. |
| `music` | `number` | `1` | 0 … 2 | — | tunable | Gain of the music bus. |
| `sfx` | `number` | `1` | 0 … 2 | — | tunable | Gain of the sfx bus. |
| `ui` | `number` | `1` | 0 … 2 | — | tunable | Gain of the ui bus. |
| `maxVoices` | `number` | `32` | 1 … 256 | — | — | The most sources that play at once; past it the lowest AudioSource.priority is stopped (SoundEnded reason SOUND_CULLED). |
| `occlusionGain` | `number` | `0.35` | 0 … 1 | — | tunable | The gain of an occluded source (AudioSource.occlude). |
| `occlusionCutoff` | `number` | `900` | 20 … 20000 | `hz` | tunable | The low-pass cutoff of an occluded source. |
| `reverbRoom` | `number` | `0.6` | 0 … 1 | — | tunable | The shared reverb's room size (the AudioBusFx.reverb sends feed it). |
| `reverbDamping` | `number` | `0.4` | 0 … 1 | — | tunable | The shared reverb's damping of high frequencies. |
| `outputLatency` | `number` | `0` | ≥ 0 | `s` | read only | Seconds from a sample's world time to the speaker, as the host reports it (og.audio.outputLatency). |
| `speedOfSound` | `number` | `343.3` | 1 … 10000 | `mps` | tunable | The speed of sound Doppler shifts are computed against (AudioSource.dopplerFactor). |
| `resampler` | `AudioResampler` | `"sinc"` | — | — | — | How voices are resampled: sinc (the default) or linear, the mix before round 4, for a game that relies on its exact output. A sound at the output's rate played at pitch 1 is its own samples either way. |

```ogt
resource Audio {
  doc "Bus volumes; master scales every bus. A change ramps over 256 frames in the mixer."
  field master f32 "Gain of the master bus, applied to every sound." {
    default 1
    min 0
    max 2
    tunable true
  }
  field music f32 "Gain of the music bus." {
    default 1
    min 0
    max 2
    tunable true
  }
  field sfx f32 "Gain of the sfx bus." {
    default 1
    min 0
    max 2
    tunable true
  }
  field ui f32 "Gain of the ui bus." {
    default 1
    min 0
    max 2
    tunable true
  }
  field maxVoices u32 "The most sources that play at once; past it the lowest AudioSource.priority is stopped (SoundEnded reason SOUND_CULLED)." {
    default 32
    min 1
    max 256
  }
  field occlusionGain f32 "The gain of an occluded source (AudioSource.occlude)." {
    default 0.35
    min 0
    max 1
    tunable true
  }
  field occlusionCutoff f32 "The low-pass cutoff of an occluded source." {
    default 900
    min 20
    max 20000
    unit hz
    tunable true
  }
  field reverbRoom f32 "The shared reverb's room size (the AudioBusFx.reverb sends feed it)." {
    default 0.6
    min 0
    max 1
    tunable true
  }
  field reverbDamping f32 "The shared reverb's damping of high frequencies." {
    default 0.4
    min 0
    max 1
    tunable true
  }
  field outputLatency f64 "Seconds from a sample's world time to the speaker, as the host reports it (og.audio.outputLatency)." {
    min 0
    unit s
    readonly true
  }
  field speedOfSound f32 "The speed of sound Doppler shifts are computed against (AudioSource.dopplerFactor)." {
    default 343.3
    min 1
    max 10000
    unit mps
    tunable true
  }
  field resampler AudioResampler "How voices are resampled: sinc (the default) or linear, the mix before round 4, for a game that relies on its exact output. A sound at the output's rate played at pitch 1 is its own samples either way." {
    default sinc
  }
}
```

<sub>Module `audio` line 194</sub>
