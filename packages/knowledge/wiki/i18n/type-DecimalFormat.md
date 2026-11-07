<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`i18n`](../i18n.md) · [index](../index.md)

<a id="type-DecimalFormat"></a>

### `DecimalFormat` (desc struct)

Decimal formatting from the engine's shared CLDR data; no host Intl dependency.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `decimals` | `number \| null` | `null` | — | — | — | Exactly this many fractional digits (clamped to 20); null keeps up to three without trailing zeros. |
| `grouping` | `boolean \| null` | `null` | — | — | — | Use the locale's digit grouping; null defaults to true. |

```ogt
struct DecimalFormat {
  doc "Decimal formatting from the engine's shared CLDR data; no host Intl dependency."
  desc true
  field decimals Option.u32 "Exactly this many fractional digits (clamped to 20); null keeps up to three without trailing zeros."
  field grouping Option.bool "Use the locale's digit grouping; null defaults to true."
}
```

<sub>Module `i18n` line 86</sub>
