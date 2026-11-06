<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`i18n`](../i18n.md) · [index](../index.md)

<a id="fn-has"></a>

### `og.i18n.has`

Whether the active locale's own table has the key (not a fallback).

```ts
og.i18n.has(world: World, key: string): boolean
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `key` | `string` | no | The key. |

**Returns** `boolean`.

Binding symbol `og_i18n_has` · publish builds: kept

```ogt
fn has {
  doc "Whether the active locale's own table has the key (not a fallback)."
  param world World "The world."
  param key String "The key."
  returns bool
}
```

<sub>Module `i18n` line 75</sub>
