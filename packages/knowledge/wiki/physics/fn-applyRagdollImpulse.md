<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`physics`](../physics.md) · [index](../index.md)

<a id="fn-applyRagdollImpulse"></a>

### `og.physics.applyRagdollImpulse`

Apply a world-space impulse at a world-space point to one dynamic ragdoll bone.

```ts
og.physics.applyRagdollImpulse(world: World, entity: Entity, boneIndex: number, impulse: Vector3, point: Vector3): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `entity` | `Entity` | no | The ragdoll owner. |
| `boneIndex` | `number` | no | The bone in rig order. |
| `impulse` | `Vector3` | no | Impulse in N*s. |
| `point` | `Vector3` | no | World-space application point. |

**Returns** `Result<void>`: Refused for an absent or non-dynamic bone or nonfinite input.

Binding symbol `og_physics_apply_ragdoll_impulse` · publish builds: kept

```ogt
fn applyRagdollImpulse {
  doc "Apply a world-space impulse at a world-space point to one dynamic ragdoll bone."
  param world World "The world."
  param entity Entity "The ragdoll owner."
  param boneIndex u32 "The bone in rig order."
  param impulse Vector3 "Impulse in N*s."
  param point Vector3 "World-space application point."
  returns Result.Void "Refused for an absent or non-dynamic bone or nonfinite input."
}
```

<sub>Module `physics` line 2656</sub>
