<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`anim`](../anim.md) · [index](../index.md)

<a id="type-SpringColliderShape"></a>

### `SpringColliderShape` (enum)

The shape of a spring collider.

A string: one of the variants below. The zero value is `"sphere"`.

| Variant | Doc |
|---|---|
| `"sphere"` | A ball of `radius` about `offset`. |
| `"capsule"` | A capsule of `radius` about the segment from `offset` to `tail`. |

```ogt
enum SpringColliderShape {
  doc "The shape of a spring collider."
  variant sphere "A ball of `radius` about `offset`."
  variant capsule "A capsule of `radius` about the segment from `offset` to `tail`."
}
```

<sub>Module `anim` line 508</sub>
