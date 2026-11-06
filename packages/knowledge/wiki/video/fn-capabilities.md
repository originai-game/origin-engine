<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`video`](../video.md) · [index](../index.md)

<a id="fn-capabilities"></a>

### `og.video.capabilities`

How streams play on this host: the decoder, whether it is hardware and its largest real-time size.

```ts
og.video.capabilities(world: World): VideoCapabilities
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world asked. |

**Returns** `VideoCapabilities`.

Binding symbol `og_video_capabilities` · publish builds: kept

```ogt
fn capabilities {
  doc "How streams play on this host: the decoder, whether it is hardware and its largest real-time size."
  param world World "The world asked."
  returns VideoCapabilities
}
```

<sub>`api.ogt` line 25691</sub>
