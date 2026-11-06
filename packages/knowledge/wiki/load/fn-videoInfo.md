<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`load`](../load.md) · [index](../index.md)

<a id="fn-videoInfo"></a>

### `og.load.videoInfo`

A loaded video's size, frames, rate, length, texture and sound.

```ts
og.load.videoInfo(world: World, video: Video): Result<VideoInfo>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world that loaded it. |
| `video` | `Video` | no | The video. |

**Returns** `Result<VideoInfo>`: Refused for a handle og.load.video did not answer.

Binding symbol `og_load_video_info` · publish builds: kept

```ogt
fn videoInfo {
  doc "A loaded video's size, frames, rate, length, texture and sound."
  param world World "The world that loaded it."
  param video Video "The video."
  returns Result.VideoInfo "Refused for a handle og.load.video did not answer."
}
```

<sub>Module `load` line 416</sub>
