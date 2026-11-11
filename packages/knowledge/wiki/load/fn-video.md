<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`load`](../load.md) · [index](../index.md)

<a id="fn-video"></a>

### `og.load.video`

Load cooked footage (an `.ogt` of `type video`), which og cook turns with ffmpeg into a clip (a texture array of its frames) or a stream (AV1, decoded as it plays) and its sound track. Either way it makes the video's Texture (VideoInfo.texture), which shows the frame its og.VideoPlayer is at, drawn like any texture by a material, an og.Sprite or a UI image. Load it in GameDesc.start. Loading the same path again answers the same handles.

See `loading §9`.

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
  doc "Load cooked footage (an `.ogt` of `type video`), which og cook turns with ffmpeg into a clip (a texture array of its frames) or a stream (AV1, decoded as it plays) and its sound track. Either way it makes the video's Texture (VideoInfo.texture), which shows the frame its og.VideoPlayer is at, drawn like any texture by a material, an og.Sprite or a UI image. Load it in GameDesc.start. Loading the same path again answers the same handles."
  guide "loading §9"
  param world World "The world whose renderer and audio own it."
  param path String "The video's asset id: the `.ogt`, such as `clips/intro.ogt`."
  returns Result.VideoInfo "Refused for a path not in the bundle (run og cook: a video needs ffmpeg there, og doctor) or not a video."
}
```

<sub>Module `load` line 435</sub>
