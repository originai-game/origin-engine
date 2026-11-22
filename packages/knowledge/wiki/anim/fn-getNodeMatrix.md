<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`anim`](../anim.md) · [index](../index.md)

<a id="fn-getNodeMatrix"></a>

### `og.anim.getNodeMatrix`

A node's matrix in the skeleton entity's frame, including node ancestors, as three.js Bone.matrixWorld relative to its SkinnedMesh; no entity/parent transforms. Game systems run before og.anim: reads see the previous completed animation tick's final FootLock/IK/LookAt/spring pose, never the interpolated draw or this tick's new inputs. Rest before the first tick or after an asset change. Reads never advance animation; snapshot restore is immediate. Declare read access to og.Skeleton.

See `animation §3`.

```ts
og.anim.getNodeMatrix(world: World, entity: Entity, node: number): Result<Matrix4>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world whose animation poses the entity. |
| `entity` | `Entity` | no | The entity with og.Skeleton. |
| `node` | `number` | no | An Animation.nodes index, not a skin joint index: skinJoints[skin][joint] converts one. Non-joint socket helpers work too. |

**Returns** `Result<Matrix4>`: Refused for an invalid entity, missing og.Skeleton, unloaded asset or out-of-range node.

Binding symbol `og_anim_get_node_matrix` · publish builds: kept

```ogt
fn getNodeMatrix {
  doc "A node's matrix in the skeleton entity's frame, including node ancestors, as three.js Bone.matrixWorld relative to its SkinnedMesh; no entity/parent transforms. Game systems run before og.anim: reads see the previous completed animation tick's final FootLock/IK/LookAt/spring pose, never the interpolated draw or this tick's new inputs. Rest before the first tick or after an asset change. Reads never advance animation; snapshot restore is immediate. Declare read access to og.Skeleton."
  guide "animation §3"
  param world World "The world whose animation poses the entity."
  param entity Entity "The entity with og.Skeleton."
  param node u32 "An Animation.nodes index, not a skin joint index: skinJoints[skin][joint] converts one. Non-joint socket helpers work too."
  returns Result.Matrix4 "Refused for an invalid entity, missing og.Skeleton, unloaded asset or out-of-range node."
}
```

<sub>Module `anim` line 631</sub>
