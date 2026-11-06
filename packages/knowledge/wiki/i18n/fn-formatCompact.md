<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`i18n`](../i18n.md) · [index](../index.md)

<a id="fn-formatCompact"></a>

### `og.i18n.formatCompact`

A number written short in the active locale: `1.2K`, `12K`, `1.2M` (en), `1,2 Mio.` (de), `1.2万` (ja). Below the locale's first compact size (de compacts no thousands) the whole number, grouped.

```ts
og.i18n.formatCompact(world: World, value: number): string
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world (its active locale). |
| `value` | `number` | no | The number. |

**Returns** `string`.

Binding symbol `og_i18n_format_compact` · publish builds: kept

```ogt
fn formatCompact {
  doc "A number written short in the active locale: `1.2K`, `12K`, `1.2M` (en), `1,2 Mio.` (de), `1.2万` (ja). Below the locale's first compact size (de compacts no thousands) the whole number, grouped."
  param world World "The world (its active locale)."
  param value f64 "The number."
  returns String
}
```

<sub>`api.ogt` line 15602</sub>
