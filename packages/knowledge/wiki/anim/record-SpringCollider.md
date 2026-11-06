<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`anim`](../anim.md) · [index](../index.md)

<a id="record-SpringCollider"></a>

### `og.SpringCollider` (component)

A collider spring bones are kept out of, three-vrm's VRMSpringBoneCollider: a sphere or a capsule riding on a joint of a skeleton (a head, a chest, an upper arm), so hair, ears and tails do not swing through the body. Each tick, after a tail of an og.SpringBone of the same skeleton has stepped and before its joint turns, the tail is pushed out of every collider whose `layers` meet the spring's `colliders` (in query order) to `radius` plus the spring's `radius`, keeping its bone's length. Put it on its own entity, one per shape; colliders follow the pose as it is when a chain is swung, so one on a joint an earlier chain swung follows that swing.

Token `og.SpringCollider`, registered under the schema name `og.SpringCollider`; its value type is `SpringCollider`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `skeleton` | `Entity` | `0` | — | — | — | The entity with the og.Skeleton whose spring bones this keeps out. |
| `joint` | `number` | `0` | ≥ 0 | — | — | The joint the collider rides on (an index into the skin's joints). |
| `shape` | `SpringColliderShape` | `"sphere"` | — | — | — | A sphere about `offset`, or a capsule from `offset` to `tail`. |
| `offset` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | — | The sphere's centre, or the capsule's first end, in the joint's frame. |
| `tail` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | — | The capsule's other end, in the joint's frame (a sphere ignores it). |
| `radius` | `number` | `0.1` | ≥ 0 | `m` | tunable | The radius, in metres of the skeleton's model space (the joint's own scale does not apply). |
| `layers` | `number` | `1` | — | — | — | The layers this collider is on, as bits (a spring bone's `colliders` picks them). |

```ogt
component SpringCollider {
  doc "A collider spring bones are kept out of, three-vrm's VRMSpringBoneCollider: a sphere or a capsule riding on a joint of a skeleton (a head, a chest, an upper arm), so hair, ears and tails do not swing through the body. Each tick, after a tail of an og.SpringBone of the same skeleton has stepped and before its joint turns, the tail is pushed out of every collider whose `layers` meet the spring's `colliders` (in query order) to `radius` plus the spring's `radius`, keeping its bone's length. Put it on its own entity, one per shape; colliders follow the pose as it is when a chain is swung, so one on a joint an earlier chain swung follows that swing."
  field skeleton Entity "The entity with the og.Skeleton whose spring bones this keeps out."
  field joint u32 "The joint the collider rides on (an index into the skin's joints)." {
    min 0
  }
  field shape SpringColliderShape "A sphere about `offset`, or a capsule from `offset` to `tail`."
  field offset Vector3 "The sphere's centre, or the capsule's first end, in the joint's frame." {
    unit m
  }
  field tail Vector3 "The capsule's other end, in the joint's frame (a sphere ignores it)." {
    unit m
  }
  field radius f32 "The radius, in metres of the skeleton's model space (the joint's own scale does not apply)." {
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

<sub>`api.ogt` line 11939</sub>
