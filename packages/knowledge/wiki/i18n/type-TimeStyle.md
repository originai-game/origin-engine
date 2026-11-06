<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`i18n`](../i18n.md) · [index](../index.md)

<a id="type-TimeStyle"></a>

### `TimeStyle` (enum)

How much of a time of day to write (en shown).

A string: one of the variants below. The zero value is `"short"`.

| Variant | Doc |
|---|---|
| `"short"` | `3:05 PM`. |
| `"medium"` | `3:05:09 PM`, with seconds. |

```ogt
enum TimeStyle {
  doc "How much of a time of day to write (en shown)."
  variant short "`3:05 PM`."
  variant medium "`3:05:09 PM`, with seconds."
}
```

<sub>`api.ogt` line 15534</sub>
