<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`anim`](../anim.md) · [index](../index.md)

<a id="type-IkSpace"></a>

### `IkSpace` (enum)

What an IK chain's target and pole are relative to.

A string: one of the variants below. The zero value is `"world"`.

| Variant | Doc |
|---|---|
| `"world"` | Positions in the frame the skeleton entity's og.Transform is written in (its position, rotation and scale are undone): true world for a skeleton entity without a parent, its parent's frame for a child (a body under a character capsule takes targets relative to the capsule; transform world points by the parent's inverse yourself). |
| `"model"` | The skeleton's model space. |

```ogt
enum IkSpace {
  doc "What an IK chain's target and pole are relative to."
  variant world "Positions in the frame the skeleton entity's og.Transform is written in (its position, rotation and scale are undone): true world for a skeleton entity without a parent, its parent's frame for a child (a body under a character capsule takes targets relative to the capsule; transform world points by the parent's inverse yourself)."
  variant model "The skeleton's model space."
}
```

<sub>`api.ogt` line 11500</sub>
