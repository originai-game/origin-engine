<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`physics`](../physics.md) · [index](../index.md)

<a id="type-RagdollBoneDesc"></a>

### `RagdollBoneDesc` (desc struct)

One bone of a ragdoll rig: a capsule from the bone's position along `axis` for `length`.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `parent` | `number` | `-1` | — | — | — | The parent bone's index (it must come earlier); -1 for the root. |
| `position` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | — | The bone's rest position relative to the entity. |
| `rotation` | `Quaternion` | `{ x: 0, y: 0, z: 0, w: 1 }` | — | — | — | The bone's rest rotation relative to the entity; a zero quaternion is no rotation. |
| `length` | `number` | `0.25` | — | `m` | — | How long the bone is (to its child). |
| `axis` | `Vector3` | `{ x: 0, y: 1, z: 0 }` | — | — | — | The direction the bone points in its own frame (towards its child). |
| `joint` | `number` | `-1` | — | — | — | The skeleton joint (its index in the skin) the bone stands for, so the model is drawn with it; -1 for none. |

```ogt
struct RagdollBoneDesc {
  doc "One bone of a ragdoll rig: a capsule from the bone's position along `axis` for `length`."
  desc true
  field parent i32 "The parent bone's index (it must come earlier); -1 for the root." {
    default -1
  }
  field position Vector3 "The bone's rest position relative to the entity." {
    unit m
  }
  field rotation Quaternion "The bone's rest rotation relative to the entity; a zero quaternion is no rotation."
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
}
```

<sub>`api.ogt` line 10430</sub>
