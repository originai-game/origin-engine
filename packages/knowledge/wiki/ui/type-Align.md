<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-Align"></a>

### `Align` (enum)

Where children sit along the main axis.

A string: one of the variants below. The zero value is `"start"`.

| Variant | Doc |
|---|---|
| `"start"` | At the start. |
| `"center"` | Centred. |
| `"end"` | At the end. |
| `"spaceBetween"` | The free space split evenly between children. |

```ogt
enum Align {
  doc "Where children sit along the main axis."
  variant start "At the start."
  variant center "Centred."
  variant end "At the end."
  variant spaceBetween "The free space split evenly between children."
}
```

<sub>`api.ogt` line 15652</sub>
