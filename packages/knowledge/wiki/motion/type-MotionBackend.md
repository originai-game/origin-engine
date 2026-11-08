<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`motion`](../motion.md) · [index](../index.md)

<a id="type-MotionBackend"></a>

### `MotionBackend` (enum)

Where the motion model runs.

A string: one of the variants below. The zero value is `"none"`.

| Variant | Doc |
|---|---|
| `"none"` | Nowhere yet: the weights are not resident (still loading, or failed: MotionCapabilities.reason says why); agents draw og.anim's pose. |
| `"gpu"` | On the GPU beside the renderer (the opt-in): the same poses only on one host and GPU. |
| `"cpu"` | On the CPU across the lanes (the default): the same poses, bit for bit, on every host. |

```ogt
enum MotionBackend {
  doc "Where the motion model runs."
  variant none "Nowhere yet: the weights are not resident (still loading, or failed: MotionCapabilities.reason says why); agents draw og.anim's pose."
  variant gpu "On the GPU beside the renderer (the opt-in): the same poses only on one host and GPU."
  variant cpu "On the CPU across the lanes (the default): the same poses, bit for bit, on every host."
}
```

<sub>Module `motion` line 24</sub>
