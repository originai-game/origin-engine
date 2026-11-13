<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`physics`](../physics.md) · [index](../index.md)

<a id="fn-ragdollRigFromSkeleton"></a>

### `og.physics.ragdollRigFromSkeleton`

A ragdoll rig from a loaded skeleton (og.load.animation's `skeleton`): a bone per joint of its first skin, parents first, each from its joint's rest pose to its first child joint, every bone naming its joint and the rig naming the skeleton. Pass it to og.physics.setRagdollRig, edited or not.

```ts
og.physics.ragdollRigFromSkeleton(world: World, skeleton: SkeletonAsset): Result<RagdollRigDesc>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `skeleton` | `SkeletonAsset` | no | The skeleton. |

**Returns** `Result<RagdollRigDesc>`: Refused for a skeleton that is not loaded or has no skin.

Binding symbol `og_physics_ragdoll_rig_from_skeleton` · publish builds: kept

```ogt
fn ragdollRigFromSkeleton {
  doc "A ragdoll rig from a loaded skeleton (og.load.animation's `skeleton`): a bone per joint of its first skin, parents first, each from its joint's rest pose to its first child joint, every bone naming its joint and the rig naming the skeleton. Pass it to og.physics.setRagdollRig, edited or not."
  param world World "The world."
  param skeleton SkeletonAsset "The skeleton."
  returns Result.RagdollRigDesc "Refused for a skeleton that is not loaded or has no skin."
}
```

<sub>Module `physics` line 2092</sub>
