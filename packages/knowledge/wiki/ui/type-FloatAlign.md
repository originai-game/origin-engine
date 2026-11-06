<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-FloatAlign"></a>

### `FloatAlign` (enum)

How a floating surface lines up with what it belongs to.

A string: one of the variants below. The zero value is `"start"`.

| Variant | Doc |
|---|---|
| `"start"` | Their start edges. |
| `"center"` | Centred on it. |
| `"end"` | Their end edges. |

```ogt
enum FloatAlign {
  doc "How a floating surface lines up with what it belongs to."
  variant start "Their start edges."
  variant center "Centred on it."
  variant end "Their end edges."
}
```

<sub>`api.ogt` line 17802</sub>
