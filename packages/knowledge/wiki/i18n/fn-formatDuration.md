<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`i18n`](../i18n.md) · [index](../index.md)

<a id="fn-formatDuration"></a>

### `og.i18n.formatDuration`

A length of time as a clock face, `1:02:03` from an hour and `2:05` below, in the locale's digits; whole seconds, rounded down. og.ui's `duration` and `clock` formats are the in-document equivalents.

```ts
og.i18n.formatDuration(world: World, seconds: number): string
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world (its active locale). |
| `seconds` | `number` | no | The length in seconds; negative is written with a minus. |

**Returns** `string`.

Binding symbol `og_i18n_format_duration` · publish builds: kept

```ogt
fn formatDuration {
  doc "A length of time as a clock face, `1:02:03` from an hour and `2:05` below, in the locale's digits; whole seconds, rounded down. og.ui's `duration` and `clock` formats are the in-document equivalents."
  param world World "The world (its active locale)."
  param seconds f64 "The length in seconds; negative is written with a minus."
  returns String
}
```

<sub>`api.ogt` line 15631</sub>
