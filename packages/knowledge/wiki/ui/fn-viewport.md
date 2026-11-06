<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-viewport"></a>

### `og.ui.viewport`

This tick's play surface as the UI lays out on it: the render target's size, safe area and covered region in og-ui units (CSS pixels over UiTheme.unit, docs/ui.md §15.2: 1 on touch and up to 720p, so CSS pixels there) and device pixels per unit, from what the host reports.

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
  doc "This tick's play surface as the UI lays out on it: the render target's size, safe area and covered region in og-ui units (CSS pixels over UiTheme.unit, docs/ui.md §15.2: 1 on touch and up to 720p, so CSS pixels there) and device pixels per unit, from what the host reports."
  param world World "The world."
  returns Viewport
}
```

<sub>`api.ogt` line 16245</sub>
