<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`i18n`](../i18n.md) · [index](../index.md)

<a id="type-DateStyle"></a>

### `DateStyle` (enum)

How much of a date to write (CLDR's styles; en shown).

A string: one of the variants below. The zero value is `"medium"`.

| Variant | Doc |
|---|---|
| `"medium"` | `Nov 2, 2026`. |
| `"short"` | `11/2/26`. |
| `"long"` | `November 2, 2026`. |
| `"full"` | `Monday, November 2, 2026`. |
| `"monthYear"` | A calendar heading: `November 2026`, `2026年11月`. |
| `"monthName"` | The stand-alone month name: `November`. |
| `"weekday"` | The weekday name: `Monday`. |
| `"weekdayShort"` | The abbreviated weekday: `Mon`. |

```ogt
enum DateStyle {
  doc "How much of a date to write (CLDR's styles; en shown)."
  variant medium "`Nov 2, 2026`."
  variant short "`11/2/26`."
  variant long "`November 2, 2026`."
  variant full "`Monday, November 2, 2026`."
  variant monthYear "A calendar heading: `November 2026`, `2026年11月`."
  variant monthName "The stand-alone month name: `November`."
  variant weekday "The weekday name: `Monday`."
  variant weekdayShort "The abbreviated weekday: `Mon`."
}
```

<sub>Module `i18n` line 6</sub>
