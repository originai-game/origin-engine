<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-ColorVisionFilter"></a>

### `ColorVisionFilter` (enum)

What UiDesc.colorVision does to the frame.

A string: one of the variants below. The zero value is `"correct"`.

| Variant | Doc |
|---|---|
| `"correct"` | Recolours the frame so what the deficiency confuses is told apart (daltonisation). |
| `"simulate"` | Shows the frame as a viewer with the deficiency sees it. |

```ogt
enum ColorVisionFilter {
  doc "What UiDesc.colorVision does to the frame."
  variant correct "Recolours the frame so what the deficiency confuses is told apart (daltonisation)."
  variant simulate "Shows the frame as a viewer with the deficiency sees it."
}
```

<sub>Module `ui` line 784</sub>
