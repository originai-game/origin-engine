<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`physics`](../physics.md) · [index](../index.md)

<a id="type-JointType"></a>

### `JointType` (enum)

How a joint holds its two bodies: Joint.kind.

See `physics §1`.

A string: one of the variants below. The zero value is `"fixed"`.

| Variant | Doc |
|---|---|
| `"fixed"` | The two bodies move as one. |
| `"revolute"` | A hinge about `axis` (2D: about z), with optional limits and a motor. |
| `"prismatic"` | A slider along `axis`, with optional limits and a motor. |
| `"spring"` | A damped spring pulling the anchors to `restLength` apart. |
| `"rope"` | A rope: the anchors stay at most `length` apart, free inside it. |
| `"ball"` | A ball and socket (3D; the 2D profile treats it as a hinge). |

```ogt
enum JointType {
  doc "How a joint holds its two bodies: Joint.kind."
  guide "physics §1"
  variant fixed "The two bodies move as one."
  variant revolute "A hinge about `axis` (2D: about z), with optional limits and a motor."
  variant prismatic "A slider along `axis`, with optional limits and a motor."
  variant spring "A damped spring pulling the anchors to `restLength` apart."
  variant rope "A rope: the anchors stay at most `length` apart, free inside it."
  variant ball "A ball and socket (3D; the 2D profile treats it as a hinge)."
}
```

<sub>Module `physics` line 348</sub>
