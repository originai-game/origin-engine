<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`audio`](../audio.md) · [index](../index.md)

<a id="type-PanningModel"></a>

### `PanningModel` (enum)

How an AudioListener places spatial sources between the ears, as Web Audio's PannerNode.panningModel.

A string: one of the variants below. The zero value is `"equalpower"`.

| Variant | Doc |
|---|---|
| `"equalpower"` | The stereo panner (Web Audio's equal-power law): for speakers. |
| `"hrtf"` | For headphones: a spherical-head model (Brown and Duda, 1998) that delays the far ear by the interaural time difference (up to 0.66 ms) and shadows its highs (up to -20 dB), so a source moves between the ears; front and back, up and down are not told apart (no pinna). |

```ogt
enum PanningModel {
  doc "How an AudioListener places spatial sources between the ears, as Web Audio's PannerNode.panningModel."
  variant equalpower "The stereo panner (Web Audio's equal-power law): for speakers."
  variant hrtf "For headphones: a spherical-head model (Brown and Duda, 1998) that delays the far ear by the interaural time difference (up to 0.66 ms) and shadows its highs (up to -20 dB), so a source moves between the ears; front and back, up and down are not told apart (no pinna)."
}
```

<sub>Module `audio` line 28</sub>
