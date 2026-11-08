<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-previewViewport"></a>

### `og.ui.previewViewport`

Lay the UI out for another play surface (a phone in portrait at 3x, a notch) until cleared with null: for headless previews and playtests of responsive layouts. Render at width x dpr by height x dpr (`og run --size`).

```ts
og.ui.previewViewport(world: World, viewport: Viewport | null): void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `viewport` | `Viewport \| null` | no | The surface to lay out for; null returns to the host's. |

**Returns** `void`.

Binding symbol `og_ui_preview_viewport` · publish builds: kept

```ogt
fn previewViewport {
  doc "Lay the UI out for another play surface (a phone in portrait at 3x, a notch) until cleared with null: for headless previews and playtests of responsive layouts. Render at width x dpr by height x dpr (`og run --size`)."
  param world World "The world."
  param viewport Option.Viewport "The surface to lay out for; null returns to the host's."
  returns Void
}
```

<sub>Module `ui` line 1266</sub>
