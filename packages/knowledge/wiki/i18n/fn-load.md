<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`i18n`](../i18n.md) · [index](../index.md)

<a id="fn-load"></a>

### `og.i18n.load`

Load a cooked string table; the active locale is negotiated again.

```ts
og.i18n.load(world: World, path: string): Result<LocaleInfo>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `path` | `string` | no | The table, such as `strings/ja.ogt`. |

**Returns** `Result<LocaleInfo>`: Refused for a path that is not a cooked string table, or a font of its chain that is not cooked.

Binding symbol `og_i18n_load` · publish builds: kept

```ogt
fn load {
  doc "Load a cooked string table; the active locale is negotiated again."
  param world World "The world."
  param path String "The table, such as `strings/ja.ogt`."
  returns Result.LocaleInfo "Refused for a path that is not a cooked string table, or a font of its chain that is not cooked."
}
```

<sub>`api.ogt` line 15556</sub>
