<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-SliceEdges"></a>

### `SliceEdges` (enum)

How a nine-slice's edges and centre fill their length.

A string: one of the variants below. The zero value is `"stretch"`.

| Variant | Doc |
|---|---|
| `"stretch"` | Stretched. |
| `"repeat"` | Tiled at the border's scale, cut at the end. |
| `"round"` | Tiled a whole number of times, stretched to fit. |

```ogt
enum SliceEdges {
  doc "How a nine-slice's edges and centre fill their length."
  variant stretch "Stretched."
  variant repeat "Tiled at the border's scale, cut at the end."
  variant round "Tiled a whole number of times, stretched to fit."
}
```

<sub>Module `ui` line 3390</sub>
