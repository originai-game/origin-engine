<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`load`](../load.md) · [index](../index.md)

<a id="fn-style"></a>

### `og.load.style`

Load a cooked UI style (an `.ogt` of `type style`: the palette roles, radii, type ramp, durations and its art; the grammar is og-ui's `parse_style`) for UiDesc.style. Version 3 is the CSS dialect every built-in style uses; versions 1 and 2 still load. Its atlas, fonts, icons and sounds are loaded with it. Loaded once per path. A built-in style a look names is in the bundle as `@styles/<name>/style.ogt`.

See `loading §6`.

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
  doc "Load a cooked UI style (an `.ogt` of `type style`: the palette roles, radii, type ramp, durations and its art; the grammar is og-ui's `parse_style`) for UiDesc.style. Version 3 is the CSS dialect every built-in style uses; versions 1 and 2 still load. Its atlas, fonts, icons and sounds are loaded with it. Loaded once per path. A built-in style a look names is in the bundle as `@styles/<name>/style.ogt`."
  guide "loading §6"
  param world World "The world whose UI uses it."
  param path String "The asset id, such as `styles/pixel/style.ogt` or `@styles/pixel/style.ogt`."
  returns Result.Style "Refused for a path not in the bundle, a style that does not parse (the reason names the line) or an asset it names that cannot load."
}
```

<sub>Module `load` line 337</sub>
