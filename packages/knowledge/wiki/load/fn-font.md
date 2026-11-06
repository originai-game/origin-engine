<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`load`](../load.md) · [index](../index.md)

<a id="fn-font"></a>

### `og.load.font`

Load a cooked MSDF font for og.render.text, or an icon set (an `.ogt` of `type icons`: SVG icons cooked into a distance-field atlas like a font) for UiDesc.icons.font.

```ts
og.load.font(world: World, path: string): Result<Font>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world that owns the font. |
| `path` | `string` | no | The font's asset id, such as `fonts/NotoSansSC-Regular-subset.ttf`. |

**Returns** `Result<Font>`.

Binding symbol `og_load_font` · publish builds: kept

```ogt
fn font {
  doc "Load a cooked MSDF font for og.render.text, or an icon set (an `.ogt` of `type icons`: SVG icons cooked into a distance-field atlas like a font) for UiDesc.icons.font."
  param world World "The world that owns the font."
  param path String "The font's asset id, such as `fonts/NotoSansSC-Regular-subset.ttf`."
  returns Result.Font
}
```

<sub>`api.ogt` line 8140</sub>
