<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`physics`](../physics.md) · [index](../index.md)

<a id="type-BonePose"></a>

### `BonePose` (struct)

A ragdoll bone's pose as the last step left it (blended with the animation by og.Ragdoll.weight).

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `position` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | — | The bone's position in world space. |
| `rotation` | `Quaternion` | `{ x: 0, y: 0, z: 0, w: 1 }` | — | — | — | Its rotation in world space. |

```ogt
struct BonePose {
  doc "A ragdoll bone's pose as the last step left it (blended with the animation by og.Ragdoll.weight)."
  field position Vector3 "The bone's position in world space." {
    unit m
  }
  field rotation Quaternion "Its rotation in world space."
}
```

<sub>Module `physics` line 2036</sub>
