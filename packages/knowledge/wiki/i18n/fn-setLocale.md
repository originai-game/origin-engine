<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`i18n`](../i18n.md) · [index](../index.md)

<a id="fn-setLocale"></a>

### `og.i18n.setLocale`

Ask for a locale (a player's choice in settings); the active one is the best table for it.

Also known as: language, change language, locale, localisation, localization, Godot TranslationServer.set_locale

```ts
og.i18n.setLocale(world: World, locale: string): LocaleInfo
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `locale` | `string` | no | A BCP 47 tag: `ja`, `zh-CN`, `ar-EG`. |

**Returns** `LocaleInfo`.

Binding symbol `og_i18n_set_locale` · publish builds: kept

```ogt
fn setLocale {
  doc "Ask for a locale (a player's choice in settings); the active one is the best table for it."
  aliases "language, change language, locale, localisation, localization, Godot TranslationServer.set_locale"
  param world World "The world."
  param locale String "A BCP 47 tag: `ja`, `zh-CN`, `ar-EG`."
  returns LocaleInfo
}
```

<sub>`api.ogt` line 15562</sub>
