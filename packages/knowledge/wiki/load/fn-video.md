<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`load`](../load.md) · [index](../index.md)

<a id="fn-video"></a>

### `og.load.video`

Load cooked footage (an `.ogt` of `type video`: `source` an .mp4, .webm or .mov beside it or under assets/, `fps`, `maxSeconds`, `size W H`, `kind color|pixel`, `audio`, and for a stream `stream true`, `keyframe`, `crf`), which og cook turns with ffmpeg into a clip (a texture array of its frames) or a stream (AV1, decoded as it plays) and its sound track. A clip's frames are uploaded as one 2D-array texture; a stream's units are fetched (on the web, from now: it is not part of the startup bundle) and decoded ahead of its players. Either way it makes the video's Texture (VideoInfo.texture), which shows the frame its og.VideoPlayer is at: a material map, an og.Sprite or a UI image draws it like any texture. Load it in GameDesc.start (every process of an og render --jobs run loads it the same way, so its handles agree). Loading the same path again answers the same handles.

```ts
og.load.video(world: World, path: string): Result<VideoInfo>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world whose renderer and audio own it. |
| `path` | `string` | no | The video's asset id: the `.ogt`, such as `clips/intro.ogt`. |

**Returns** `Result<VideoInfo>`: Refused for a path not in the bundle (run og cook: a video needs ffmpeg there, og doctor) or not a video.

Binding symbol `og_load_video` · publish builds: kept

```ogt
fn video {
  doc "Load cooked footage (an `.ogt` of `type video`: `source` an .mp4, .webm or .mov beside it or under assets/, `fps`, `maxSeconds`, `size W H`, `kind color|pixel`, `audio`, and for a stream `stream true`, `keyframe`, `crf`), which og cook turns with ffmpeg into a clip (a texture array of its frames) or a stream (AV1, decoded as it plays) and its sound track. A clip's frames are uploaded as one 2D-array texture; a stream's units are fetched (on the web, from now: it is not part of the startup bundle) and decoded ahead of its players. Either way it makes the video's Texture (VideoInfo.texture), which shows the frame its og.VideoPlayer is at: a material map, an og.Sprite or a UI image draws it like any texture. Load it in GameDesc.start (every process of an og render --jobs run loads it the same way, so its handles agree). Loading the same path again answers the same handles."
  param world World "The world whose renderer and audio own it."
  param path String "The video's asset id: the `.ogt`, such as `clips/intro.ogt`."
  returns Result.VideoInfo "Refused for a path not in the bundle (run og cook: a video needs ffmpeg there, og doctor) or not a video."
}
```

<sub>`api.ogt` line 8320</sub>
