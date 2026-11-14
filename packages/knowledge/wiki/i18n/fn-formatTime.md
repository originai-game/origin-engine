<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`i18n`](../i18n.md) · [index](../index.md)

<a id="fn-formatTime"></a>

### `og.i18n.formatTime`

A time of day in the active locale: `3:05 PM` (en), `15:05` (de).

Also known as: time of day format, 12 hour clock, 24 hour clock

```ts
og.i18n.formatTime(world: World, timeMs: number, format?: TimeFormat): string
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world (its active locale). |
| `timeMs` | `number` | no | Milliseconds since 1970-01-01 00:00 UTC, as og.platform.now and PlatformResult.time; fractions of a second are dropped. |
| `format` | `TimeFormat` | yes | The style and offset (short, UTC). |

**Returns** `string`.

Binding symbol `og_i18n_format_time` · publish builds: kept

```ogt
fn formatTime {
  doc "A time of day in the active locale: `3:05 PM` (en), `15:05` (de)."
  aliases "time of day format, 12 hour clock, 24 hour clock"
  param world World "The world (its active locale)."
  param timeMs f64 "Milliseconds since 1970-01-01 00:00 UTC, as og.platform.now and PlatformResult.time; fractions of a second are dropped."
  param format TimeFormat "The style and offset (short, UTC)." {
    optional true
  }
  returns String
}
```

<sub>Module `i18n` line 131</sub>
