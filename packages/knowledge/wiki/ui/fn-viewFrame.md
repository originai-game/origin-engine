<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-viewFrame"></a>

### `og.ui.viewFrame`

A viewport in a frame (docs/ui.md §5.5, Foundations): og.ui.sceneViewport in a style-dressed slot that turns with a drag (keeping the fling's speed for a moment), tilts within its limits, zooms with the wheel or a pinch, and turns by 15° per Left or Right while it has the focus (Tab, a pad). Until the target has a texture it shows a placeholder with a spinner. Draw the scene into a target of view.pixelWidth × view.pixelHeight with the yaw, pitch and zoom it answers; a target of another shape covers the frame, cropped, never squashed. Parts: `ViewFrame.frame[focused]`, `ViewFrame.hint`. Role image (its label), its turn as its value. Probe: `<id>`, `<id>/Box/Picture/View`, `<id>/Box/HintRow/Hint`.

```ts
og.ui.viewFrame(id: string, desc?: ViewFrameDesc): ViewFrameResult
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `desc` | `ViewFrameDesc` | yes | The target, its camera and its turn. |

**Returns** `ViewFrameResult`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn viewFrame {
  doc "A viewport in a frame (docs/ui.md §5.5, Foundations): og.ui.sceneViewport in a style-dressed slot that turns with a drag (keeping the fling's speed for a moment), tilts within its limits, zooms with the wheel or a pinch, and turns by 15° per Left or Right while it has the focus (Tab, a pad). Until the target has a texture it shows a placeholder with a spinner. Draw the scene into a target of view.pixelWidth × view.pixelHeight with the yaw, pitch and zoom it answers; a target of another shape covers the frame, cropped, never squashed. Parts: `ViewFrame.frame[focused]`, `ViewFrame.hint`. Role image (its label), its turn as its value. Probe: `<id>`, `<id>/Box/Picture/View`, `<id>/Box/HintRow/Hint`."
  host false
  param id String "Its id."
  param desc ViewFrameDesc "The target, its camera and its turn."
  returns ViewFrameResult
}
```

<sub>`api.ogt` line 19130</sub>
