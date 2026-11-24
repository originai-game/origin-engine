<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`physics`](../physics.md) · [index](../index.md)

<a id="type-RagdollBoneShape"></a>

### `RagdollBoneShape` (enum)

The collider fitted along a ragdoll bone.

A string: one of the variants below. The zero value is `"capsule"`.

| Variant | Doc |
|---|---|
| `"capsule"` | Capsule along the bone axis. |
| `"box"` | Box along the bone axis. |
| `"sphere"` | Sphere at the bone's midpoint. |

```ogt
enum RagdollBoneShape {
  doc "The collider fitted along a ragdoll bone."
  variant capsule "Capsule along the bone axis."
  variant box "Box along the bone axis."
  variant sphere "Sphere at the bone's midpoint."
}
```

<sub>Module `physics` line 2556</sub>
