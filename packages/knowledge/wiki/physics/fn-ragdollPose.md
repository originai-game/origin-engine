<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`physics`](../physics.md) · [index](../index.md)

<a id="fn-ragdollPose"></a>

### `og.physics.ragdollPose`

An og.Ragdoll's bones after the last step, in rig order (empty before its first step).

```ts
og.physics.ragdollPose(world: World, entity: Entity): Result<readonly BonePose[]>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `entity` | `Entity` | no | The ragdoll's entity. |

**Returns** `Result<readonly BonePose[]>`: Refused for an entity without a ragdoll.

Binding symbol `og_physics_ragdoll_pose` · publish builds: kept

```ogt
fn ragdollPose {
  doc "An og.Ragdoll's bones after the last step, in rig order (empty before its first step)."
  param world World "The world."
  param entity Entity "The ragdoll's entity."
  returns Result.List.BonePose "Refused for an entity without a ragdoll."
}
```

<sub>Module `physics` line 2215</sub>
