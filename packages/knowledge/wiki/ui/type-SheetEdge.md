<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-SheetEdge"></a>

### `SheetEdge` (enum)

The edge a sheet slides in from; start and end follow the writing direction.

A string: one of the variants below. The zero value is `"bottom"`.

| Variant | Doc |
|---|---|
| `"bottom"` | The bottom (the default). |
| `"end"` | The end side. |
| `"start"` | The start side. |

```ogt
enum SheetEdge {
  doc "The edge a sheet slides in from; start and end follow the writing direction."
  variant bottom "The bottom (the default)."
  variant end "The end side."
  variant start "The start side."
}
```

<sub>`api.ogt` line 17973</sub>
