<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-frameGlyph"></a>

### `og.ui.frameGlyph`

An action's glyph on the device used last, as og.ui.glyph answers it, without the world: for a component that shows a prompt (a tab row's bumpers, the action bar) and only builds UI. It follows the device live (keyboard, Xbox, PlayStation, Switch, a generic pad, touch) and the player's rebinding. A name that is no UI action answers as og.input.glyph does for the world whose frame is being built (a gameplay action an ability slot names). Null for an unknown action, one with no binding on that device, or outside a frame being built.

```ts
og.ui.frameGlyph(action: string): UiGlyph | null
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `action` | `string` | no | The action. |

**Returns** `UiGlyph | null`.

Binding symbol `og_ui_frame_glyph` · publish builds: kept

```ogt
fn frameGlyph {
  doc "An action's glyph on the device used last, as og.ui.glyph answers it, without the world: for a component that shows a prompt (a tab row's bumpers, the action bar) and only builds UI. It follows the device live (keyboard, Xbox, PlayStation, Switch, a generic pad, touch) and the player's rebinding. A name that is no UI action answers as og.input.glyph does for the world whose frame is being built (a gameplay action an ability slot names). Null for an unknown action, one with no binding on that device, or outside a frame being built."
  param action String "The action."
  returns Option.UiGlyph
}
```

<sub>Module `ui` line 8112</sub>
