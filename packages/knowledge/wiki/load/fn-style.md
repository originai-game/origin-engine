<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`load`](../load.md) · [index](../index.md)

<a id="fn-style"></a>

### `og.load.style`

Load a cooked UI style from a style directory (`styles/<name>`, containing `theme.json`, `pack.json`, or both) or a built-in theme slug (`minimal`, `tactical-dark`, `fantasy-parchment`, etc.) for UiDesc.style. Legacy `style.ogt` CSS packs are removed and refused. Loaded once per path.

See `loading §6`.

```ts
og.load.style(world: World, path: string): Result<Style>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world whose UI uses it. |
| `path` | `string` | no | The style directory or built-in slug, such as `styles/main` or `tactical-dark`. |

**Returns** `Result<Style>`: Refused for a path not in the bundle, a legacy `style.ogt` path, or a theme/pack that fails to parse.

Binding symbol `og_load_style` · publish builds: kept

```ogt
fn style {
  doc "Load a cooked UI style from a style directory (`styles/<name>`, containing `theme.json`, `pack.json`, or both) or a built-in theme slug (`minimal`, `tactical-dark`, `fantasy-parchment`, etc.) for UiDesc.style. Legacy `style.ogt` CSS packs are removed and refused. Loaded once per path."
  guide "loading §6"
  param world World "The world whose UI uses it."
  param path String "The style directory or built-in slug, such as `styles/main` or `tactical-dark`."
  returns Result.Style "Refused for a path not in the bundle, a legacy `style.ogt` path, or a theme/pack that fails to parse."
}
```

<sub>Module `load` line 422</sub>
