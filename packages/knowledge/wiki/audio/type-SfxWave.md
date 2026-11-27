<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`audio`](../audio.md) · [index](../index.md)

<a id="type-SfxWave"></a>

### `SfxWave` (enum)

Oscillator waveform for procedural SFX synthesis.

A string: one of the variants below. The zero value is `"square"`.

| Variant | Doc |
|---|---|
| `"square"` | PolyBLEP band-limited pulse/square wave (controlled by duty). |
| `"saw"` | PolyBLEP band-limited sawtooth wave. |
| `"sine"` | Pure sine wave. |
| `"triangle"` | Triangle wave. |
| `"noise"` | Deterministic white or sample-and-hold noise (SplitMix64). |

```ogt
enum SfxWave {
  doc "Oscillator waveform for procedural SFX synthesis."
  variant square "PolyBLEP band-limited pulse/square wave (controlled by duty)."
  variant saw "PolyBLEP band-limited sawtooth wave."
  variant sine "Pure sine wave."
  variant triangle "Triangle wave."
  variant noise "Deterministic white or sample-and-hold noise (SplitMix64)."
}
```

<sub>Module `audio` line 731</sub>
