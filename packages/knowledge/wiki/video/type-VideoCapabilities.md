<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`video`](../video.md) · [index](../index.md)

<a id="type-VideoCapabilities"></a>

### `VideoCapabilities` (struct)

How this host plays streams. Backend limits are reported here, never by missing members: every host loads and plays a stream through the same API.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `streamed` | `boolean` | `false` | — | — | — | Whether streams decode here: false only on a host without a renderer (a headless run without a GPU), where a stream's texture shows nothing and its sound plays as usual. |
| `decoder` | `string` | `""` | — | — | — | The AV1 decoder: `rav1d` (native, a thread per stream), `webcodecs` (the browser's VideoDecoder), `rav1d-wasm` (rav1d in a Worker of its own, where the browser has no AV1 decoder) or `none`. |
| `hardware` | `boolean` | `false` | — | — | — | Whether the browser reports its decoder as hardware-accelerated (WebCodecs' `prefer-hardware` is supported); false for rav1d. |
| `maxWidth` | `number` | `0` | — | — | — | The widest stream this decoder plays in real time at 30 fps; wider ones still play, with frames held where decoding falls behind. rav1d-wasm plays a larger stream's 720p rendition, cooked beside it, instead. See `audio §8`. |
| `maxHeight` | `number` | `0` | — | — | — | The tallest, as maxWidth. |

```ogt
struct VideoCapabilities {
  doc "How this host plays streams. Backend limits are reported here, never by missing members: every host loads and plays a stream through the same API."
  field streamed bool "Whether streams decode here: false only on a host without a renderer (a headless run without a GPU), where a stream's texture shows nothing and its sound plays as usual."
  field decoder String "The AV1 decoder: `rav1d` (native, a thread per stream), `webcodecs` (the browser's VideoDecoder), `rav1d-wasm` (rav1d in a Worker of its own, where the browser has no AV1 decoder) or `none`."
  field hardware bool "Whether the browser reports its decoder as hardware-accelerated (WebCodecs' `prefer-hardware` is supported); false for rav1d."
  field maxWidth u32 "The widest stream this decoder plays in real time at 30 fps; wider ones still play, with frames held where decoding falls behind. rav1d-wasm plays a larger stream's 720p rendition, cooked beside it, instead." {
    guide "audio §8"
  }
  field maxHeight u32 "The tallest, as maxWidth."
}
```

<sub>Module `video` line 36</sub>
