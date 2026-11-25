<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`video`](../video.md) · [index](../index.md)

<a id="type-VideoBuffering"></a>

### `VideoBuffering` (struct)

Presentation-only buffering status of active video streams.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `buffering` | `boolean` | `false` | — | — | — | Whether any active player with `wait` is currently holding the clock for decode or chunk download. |
| `progress` | `number` | `0` | — | `ratio` | — | 0 to 1: download progress of the active stream's next needed chunk (1 when ready). |
| `quality` | `VideoQuality` | `"auto"` | — | — | — | The quality tier currently active for streamed playback. |

```ogt
struct VideoBuffering {
  doc "Presentation-only buffering status of active video streams."
  field buffering bool "Whether any active player with `wait` is currently holding the clock for decode or chunk download."
  field progress f32 "0 to 1: download progress of the active stream's next needed chunk (1 when ready)." {
    unit ratio
  }
  field quality VideoQuality "The quality tier currently active for streamed playback."
}
```

<sub>Module `video` line 51</sub>
