<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`physics`](../physics.md) · [index](../index.md)

<a id="record-Buoyancy"></a>

### `og.Buoyancy` (component)

Floats a rigid body in 2D or 3D, or a limp ragdoll's actual bone volumes; a 3D character swims. Cached collider-volume samples accumulate lift, drag and torque from one substep snapshot. Overlaps lift once at the highest enclosing level. Replaces the pool's automatic lift.

See `physics §8`.

Also known as: bob on the sea, float on water, floating, buoyant hull

Token `og.Buoyancy`, registered under the schema name `og.Buoyancy`; its value type is `Buoyancy`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `waterDensity` | `number` | `2` | ≥ 0 | `kgPerM3` | tunable | The water's density, in og.Collider.density's units: the default 2 floats a body of the colliders' default density 1 half under, as wood floats; at 1 the same body hangs just under the surface, under it sinks. |
| `samples` | `number` | `4` | 1 … 12 | — | — | Cells along each side of its colliders' box (1: one point at its centre; up to 12³ points): more follow its shape and the waves more closely. |
| `drag` | `number` | `4` | 0 … 100 | `perSecond` | tunable | How much of its speed through the water it loses each second when fully under (in part when in part); off-centre cells slow its turning too, so it settles instead of bobbing. |
| `angularDrag` | `number` | `3` | 0 … 100 | `perSecond` | tunable | How much of its spin it loses each second when fully under (in part when in part). |
| `submerged` | `number` | `0` | 0 … 1 | — | read only | Written: the fraction of its hull under water after the last step (0 above it or out of water, 1 sunk): for wakes, splashes and a sinking ship; of a limp ragdoll's mass or a character's height. |

```ogt
component Buoyancy {
  aliases "bob on the sea, float on water, floating, buoyant hull"
  doc "Floats a rigid body in 2D or 3D, or a limp ragdoll's actual bone volumes; a 3D character swims. Cached collider-volume samples accumulate lift, drag and torque from one substep snapshot. Overlaps lift once at the highest enclosing level. Replaces the pool's automatic lift."
  guide "physics §8"
  field waterDensity f32 "The water's density, in og.Collider.density's units: the default 2 floats a body of the colliders' default density 1 half under, as wood floats; at 1 the same body hangs just under the surface, under it sinks." {
    default 2
    min 0
    unit kgPerM3
    tunable true
  }
  field samples u32 "Cells along each side of its colliders' box (1: one point at its centre; up to 12³ points): more follow its shape and the waves more closely." {
    default 4
    min 1
    max 12
  }
  field drag f32 "How much of its speed through the water it loses each second when fully under (in part when in part); off-centre cells slow its turning too, so it settles instead of bobbing." {
    default 4
    min 0
    max 100
    unit perSecond
    tunable true
  }
  field angularDrag f32 "How much of its spin it loses each second when fully under (in part when in part)." {
    default 3
    min 0
    max 100
    unit perSecond
    tunable true
  }
  field submerged f32 "Written: the fraction of its hull under water after the last step (0 above it or out of water, 1 sunk): for wakes, splashes and a sinking ship; of a limp ragdoll's mass or a character's height." {
    min 0
    max 1
    readonly true
  }
}
```

<sub>Module `physics` line 2101</sub>
