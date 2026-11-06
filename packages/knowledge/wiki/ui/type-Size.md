<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-Size"></a>

### `Size` (union)

A size along one axis. Absent (null) is fit: the content's size.

Tagged by `kind`.

| Case | Payload | Doc |
|---|---|---|
| `kind: "fit"` | [`SizeRange`](../ui/type-SizeRange.md#type-SizeRange) | The content's size, within the range. |
| `kind: "grow"` | [`SizeRange`](../ui/type-SizeRange.md#type-SizeRange) | The content's size, then a share of the parent's free space, within the range. |
| `kind: "fixed"` | [`FixedSize`](../ui/type-FixedSize.md#type-FixedSize) | Exactly this size. |
| `kind: "percent"` | [`PercentSize`](../ui/type-PercentSize.md#type-PercentSize) | A fraction of the parent's inner size. |

```ogt
union Size {
  doc "A size along one axis. Absent (null) is fit: the content's size."
  tag kind
  case fit SizeRange "The content's size, within the range."
  case grow SizeRange "The content's size, then a share of the parent's free space, within the range."
  case fixed FixedSize "Exactly this size."
  case percent PercentSize "A fraction of the parent's inner size."
}
```

<sub>`api.ogt` line 15808</sub>
