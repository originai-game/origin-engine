<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-viewport"></a>

### `og.ui.viewport`

This tick's play surface as the UI lays out on it: the render target's size, safe area and covered region in CSS px and device pixels per CSS px, from what the host reports.

```ts
og.ui.viewport(world: World): Viewport
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |

**Returns** `Viewport`.

Binding symbol `og_ui_viewport` · publish builds: kept

```ogt
fn viewport {
  doc "This tick's play surface as the UI lays out on it: the render target's size, safe area and covered region in CSS px and device pixels per CSS px, from what the host reports."
  param world World "The world."
  returns Viewport
}
```

<sub>Module `ui` line 310</sub>
