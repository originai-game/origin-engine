<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`anim`](../anim.md) · [index](../index.md)

<a id="type-SpringSpace"></a>

### `SpringSpace` (enum)

Where a spring bone's tails move.

A string: one of the variants below. The zero value is `"world"`.

| Variant | Doc |
|---|---|
| `"world"` | The frame the skeleton entity's og.Transform is written in: the chain lags behind the entity's motion (walking, root motion, a jump). |
| `"model"` | The skeleton's model space: only the animation moves the chain (a teleport or a vehicle carrying the entity does not). |

```ogt
enum SpringSpace {
  doc "Where a spring bone's tails move."
  variant world "The frame the skeleton entity's og.Transform is written in: the chain lags behind the entity's motion (walking, root motion, a jump)."
  variant model "The skeleton's model space: only the animation moves the chain (a teleport or a vehicle carrying the entity does not)."
}
```

<sub>Module `anim` line 434</sub>
