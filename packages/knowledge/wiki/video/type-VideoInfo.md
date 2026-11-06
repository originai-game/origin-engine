<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`video`](../video.md) · [index](../index.md)

<a id="type-VideoInfo"></a>

### `VideoInfo` (struct)

What og.load.video loaded.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `video` | `Video` | `0` | — | — | — | The video. |
| `texture` | `Texture` | `0` | — | — | — | The texture that shows the frame its og.VideoPlayer is at (frame 0 until one plays it). |
| `sound` | `Sound` | `0` | — | — | — | Its sound track (og.load.sound's kind of handle, for an og.AudioSource); 0 when it has none or was cooked with `audio false`. |
| `width` | `number` | `0` | — | — | — | Each frame's width in texels. |
| `height` | `number` | `0` | — | — | — | Each frame's height in texels. |
| `frames` | `number` | `0` | — | — | — | Frames kept (at most 256). |
| `fps` | `number` | `0` | — | — | — | Frames per second kept. |
| `seconds` | `number` | `0` | — | `s` | — | frames / fps. |
| `format` | `string` | `""` | — | — | — | The frames' encoding on the GPU (`rgba8`, `bc7`, `astc4x4`, `etc2`; a stream's texture is `rgba8`). |
| `bytes` | `number` | `0` | — | `bytes` | — | Bytes its frames take on the GPU (a stream: its texture's, one frame). |
| `streamed` | `boolean` | `false` | — | — | — | Whether it is a stream (`stream true`: decoded as it plays) rather than a clip of cooked frames. |
| `codec` | `string` | `""` | — | — | — | A stream's codec (`av1`); empty for a clip. |
| `poster` | `Texture` | `0` | — | — | — | A stream's first frame as an ordinary texture, ready as soon as the bundle is (a menu's thumbnail, the picture while the stream is fetched); for a clip, VideoInfo.texture. |

```ogt
struct VideoInfo {
  doc "What og.load.video loaded."
  field video Video "The video."
  field texture Texture "The texture that shows the frame its og.VideoPlayer is at (frame 0 until one plays it)."
  field sound Sound "Its sound track (og.load.sound's kind of handle, for an og.AudioSource); 0 when it has none or was cooked with `audio false`."
  field width u32 "Each frame's width in texels."
  field height u32 "Each frame's height in texels."
  field frames u32 "Frames kept (at most 256)."
  field fps u32 "Frames per second kept."
  field seconds f32 "frames / fps." {
    unit s
  }
  field format String "The frames' encoding on the GPU (`rgba8`, `bc7`, `astc4x4`, `etc2`; a stream's texture is `rgba8`)."
  field bytes f64 "Bytes its frames take on the GPU (a stream: its texture's, one frame)." {
    unit bytes
  }
  field streamed bool "Whether it is a stream (`stream true`: decoded as it plays) rather than a clip of cooked frames."
  field codec String "A stream's codec (`av1`); empty for a clip."
  field poster Texture "A stream's first frame as an ordinary texture, ready as soon as the bundle is (a menu's thumbnail, the picture while the stream is fetched); for a clip, VideoInfo.texture."
}
```

<sub>`api.ogt` line 25647</sub>
