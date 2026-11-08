<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-glyph"></a>

### `og.ui.glyph`

An action's glyph on the device used last: its first binding there. Null for an unknown action or one with no binding on that device.

```ts
og.ui.glyph(world: World, action: string): UiGlyph | null
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `action` | `string` | no | The action. |

**Returns** `UiGlyph | null`.

Binding symbol `og_ui_glyph` · publish builds: kept

```ogt
fn glyph {
  doc "An action's glyph on the device used last: its first binding there. Null for an unknown action or one with no binding on that device."
  param world World "The world."
  param action String "The action."
  returns Option.UiGlyph
}
```

<sub>Module `ui` line 3420</sub>
