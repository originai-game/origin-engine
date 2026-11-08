<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-setVirtualCursor"></a>

### `og.ui.setVirtualCursor`

A pad's virtual cursor, for a screen built for the pointer (a map, a shop grid, a strategy HUD) played on a pad: pad 0's left stick moves it instead of the focus, slowing and sticking over controls; its A presses where it points as the mouse's button does (drags, sliders and long presses work); its right stick scrolls under it. The style draws it and the system cursor hides (og.ui.frameCursor answers none). The real pointer moving hides it and the stick shows it again. Declare it every tick the screen shows; it takes effect from the next tick. Null turns it off.

```ts
og.ui.setVirtualCursor(desc: VirtualCursorDesc | null): void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `desc` | `VirtualCursorDesc \| null` | no | How it moves; null is none. |

**Returns** `void`.

Binding symbol `og_ui_set_virtual_cursor` · publish builds: kept

```ogt
fn setVirtualCursor {
  doc "A pad's virtual cursor, for a screen built for the pointer (a map, a shop grid, a strategy HUD) played on a pad: pad 0's left stick moves it instead of the focus, slowing and sticking over controls; its A presses where it points as the mouse's button does (drags, sliders and long presses work); its right stick scrolls under it. The style draws it and the system cursor hides (og.ui.frameCursor answers none). The real pointer moving hides it and the stick shows it again. Declare it every tick the screen shows; it takes effect from the next tick. Null turns it off."
  param desc Option.VirtualCursorDesc "How it moves; null is none."
  returns Void
}
```

<sub>Module `ui` line 1524</sub>
