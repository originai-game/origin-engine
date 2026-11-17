<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-remoteFrames"></a>

### `og.ui.remoteFrames`

Whether this engine shows remote `<iframe>`s (a capability query, the same on every host): true only for a game whose project.ogt lists `frames`, which og build ships the engine with remote frames to. Where false, a remote src is refused, no remote document is fetched and og.ui.frameMessages stays empty; local `<iframe src="x.svelte">` frames work everywhere.

```ts
og.ui.remoteFrames(world: World): boolean
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |

**Returns** `boolean`.

Binding symbol `og_ui_remote_frames` · publish builds: kept

```ogt
fn remoteFrames {
  doc "Whether this engine shows remote `<iframe>`s (a capability query, the same on every host): true only for a game whose project.ogt lists `frames`, which og build ships the engine with remote frames to. Where false, a remote src is refused, no remote document is fetched and og.ui.frameMessages stays empty; local `<iframe src=\"x.svelte\">` frames work everywhere."
  param world World "The world."
  returns bool
}
```

<sub>Module `ui` line 1834</sub>
