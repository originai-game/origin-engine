<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`probe`](../probe.md) · [index](../index.md)

<a id="type-DrawReason"></a>

### `DrawReason` (enum)

Why the main camera draws an entity or not, the first that holds.

See `testing §4`.

A string: one of the variants below. The zero value is `"drawn"`.

| Variant | Doc |
|---|---|
| `"drawn"` | A drawable of it survives the camera's cull. |
| `"hlod"` | Its model is drawn by an HLOD proxy in its place. |
| `"hidden"` | A component of it that draws has `visible` false; detail names the field. |
| `"loading"` | Its model has not loaded yet, or is held back until its textures arrive; detail names the asset. |
| `"layer"` | Every drawable of it is off the camera's layers (og.Camera.layers). |
| `"outside_view"` | Every drawable of it lies outside the camera's view, past near or far included. |
| `"occluded"` | A drawable of it is inside the camera's view but behind what the occluders drew (the GPU cull's occlusion test), and none is drawn. |
| `"fading_in"` | It is fading in (og.render.reveal) and none of it shows yet. |
| `"untracked"` | It draws through a path the cull record does not follow (particles, lines, text, a tilemap, a wireframe material); detail names the component, or `wireframe`. Whether it shows is not known. |
| `"nothing_to_draw"` | It has no component that draws. |

```ogt
enum DrawReason {
  doc "Why the main camera draws an entity or not, the first that holds."
  guide "testing §4"
  variant drawn "A drawable of it survives the camera's cull."
  variant hlod "Its model is drawn by an HLOD proxy in its place."
  variant hidden "A component of it that draws has `visible` false; detail names the field."
  variant loading "Its model has not loaded yet, or is held back until its textures arrive; detail names the asset."
  variant layer "Every drawable of it is off the camera's layers (og.Camera.layers)."
  variant outside_view "Every drawable of it lies outside the camera's view, past near or far included."
  variant occluded "A drawable of it is inside the camera's view but behind what the occluders drew (the GPU cull's occlusion test), and none is drawn."
  variant fading_in "It is fading in (og.render.reveal) and none of it shows yet."
  variant untracked "It draws through a path the cull record does not follow (particles, lines, text, a tilemap, a wireframe material); detail names the component, or `wireframe`. Whether it shows is not known."
  variant nothing_to_draw "It has no component that draws."
}
```

<sub>Module `probe` line 465</sub>
