<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-Cross"></a>

### `Cross` (enum)

Where children sit across the main axis.

A string: one of the variants below. The zero value is `"start"`.

| Variant | Doc |
|---|---|
| `"start"` | At the start. |
| `"center"` | Centred. |
| `"end"` | At the end. |
| `"stretch"` | Stretched to the container's inner size. |
| `"baseline"` | In a row, children share their first text baseline (rows of mixed type sizes); a column treats it as start. |

```ogt
enum Cross {
  doc "Where children sit across the main axis."
  variant start "At the start."
  variant center "Centred."
  variant end "At the end."
  variant stretch "Stretched to the container's inner size."
  variant baseline "In a row, children share their first text baseline (rows of mixed type sizes); a column treats it as start."
}
```

<sub>Module `ui` line 20</sub>
