<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`physics`](../physics.md) · [index](../index.md)

<a id="type-RigidBodyType"></a>

### `RigidBodyType` (enum)

A body's simulation mode. Fixed has Rapier's meaning; kinematic is the engine's combined Velocity/Transform contract.

See `physics §1`.

A string: one of the variants below. The zero value is `"dynamic"`.

| Variant | Doc |
|---|---|
| `"dynamic"` | Moved by the solver; Transform and Velocity are written back every tick. |
| `"fixed"` | Not moved by the solver; collides with dynamic bodies. |
| `"kinematic"` | Velocity moves it each tick; a Transform write teleports it first. Without Velocity it stays at Transform. It pushes dynamic bodies and carries controllers. Transform is not a next-position target from which velocity is inferred. |

```ogt
enum RigidBodyType {
  doc "A body's simulation mode. Fixed has Rapier's meaning; kinematic is the engine's combined Velocity/Transform contract."
  guide "physics §1"
  variant dynamic "Moved by the solver; Transform and Velocity are written back every tick."
  variant fixed "Not moved by the solver; collides with dynamic bodies."
  variant kinematic "Velocity moves it each tick; a Transform write teleports it first. Without Velocity it stays at Transform. It pushes dynamic bodies and carries controllers. Transform is not a next-position target from which velocity is inferred."
}
```

<sub>Module `physics` line 27</sub>
