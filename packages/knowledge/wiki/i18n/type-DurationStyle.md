<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`i18n`](../i18n.md) · [index](../index.md)

<a id="type-DurationStyle"></a>

### `DurationStyle` (enum)

How a duration is displayed.

A string: one of the variants below. The zero value is `"clock"`.

| Variant | Doc |
|---|---|
| `"clock"` | Clock face: `1:02:03`, whole seconds rounded down. |
| `"short"` | Narrow largest-unit label: `2h`, `3m`, rounded up; under a minute, whole seconds. |

```ogt
enum DurationStyle {
  doc "How a duration is displayed."
  variant clock "Clock face: `1:02:03`, whole seconds rounded down."
  variant short "Narrow largest-unit label: `2h`, `3m`, rounded up; under a minute, whole seconds."
}
```

<sub>Module `i18n` line 142</sub>
