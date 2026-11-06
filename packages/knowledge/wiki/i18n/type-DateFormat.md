<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`i18n`](../i18n.md) · [index](../index.md)

<a id="type-DateFormat"></a>

### `DateFormat` (desc struct)

How formatDate writes a moment.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `style` | `DateStyle` | `"medium"` | — | — | — | How much of the date. |
| `offsetMinutes` | `number` | `0` | — | — | — | Minutes ahead of UTC to show it in (a player's time zone, which the game passes in; 540 for Tokyo). |

```ogt
struct DateFormat {
  doc "How formatDate writes a moment."
  desc true
  field style DateStyle "How much of the date."
  field offsetMinutes i32 "Minutes ahead of UTC to show it in (a player's time zone, which the game passes in; 540 for Tokyo)."
}
```

<sub>`api.ogt` line 15533</sub>
