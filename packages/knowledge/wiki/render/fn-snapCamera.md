<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="fn-snapCamera"></a>

### `og.render.snapCamera`

Draw og.Camera at its tick pose on the frames until the next tick instead of blending from where it was: a cut. A camera move past RenderSettings.snapDistance snaps without it. Presentation only, as og.render.snap.

```ts
og.render.snapCamera(world: World): void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world whose renderer draws it. |

**Returns** `void`.

Binding symbol `og_render_snap_camera` · publish builds: kept

```ogt
fn snapCamera {
  doc "Draw og.Camera at its tick pose on the frames until the next tick instead of blending from where it was: a cut. A camera move past RenderSettings.snapDistance snaps without it. Presentation only, as og.render.snap."
  param world World "The world whose renderer draws it."
  returns Void
}
```

<sub>Module `render` line 785</sub>
