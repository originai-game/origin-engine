<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`physics`](../physics.md) · [index](../index.md)

<a id="fn-setRagdollRig"></a>

### `og.physics.setRagdollRig`

Give an entity with an og.Ragdoll its bones (and, from og.physics.ragdollRigFromSkeleton, the skeleton they stand for: its og.Model is then drawn blended from the animation towards the bones by og.Ragdoll.blend). While weight is 0 the bones follow the rig's rest pose at the entity. Rigs are assets, not world state: kept by the engine and given again to a reloaded world.

```ts
og.physics.setRagdollRig(world: World, entity: Entity, desc?: RagdollRigDesc): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `entity` | `Entity` | no | The ragdoll's entity. |
| `desc` | `RagdollRigDesc` | yes | The bones, parents first. |

**Returns** `Result<void>`: Refused for an empty rig or a parent that does not come before its child.

Binding symbol `og_physics_set_ragdoll_rig` · publish builds: kept

```ogt
fn setRagdollRig {
  doc "Give an entity with an og.Ragdoll its bones (and, from og.physics.ragdollRigFromSkeleton, the skeleton they stand for: its og.Model is then drawn blended from the animation towards the bones by og.Ragdoll.blend). While weight is 0 the bones follow the rig's rest pose at the entity. Rigs are assets, not world state: kept by the engine and given again to a reloaded world."
  param world World "The world."
  param entity Entity "The ragdoll's entity."
  param desc RagdollRigDesc "The bones, parents first."
  returns Result.Void "Refused for an empty rig or a parent that does not come before its child."
}
```

<sub>Module `physics` line 2071</sub>
