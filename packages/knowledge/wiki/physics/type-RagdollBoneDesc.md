<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`physics`](../physics.md) · [index](../index.md)

<a id="type-RagdollBoneDesc"></a>

### `RagdollBoneDesc` (desc struct)

A rest-pose bone: a shaped collider along axis for length, with explicit mass and joint constraints.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `parent` | `number` | `-1` | — | — | — | The parent bone's index (it must come earlier); -1 for the root. |
| `position` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | — | The bone's rest position relative to the entity. |
| `quaternion` | `Quaternion` | `{ x: 0, y: 0, z: 0, w: 1 }` | — | — | — | The bone's rest rotation relative to the entity; a zero quaternion is no rotation. |
| `length` | `number` | `0.25` | — | `m` | — | How long the bone is (to its child). |
| `axis` | `Vector3` | `{ x: 0, y: 1, z: 0 }` | — | — | — | The direction the bone points in its own frame (towards its child). |
| `joint` | `number` | `-1` | — | — | — | The skeleton joint (its index in the skin) the bone stands for, so the model is drawn with it; -1 for none. |
| `mass` | `number` | `0` | ≥ 0 | `kg` | — | Per-bone mass in kg; zero disables the override and uses Ragdoll.mass distributed by length. |
| `shape` | `RagdollBoneShape` | `"capsule"` | — | — | — | Collider shape along the bone. |
| `jointKind` | `RagdollJointKind` | `"coneTwist"` | — | — | — | Cone/twist about axis, or hinge about hingeAxis. |
| `hingeAxis` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | — | — | Hinge axis in the bone's rest-local frame; required for a hinge. |
| `swing` | `number` | `0` | ≥ 0 | `rad` | — | Per-bone swing half-angle; zero disables the override and uses Ragdoll.jointLimit. |
| `twistMin` | `number` | `0` | — | `rad` | — | Least twist or hinge angle; both bounds zero disables the override. |
| `twistMax` | `number` | `0` | — | `rad` | — | Greatest twist or hinge angle; both bounds zero disables the override. |

```ogt
struct RagdollBoneDesc {
  doc "A rest-pose bone: a shaped collider along axis for length, with explicit mass and joint constraints."
  desc true
  field parent i32 "The parent bone's index (it must come earlier); -1 for the root." {
    default -1
  }
  field position Vector3 "The bone's rest position relative to the entity." {
    unit m
  }
  field quaternion Quaternion "The bone's rest rotation relative to the entity; a zero quaternion is no rotation."
  field length f32 "How long the bone is (to its child)." {
    default 0.25
    unit m
  }
  field axis Vector3 "The direction the bone points in its own frame (towards its child)." {
    default 0 1 0
  }
  field joint i32 "The skeleton joint (its index in the skin) the bone stands for, so the model is drawn with it; -1 for none." {
    default -1
  }
  field mass f32 "Per-bone mass in kg; zero disables the override and uses Ragdoll.mass distributed by length." {
    min 0
    unit kg
  }
  field shape RagdollBoneShape "Collider shape along the bone."
  field jointKind RagdollJointKind "Cone/twist about axis, or hinge about hingeAxis."
  field hingeAxis Vector3 "Hinge axis in the bone's rest-local frame; required for a hinge."
  field swing f32 "Per-bone swing half-angle; zero disables the override and uses Ragdoll.jointLimit." {
    min 0
    unit rad
  }
  field twistMin f32 "Least twist or hinge angle; both bounds zero disables the override." {
    unit rad
  }
  field twistMax f32 "Greatest twist or hinge angle; both bounds zero disables the override." {
    unit rad
  }
}
```

<sub>Module `physics` line 2568</sub>
