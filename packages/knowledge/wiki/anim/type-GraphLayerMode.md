<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`anim`](../anim.md) · [index](../index.md)

<a id="type-GraphLayerMode"></a>

### `GraphLayerMode` (enum)

How a graph layer combines poses.

A string: one of the variants below. The zero value is `"override"`.

| Variant | Doc |
|---|---|
| `"override"` | Replace the pose below by the layer weight. |
| `"additive"` | Add the difference from the motion's first frame. |

```ogt
enum GraphLayerMode {
  doc "How a graph layer combines poses."
  variant override "Replace the pose below by the layer weight."
  variant additive "Add the difference from the motion's first frame."
}
```

<sub>Module `anim` line 689</sub>
