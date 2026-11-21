<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-frameMessages"></a>

### `og.ui.frameMessages`

Take what the documents of remote `<iframe>`s sent this frame: their clicks, changed controls and named events, each with the origin it came from. Always empty where og.ui.remoteFrames answers false.

```ts
og.ui.frameMessages(world: World): readonly UiFrameMessage[]
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |

**Returns** `readonly UiFrameMessage[]`.

Binding symbol `og_ui_frame_messages` · publish builds: kept

```ogt
fn frameMessages {
  doc "Take what the documents of remote `<iframe>`s sent this frame: their clicks, changed controls and named events, each with the origin it came from. Always empty where og.ui.remoteFrames answers false."
  param world World "The world."
  returns List.UiFrameMessage
}
```

<sub>Module `ui` line 1642</sub>
