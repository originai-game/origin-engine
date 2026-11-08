<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-MassSpace"></a>

### `MassSpace` (enum)

Where a mass element's position is.

A string: one of the variants below. The zero value is `"world"`.

| Variant | Doc |
|---|---|
| `"world"` | A world point, projected by the GPU each frame. |
| `"screen"` | CSS pixels of the screen. |

```ogt
enum MassSpace {
  doc "Where a mass element's position is."
  variant world "A world point, projected by the GPU each frame."
  variant screen "CSS pixels of the screen."
}
```

<sub>Module `ui` line 7494</sub>
