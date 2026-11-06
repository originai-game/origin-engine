<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`i18n`](../i18n.md) · [index](../index.md)

<a id="fn-locale"></a>

### `og.i18n.locale`

The requested and active locales, the direction and the font chain.

```ts
og.i18n.locale(world: World): LocaleInfo
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |

**Returns** `LocaleInfo`.

Binding symbol `og_i18n_locale` · publish builds: kept

```ogt
fn locale {
  doc "The requested and active locales, the direction and the font chain."
  param world World "The world."
  returns LocaleInfo
}
```

<sub>`api.ogt` line 15570</sub>
