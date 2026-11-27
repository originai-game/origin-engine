<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`anim`](../anim.md) · [index](../index.md)

<a id="type-RootMotion"></a>

### `RootMotion` (enum)

What the root joint's motion does to the entity.

A string: one of the variants below. The zero value is `"off"`.

| Variant | Doc |
|---|---|
| `"off"` | The root animates in place; the entity does not move. The root joint's horizontal (xz) travel is held at its rest place, so a travelling clip (a run, a swing on a rope) plays on the spot; its height and turns still animate. |
| `"planar"` | The root's horizontal (xz) travel and yaw move og.Transform. |
| `"full"` | The root's whole translation and rotation move og.Transform. |

```ogt
enum RootMotion {
  doc "What the root joint's motion does to the entity."
  variant off "The root animates in place; the entity does not move. The root joint's horizontal (xz) travel is held at its rest place, so a travelling clip (a run, a swing on a rope) plays on the spot; its height and turns still animate."
  variant planar "The root's horizontal (xz) travel and yaw move og.Transform."
  variant full "The root's whole translation and rotation move og.Transform."
}
```

<sub>Module `anim` line 32</sub>
