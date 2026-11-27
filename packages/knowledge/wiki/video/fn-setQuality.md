<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`video`](../video.md) · [index](../index.md)

<a id="fn-setQuality"></a>

### `og.video.setQuality`

Presentation-only: choose the quality tier (`auto`, `high`, `medium`, `low`) for streamed videos. Never affects simulation ticks or world hashes.

```ts
og.video.setQuality(world: World, quality: VideoQuality): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `quality` | `VideoQuality` | no | The quality tier. |

**Returns** `Result<void>`: Refused for an unknown quality tier.

Binding symbol `og_video_set_quality` · publish builds: kept

```ogt
fn setQuality {
  doc "Presentation-only: choose the quality tier (`auto`, `high`, `medium`, `low`) for streamed videos. Never affects simulation ticks or world hashes."
  param world World "The world."
  param quality VideoQuality "The quality tier."
  returns Result.Void "Refused for an unknown quality tier."
}
```

<sub>Module `video` line 76</sub>
