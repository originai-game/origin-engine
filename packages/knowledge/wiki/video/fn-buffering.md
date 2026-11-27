<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`video`](../video.md) · [index](../index.md)

<a id="fn-buffering"></a>

### `og.video.buffering`

Presentation-only: whether any player with `wait` is currently buffering, its download progress, and the active quality tier.

```ts
og.video.buffering(world: World): VideoBuffering
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |

**Returns** `VideoBuffering`.

Binding symbol `og_video_buffering` · publish builds: kept

```ogt
fn buffering {
  doc "Presentation-only: whether any player with `wait` is currently buffering, its download progress, and the active quality tier."
  param world World "The world."
  returns VideoBuffering
}
```

<sub>Module `video` line 83</sub>
