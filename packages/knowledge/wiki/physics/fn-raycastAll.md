<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`physics`](../physics.md) · [index](../index.md)

<a id="fn-raycastAll"></a>

### `og.physics.raycastAll`

Every collider along a ray within its reach, one hit each at its first crossing, nearest first (ties in collider entity order, so every host and run list them alike), at most `maxHits`: what a piercing shot passes through, everything under a cursor. The same view of the colliders as og.physics.raycast; its first hit is raycast's.

```ts
og.physics.raycastAll(world: World, desc?: RaycastAllDesc): Result<readonly RaycastHit[]>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `desc` | `RaycastAllDesc` | yes | The ray, the filter and the cap. |

**Returns** `Result<readonly RaycastHit[]>`: The hits, nearest first; empty when nothing is hit; refused for a zero direction.

Binding symbol `og_physics_raycast_all` · publish builds: kept

```ogt
fn raycastAll {
  doc "Every collider along a ray within its reach, one hit each at its first crossing, nearest first (ties in collider entity order, so every host and run list them alike), at most `maxHits`: what a piercing shot passes through, everything under a cursor. The same view of the colliders as og.physics.raycast; its first hit is raycast's."
  param world World "The world."
  param desc RaycastAllDesc "The ray, the filter and the cap."
  returns Result.List.RaycastHit "The hits, nearest first; empty when nothing is hit; refused for a zero direction."
}
```

<sub>`api.ogt` line 10301</sub>
