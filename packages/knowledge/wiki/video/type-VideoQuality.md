<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`video`](../video.md) · [index](../index.md)

<a id="type-VideoQuality"></a>

### `VideoQuality` (enum)

The quality tier for streamed video playback (presentation-only: never changes simulation).

A string: one of the variants below. The zero value is `"auto"`.

| Variant | Doc |
|---|---|
| `"auto"` | Automatic adaptive selection from decoder capabilities and playback stalls. |
| `"high"` | Source / highest cooked rendition. |
| `"medium"` | 720p rendition when cooked, else highest. |
| `"low"` | The 480p rendition (a stream cooked with `lowRendition true`), else the lowest cooked. |

```ogt
enum VideoQuality {
  doc "The quality tier for streamed video playback (presentation-only: never changes simulation)."
  variant auto "Automatic adaptive selection from decoder capabilities and playback stalls."
  variant high "Source / highest cooked rendition."
  variant medium "720p rendition when cooked, else highest."
  variant low "The 480p rendition (a stream cooked with `lowRendition true`), else the lowest cooked."
}
```

<sub>Module `video` line 43</sub>
