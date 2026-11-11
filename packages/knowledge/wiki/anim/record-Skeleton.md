<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`anim`](../anim.md) · [index](../index.md)

<a id="record-Skeleton"></a>

### `og.Skeleton` (component)

A posed skeleton: the entity is drawn skinned by the joint palette og.anim writes each tick. The pose itself (every node's local transform) is world state kept by the `og.anim` state block.

Token `og.Skeleton`, registered under the schema name `og.Skeleton`; its value type is `Skeleton`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `asset` | `SkeletonAsset` | `0` | — | — | — | The skeleton asset (og.load.animation's `skeleton`); 0 is none. |
| `skin` | `number` | `0` | ≥ 0 | — | — | Which of the asset's skins the joint palette is for (glTF skin index). |
| `rootMotion` | `RootMotion` | `"off"` | — | — | — | Root motion extraction: off (animate in place), planar (xz travel and yaw move og.Transform) or full. |
| `rootJoint` | `number` | `0` | ≥ 0 | — | — | The joint whose motion is extracted (an index into the skin's joints; 0 is its first joint, usually the hips). |
| `jointCount` | `number` | `0` | — | — | read only | Written: the number of joints in the palette (0 until the first tick). |

```ogt
component Skeleton {
  doc "A posed skeleton: the entity is drawn skinned by the joint palette og.anim writes each tick. The pose itself (every node's local transform) is world state kept by the `og.anim` state block."
  field asset SkeletonAsset "The skeleton asset (og.load.animation's `skeleton`); 0 is none."
  field skin u32 "Which of the asset's skins the joint palette is for (glTF skin index)." {
    min 0
  }
  field rootMotion RootMotion "Root motion extraction: off (animate in place), planar (xz travel and yaw move og.Transform) or full."
  field rootJoint u32 "The joint whose motion is extracted (an index into the skin's joints; 0 is its first joint, usually the hips)." {
    min 0
  }
  field jointCount u32 "Written: the number of joints in the palette (0 until the first tick)." {
    readonly true
  }
}
```

<sub>Module `anim` line 85</sub>
