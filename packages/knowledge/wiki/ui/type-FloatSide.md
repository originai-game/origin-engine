<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-FloatSide"></a>

### `FloatSide` (enum)

Where a floating surface hangs off what it belongs to; start and end follow the writing direction.

A string: one of the variants below. The zero value is `"below"`.

| Variant | Doc |
|---|---|
| `"below"` | Below it (the default). |
| `"above"` | Above it. |
| `"start"` | At its start. |
| `"end"` | At its end. |

```ogt
enum FloatSide {
  doc "Where a floating surface hangs off what it belongs to; start and end follow the writing direction."
  variant below "Below it (the default)."
  variant above "Above it."
  variant start "At its start."
  variant end "At its end."
}
```

<sub>Module `ui` line 2129</sub>
