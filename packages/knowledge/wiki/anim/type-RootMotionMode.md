<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`anim`](../anim.md) · [index](../index.md)

<a id="type-RootMotionMode"></a>

### `RootMotionMode` (enum)

Where extracted root motion is applied.

A string: one of the variants below. The zero value is `"transform"`.

| Variant | Doc |
|---|---|
| `"transform"` | Write the entity Transform directly. |
| `"controller"` | Pass world displacement to CharacterController for collision-safe sweeping. |

```ogt
enum RootMotionMode {
  doc "Where extracted root motion is applied."
  variant transform "Write the entity Transform directly."
  variant controller "Pass world displacement to CharacterController for collision-safe sweeping."
}
```

<sub>Module `anim` line 47</sub>
