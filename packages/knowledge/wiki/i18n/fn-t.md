<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`i18n`](../i18n.md) · [index](../index.md)

<a id="fn-t"></a>

### `og.i18n.t`

A key's message in the active locale, formatted with its arguments; a key no table has answers the key itself and logs one warning.

Also known as: translate, localised string, localized text, gettext, Godot tr, Unity LocalizedString, translation key

```ts
og.i18n.t(world: World, key: string, args?: unknown): string
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `key` | `string` | no | The key, such as `menu.start`. |
| `args` | `unknown` | yes | The arguments: an object of strings and numbers, `{ count: 3, name: "Kai" }`. |

**Returns** `string`.

Binding symbol `og_i18n_t` · publish builds: kept

```ogt
fn t {
  doc "A key's message in the active locale, formatted with its arguments; a key no table has answers the key itself and logs one warning."
  aliases "translate, localised string, localized text, gettext, Godot tr, Unity LocalizedString, translation key"
  param world World "The world."
  param key String "The key, such as `menu.start`."
  param args Json "The arguments: an object of strings and numbers, `{ count: 3, name: \"Kai\" }`." {
    optional true
  }
  returns String
}
```

<sub>`api.ogt` line 15562</sub>
