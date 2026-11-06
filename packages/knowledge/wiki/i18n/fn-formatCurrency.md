<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`i18n`](../i18n.md) · [index](../index.md)

<a id="fn-formatCurrency"></a>

### `og.i18n.formatCurrency`

An amount of money in the active locale: `$1,234.50` (en), `1.234,50 €` (de), `￥1,235` (ja), with the currency's own fraction digits (none for JPY and KRW). A code the subset lacks is written as itself.

```ts
og.i18n.formatCurrency(world: World, amount: number, currency: string): string
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world (its active locale). |
| `amount` | `number` | no | The amount, in the currency's whole units (dollars, not cents). |
| `currency` | `string` | no | An ISO 4217 code: `USD`, `EUR`, `JPY`. |

**Returns** `string`.

Binding symbol `og_i18n_format_currency` · publish builds: kept

```ogt
fn formatCurrency {
  doc "An amount of money in the active locale: `$1,234.50` (en), `1.234,50 €` (de), `￥1,235` (ja), with the currency's own fraction digits (none for JPY and KRW). A code the subset lacks is written as itself."
  param world World "The world (its active locale)."
  param amount f64 "The amount, in the currency's whole units (dollars, not cents)."
  param currency String "An ISO 4217 code: `USD`, `EUR`, `JPY`."
  returns String
}
```

<sub>`api.ogt` line 15601</sub>
