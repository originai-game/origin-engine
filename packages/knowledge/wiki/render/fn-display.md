<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="fn-display"></a>

### `og.render.display`

What the display draws now: the settings, the frame cap resolved, the render scale automatic quality chose, the scene's size, the device losses survived, the GPU memory, and what the browser decides (vsync, fullscreen now, the adapter's power preference and whether a restart is needed).

```ts
og.render.display(world: World): DisplayInfo
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world asked. |

**Returns** `DisplayInfo`.

Binding symbol `og_render_display` · publish builds: kept

```ogt
fn display {
  doc "What the display draws now: the settings, the frame cap resolved, the render scale automatic quality chose, the scene's size, the device losses survived, the GPU memory, and what the browser decides (vsync, fullscreen now, the adapter's power preference and whether a restart is needed)."
  param world World "The world asked."
  returns DisplayInfo
}
```

<sub>Module `render` line 2845</sub>
