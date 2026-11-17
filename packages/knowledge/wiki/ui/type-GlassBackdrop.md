<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-GlassBackdrop"></a>

### `GlassBackdrop` (enum)

Whether glass surfaces may blur the frame behind them.

A string: one of the variants below. The zero value is `"none"`.

| Variant | Doc |
|---|---|
| `"none"` | No backdrop blur. |
| `"auto"` | Enable backdrop blur where the style requests glass. |

```ogt
enum GlassBackdrop {
  doc "Whether glass surfaces may blur the frame behind them."
  variant none "No backdrop blur."
  variant auto "Enable backdrop blur where the style requests glass."
}
```

<sub>Module `ui` line 19</sub>
