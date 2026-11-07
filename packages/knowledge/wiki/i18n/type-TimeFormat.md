<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`i18n`](../i18n.md) · [index](../index.md)

<a id="type-TimeFormat"></a>

### `TimeFormat` (desc struct)

How formatTime writes a moment.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `style` | `TimeStyle` | `"short"` | — | — | — | With or without seconds. |
| `offsetMinutes` | `number` | `0` | — | — | — | Minutes ahead of UTC to show it in. |

```ogt
struct TimeFormat {
  doc "How formatTime writes a moment."
  desc true
  field style TimeStyle "With or without seconds."
  field offsetMinutes i32 "Minutes ahead of UTC to show it in."
}
```

<sub>Module `i18n` line 31</sub>
