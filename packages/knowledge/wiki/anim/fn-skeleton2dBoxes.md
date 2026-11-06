<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`anim`](../anim.md) · [index](../index.md)

<a id="fn-skeleton2dBoxes"></a>

### `og.anim.skeleton2dBoxes`

The bounding boxes an og.Skeleton2D's pose showed when og.skeleton2d last ran (after the game's systems, so a system reads the previous tick's): hurt and hit boxes a clip switches on and off with its slots' attachments, to test against points or other shapes. Empty for a rig without boxes or when no slot shows one. They are part of the og.skeleton2d state block, so a restored snapshot answers what it saved.

```ts
og.anim.skeleton2dBoxes(world: World, entity: Entity): Result<readonly Skeleton2DBox[]>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world whose animation poses the skeleton. |
| `entity` | `Entity` | no | The entity with the og.Skeleton2D. |

**Returns** `Result<readonly Skeleton2DBox[]>`: Refused for an entity og.skeleton2d has not posed: no og.Skeleton2D with a loaded rig, or before its first tick.

Binding symbol `og_anim_skeleton2d_boxes` · publish builds: kept

```ogt
fn skeleton2dBoxes {
  doc "The bounding boxes an og.Skeleton2D's pose showed when og.skeleton2d last ran (after the game's systems, so a system reads the previous tick's): hurt and hit boxes a clip switches on and off with its slots' attachments, to test against points or other shapes. Empty for a rig without boxes or when no slot shows one. They are part of the og.skeleton2d state block, so a restored snapshot answers what it saved."
  param world World "The world whose animation poses the skeleton."
  param entity Entity "The entity with the og.Skeleton2D."
  returns Result.List.Skeleton2DBox "Refused for an entity og.skeleton2d has not posed: no og.Skeleton2D with a loaded rig, or before its first tick."
}
```

<sub>`api.ogt` line 12336</sub>
