<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`motion`](../motion.md) · [index](../index.md)

<a id="type-MotionBackend"></a>

### `MotionBackend` (enum)

Where the motion model runs.

A string: one of the variants below. The zero value is `"none"`.

| Variant | Doc |
|---|---|
| `"none"` | Nowhere yet: the weights are not resident (still loading, or failed: MotionCapabilities.reason says why); agents draw og.anim's pose. |
| `"gpu"` | On the GPU beside the renderer. |

```ogt
enum MotionBackend {
  doc "Where the motion model runs."
  variant none "Nowhere yet: the weights are not resident (still loading, or failed: MotionCapabilities.reason says why); agents draw og.anim's pose."
  variant gpu "On the GPU beside the renderer."
}
```

<sub>`api.ogt` line 26652</sub>
