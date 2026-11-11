<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="fn-text"></a>

### `og.render.text`

Draw text this frame. Systems and commands may call it; the list is cleared at every tick, so submit it every tick the text should show. Multi-page fonts are drawn page by page with one layout.

Also known as: draw text, world text, label

```ts
og.render.text(world: World, desc?: TextDesc): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world whose renderer draws it. |
| `desc` | `TextDesc` | yes | The font, text, placement and colour. |

**Returns** `Result<void>`: Refused for a dead font.

Binding symbol `og_render_text` · publish builds: kept

```ogt
fn text {
  doc "Draw text this frame. Systems and commands may call it; the list is cleared at every tick, so submit it every tick the text should show. Multi-page fonts are drawn page by page with one layout."
  aliases "draw text, world text, label"
  param world World "The world whose renderer draws it."
  param desc TextDesc "The font, text, placement and colour."
  returns Result.Void "Refused for a dead font."
}
```

<sub>Module `render` line 3069</sub>
