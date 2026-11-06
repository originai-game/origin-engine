<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`anim`](../anim.md) · [index](../index.md)

<a id="type-RootMotion"></a>

### `RootMotion` (enum)

What the root joint's motion does to the entity.

A string: one of the variants below. The zero value is `"off"`.

| Variant | Doc |
|---|---|
| `"off"` | The root animates in place; the entity does not move. |
| `"planar"` | The root's horizontal (xz) travel and yaw move og.Transform. |
| `"full"` | The root's whole translation and rotation move og.Transform. |

```ogt
enum RootMotion {
  doc "What the root joint's motion does to the entity."
  variant off "The root animates in place; the entity does not move."
  variant planar "The root's horizontal (xz) travel and yaw move og.Transform."
  variant full "The root's whole translation and rotation move og.Transform."
}
```

<sub>`api.ogt` line 11460</sub>
