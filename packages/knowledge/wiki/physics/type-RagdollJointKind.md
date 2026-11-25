<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`physics`](../physics.md) · [index](../index.md)

<a id="type-RagdollJointKind"></a>

### `RagdollJointKind` (enum)

A ragdoll joint's angular degrees of freedom.

A string: one of the variants below. The zero value is `"coneTwist"`.

| Variant | Doc |
|---|---|
| `"coneTwist"` | Swing cone and twist about the bone axis. |
| `"hinge"` | One angular degree of freedom about hingeAxis. |

```ogt
enum RagdollJointKind {
  doc "A ragdoll joint's angular degrees of freedom."
  variant coneTwist "Swing cone and twist about the bone axis."
  variant hinge "One angular degree of freedom about hingeAxis."
}
```

<sub>Module `physics` line 2564</sub>
