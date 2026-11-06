<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`load`](../load.md) · [index](../index.md)

<a id="fn-style"></a>

### `og.load.style`

Load a cooked UI style (an `.ogt` of `type style`: every palette role, the radii, type ramp, durations, and its art: the atlas and its nine-slice frames, ornaments, fonts, icons, motion and sounds; the grammar is og-ui's `parse_style`) for UiDesc.style. Version 2 (docs/ui.md §12, what `og style upgrade` writes from version 1 and every built-in style is) adds the tokens (rarity, team, damage and chart colours each with its cue, the type scale per breakpoint, the space, size, radius and stroke scales, elevation, the shape language, motion springs and eases, sounds and haptics per UI event) and dresses parts by selector (`dress "Button.base[pressed]" { … }`); the tokens are carried, not yet drawn from. Version 1 still loads. Its atlas, fonts, icons and sounds are loaded with it. Loaded once per path. A built-in style a look names is in the bundle as `@styles/<name>/style.ogt`.

```ts
og.load.style(world: World, path: string): Result<Style>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world whose UI uses it. |
| `path` | `string` | no | The asset id, such as `styles/pixel/style.ogt` or `@styles/pixel/style.ogt`. |

**Returns** `Result<Style>`: Refused for a path not in the bundle, a style that does not parse (the reason names the line) or an asset it names that cannot load.

Binding symbol `og_load_style` · publish builds: kept

```ogt
fn style {
  doc "Load a cooked UI style (an `.ogt` of `type style`: every palette role, the radii, type ramp, durations, and its art: the atlas and its nine-slice frames, ornaments, fonts, icons, motion and sounds; the grammar is og-ui's `parse_style`) for UiDesc.style. Version 2 (docs/ui.md §12, what `og style upgrade` writes from version 1 and every built-in style is) adds the tokens (rarity, team, damage and chart colours each with its cue, the type scale per breakpoint, the space, size, radius and stroke scales, elevation, the shape language, motion springs and eases, sounds and haptics per UI event) and dresses parts by selector (`dress \"Button.base[pressed]\" { … }`); the tokens are carried, not yet drawn from. Version 1 still loads. Its atlas, fonts, icons and sounds are loaded with it. Loaded once per path. A built-in style a look names is in the bundle as `@styles/<name>/style.ogt`."
  param world World "The world whose UI uses it."
  param path String "The asset id, such as `styles/pixel/style.ogt` or `@styles/pixel/style.ogt`."
  returns Result.Style "Refused for a path not in the bundle, a style that does not parse (the reason names the line) or an asset it names that cannot load."
}
```

<sub>`api.ogt` line 8228</sub>
