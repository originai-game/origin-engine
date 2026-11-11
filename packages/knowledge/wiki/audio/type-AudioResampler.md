<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`audio`](../audio.md) · [index](../index.md)

<a id="type-AudioResampler"></a>

### `AudioResampler` (enum)

How the mixer resamples a voice whose pitch or sound rate differs from the output's.

A string: one of the variants below. The zero value is `"linear"`.

| Variant | Doc |
|---|---|
| `"linear"` | Linear interpolation: the engine's mix before round 4, cheaper, with duller highs (-3.4 dB at 15 kHz from 44.1 kHz) and images (-11.5 dB there). |
| `"sinc"` | A Kaiser-windowed sinc (beta 7, 16 taps, 2048 phases; widened up to 64 taps for a voice pitched up to 4x): flat to the highs, images below -70 dB. |

```ogt
enum AudioResampler {
  doc "How the mixer resamples a voice whose pitch or sound rate differs from the output's."
  variant linear "Linear interpolation: the engine's mix before round 4, cheaper, with duller highs (-3.4 dB at 15 kHz from 44.1 kHz) and images (-11.5 dB there)."
  variant sinc "A Kaiser-windowed sinc (beta 7, 16 taps, 2048 phases; widened up to 64 taps for a voice pitched up to 4x): flat to the highs, images below -70 dB."
}
```

<sub>Module `audio` line 34</sub>
