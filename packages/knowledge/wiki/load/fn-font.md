<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`load`](../load.md) · [index](../index.md)

<a id="fn-font"></a>

### `og.load.font`

Load a cooked font for og.render.text, or an icon set (an `.ogt` of `type icons`: SVG icons cooked into a distance-field atlas like a font) for UiDesc.icons.font. Synchronous on both hosts: an ungrouped font-pack face must be in the cook's boot preload set. Undeclared paths are refused on both hosts, even if their bytes are resident; name them in game code or project.ogt publish.keep and recook (loading guide §1). Optional locale coverage uses og.i18n.

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
  doc "Load a cooked font for og.render.text, or an icon set (an `.ogt` of `type icons`: SVG icons cooked into a distance-field atlas like a font) for UiDesc.icons.font. Synchronous on both hosts: an ungrouped font-pack face must be in the cook's boot preload set. Undeclared paths are refused on both hosts, even if their bytes are resident; name them in game code or project.ogt publish.keep and recook (loading guide §1). Optional locale coverage uses og.i18n."
  param world World "The world that owns the font."
  param path String "The font's asset id, such as `fonts/NotoSansSC-Regular-subset.ttf`."
  returns Result.Font
}
```

<sub>Module `load` line 268</sub>
