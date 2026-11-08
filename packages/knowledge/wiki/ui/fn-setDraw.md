<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-setDraw"></a>

### `og.ui.setDraw`

Draws the element added last (a container just opened, or the last control) with desc: shapes beyond rectangles, layered gradient, image and vector fills, strokes with their own paints, shadows, glows, glass, clipping to its shape, masks, blend modes and materials (docs/ui.md §3.3). Call it right after the element's open or control; immediate mode, so every tick it is drawn.

```ts
og.ui.setDraw(desc?: DrawDesc): void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `desc` | `DrawDesc` | yes | What it draws with. |

**Returns** `void`.

Binding symbol `og_ui_set_draw` · publish builds: kept

```ogt
fn setDraw {
  doc "Draws the element added last (a container just opened, or the last control) with desc: shapes beyond rectangles, layered gradient, image and vector fills, strokes with their own paints, shadows, glows, glass, clipping to its shape, masks, blend modes and materials (docs/ui.md §3.3). Call it right after the element's open or control; immediate mode, so every tick it is drawn."
  param desc DrawDesc "What it draws with."
  returns Void
}
```

<sub>Module `ui` line 3405</sub>
