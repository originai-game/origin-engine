<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`i18n`](../i18n.md) · [index](../index.md)

<a id="fn-formatDate"></a>

### `og.i18n.formatDate`

A date in the active locale, in the style asked for: `Nov 2, 2026` (en), `02.11.2026` (de).

```ts
og.i18n.formatDate(world: World, seconds: number, format?: DateFormat): string
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world (its active locale). |
| `seconds` | `number` | no | Seconds since 1970-01-01 00:00 UTC; fractions are dropped. |
| `format` | `DateFormat` | yes | The style and offset (medium, UTC). |

**Returns** `string`.

Binding symbol `og_i18n_format_date` · publish builds: kept

```ogt
fn formatDate {
  doc "A date in the active locale, in the style asked for: `Nov 2, 2026` (en), `02.11.2026` (de)."
  param world World "The world (its active locale)."
  param seconds f64 "Seconds since 1970-01-01 00:00 UTC; fractions are dropped."
  param format DateFormat "The style and offset (medium, UTC)." {
    optional true
  }
  returns String
}
```

<sub>`api.ogt` line 15617</sub>
