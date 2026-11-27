<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`anim`](../anim.md) · [index](../index.md)

<a id="record-SpringCollider"></a>

### `og.SpringCollider` (component)

A joint-local sphere, capsule or plane that keeps whole SpringBone segments out of the body. Matching layers are projected in query order, preserving bone length. Radius follows joint and skeleton scale; a plane uses tail as its normal. One entity per shape, placed from the pose when the chain swings.

Token `og.SpringCollider`, registered under the schema name `og.SpringCollider`; its value type is `SpringCollider`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `skeleton` | `Entity` | `0` | — | — | — | The entity with the og.Skeleton whose spring bones this keeps out. |
| `joint` | `number` | `0` | ≥ 0 | — | — | The joint the collider rides on (an index into the skin's joints). |
| `shape` | `SpringColliderShape` | `"sphere"` | — | — | — | Sphere about offset, capsule from offset to tail, or plane through offset. |
| `offset` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | — | The sphere's centre, or the capsule's first end, in the joint's frame. |
| `tail` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | — | Capsule endpoint or plane normal in the joint's frame; ignored by a sphere. |
| `radius` | `number` | `0.1` | ≥ 0 | `m` | tunable | The radius in the joint's local frame, scaled with the joint and skeleton. |
| `layers` | `number` | `1` | — | — | — | The layers this collider is on, as bits (a spring bone's `colliders` picks them). |

```ogt
component SpringCollider {
  doc "A joint-local sphere, capsule or plane that keeps whole SpringBone segments out of the body. Matching layers are projected in query order, preserving bone length. Radius follows joint and skeleton scale; a plane uses tail as its normal. One entity per shape, placed from the pose when the chain swings."
  field skeleton Entity "The entity with the og.Skeleton whose spring bones this keeps out."
  field joint u32 "The joint the collider rides on (an index into the skin's joints)." {
    min 0
  }
  field shape SpringColliderShape "Sphere about offset, capsule from offset to tail, or plane through offset."
  field offset Vector3 "The sphere's centre, or the capsule's first end, in the joint's frame." {
    unit m
  }
  field tail Vector3 "Capsule endpoint or plane normal in the joint's frame; ignored by a sphere." {
    unit m
  }
  field radius f32 "The radius in the joint's local frame, scaled with the joint and skeleton." {
    default 0.1
    min 0
    unit m
    tunable true
  }
  field layers u32 "The layers this collider is on, as bits (a spring bone's `colliders` picks them)." {
    default 1
  }
}
```

<sub>Module `anim` line 604</sub>
