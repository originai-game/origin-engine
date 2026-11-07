<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`i18n`](../i18n.md) · [index](../index.md)

<a id="fn-formatNumber"></a>

### `og.i18n.formatNumber`

A decimal number using the active locale's grouping, separators, native digits and sign. pseudo/en-XA expands and brackets output; ar-XB also isolates it right-to-left. Pseudo requests remain active for formatting even when translations fall back to a real table.

```ts
og.i18n.formatNumber(world: World, value: number, format?: DecimalFormat): string
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world (its active locale). |
| `value` | `number` | no | The number. |
| `format` | `DecimalFormat` | yes | Precision and grouping. |

**Returns** `string`.

Binding symbol `og_i18n_format_number` · publish builds: kept

```ogt
fn formatNumber {
  doc "A decimal number using the active locale's grouping, separators, native digits and sign. pseudo/en-XA expands and brackets output; ar-XB also isolates it right-to-left. Pseudo requests remain active for formatting even when translations fall back to a real table."
  param world World "The world (its active locale)."
  param value f64 "The number."
  param format DecimalFormat "Precision and grouping." {
    optional true
  }
  returns String
}
```

<sub>Module `i18n` line 93</sub>
