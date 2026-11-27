<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`physics`](../physics.md) · [index](../index.md)

<a id="fn-ragdollRigFromSkeleton"></a>

### `og.physics.ragdollRigFromSkeleton`

Build a rest-pose rig from explicitly selected joints of the first skin, parents first. No name-based humanoid inference. Edit per-bone mass, shape and joint limits before setRagdollRig.

```ts
og.physics.ragdollRigFromSkeleton(world: World, skeleton: SkeletonAsset, bones: readonly number[]): Result<RagdollRigDesc>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `skeleton` | `SkeletonAsset` | no | The skeleton. |
| `bones` | `readonly number[]` | no | Explicit nonempty whitelist of skin joint indices. |

**Returns** `Result<RagdollRigDesc>`: Refused for a skeleton that is not loaded or has no skin.

Binding symbol `og_physics_ragdoll_rig_from_skeleton` · publish builds: kept

```ogt
fn ragdollRigFromSkeleton {
  doc "Build a rest-pose rig from explicitly selected joints of the first skin, parents first. No name-based humanoid inference. Edit per-bone mass, shape and joint limits before setRagdollRig."
  param world World "The world."
  param skeleton SkeletonAsset "The skeleton."
  param bones List.u32 "Explicit nonempty whitelist of skin joint indices."
  returns Result.RagdollRigDesc "Refused for a skeleton that is not loaded or has no skin."
}
```

<sub>Module `physics` line 2633</sub>
