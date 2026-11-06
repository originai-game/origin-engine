<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-zoomView"></a>

### `og.ui.zoomView`

A zoomable picture (a map, an artwork): two fingers pinch it to zoom about their centre and pan it as they move (og.Pointers), the wheel zooms about the pointer, a drag pans; Tab or a pad focuses it, and then + (= and the keypad's) and - zoom it about its centre, 0 fits it again and the arrows pan it a tenth of its size. It stays on its picture and within its zoom limits; its zoom and centre are UI state (snapshots carry them). Two fingers over a scroll container scroll it the same way, and fling it as they lift. The probe reads its zoom as its value.

```ts
og.ui.zoomView(id: string, desc?: ZoomViewDesc): ZoomResult
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `desc` | `ZoomViewDesc` | yes | The picture, its size and zoom limits. |

**Returns** `ZoomResult`.

Binding symbol `og_ui_zoom_view` · publish builds: kept

```ogt
fn zoomView {
  doc "A zoomable picture (a map, an artwork): two fingers pinch it to zoom about their centre and pan it as they move (og.Pointers), the wheel zooms about the pointer, a drag pans; Tab or a pad focuses it, and then + (= and the keypad's) and - zoom it about its centre, 0 fits it again and the arrows pan it a tenth of its size. It stays on its picture and within its zoom limits; its zoom and centre are UI state (snapshots carry them). Two fingers over a scroll container scroll it the same way, and fling it as they lift. The probe reads its zoom as its value."
  param id String "Its id."
  param desc ZoomViewDesc "The picture, its size and zoom limits."
  returns ZoomResult
}
```

<sub>`api.ogt` line 16370</sub>
